use super::main::{
    create_dustdata,
    rustbase::{QueryResult, QueryResultType},
};
use crate::{
    config::schema,
    query::{self, parser::Query},
};
use bson::to_vec;
use dustdata::DustData;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{
    spawn,
    sync::{mpsc, watch, Mutex as TMutex},
};

pub struct DatabaseEngine {
    pub routers: Arc<Mutex<BTreeMap<String, DustData>>>,
    pub cache: Arc<Mutex<super::cache::Cache>>,
    pub config: schema::RustbaseConfig,
    pub in_receiver: watch::Receiver<(Query, String)>,
    pub out_sender: mpsc::Sender<QueryResult>,
}

pub struct QueryDatabase {
    pub query: Query,
    pub database: String,
}
impl DatabaseEngine {
    pub async fn new(
        routers: Arc<Mutex<BTreeMap<String, DustData>>>,
        cache: Arc<Mutex<super::cache::Cache>>,
        config: schema::RustbaseConfig,
    ) -> (
        Self,
        watch::Sender<(Query, String)>,
        mpsc::Receiver<QueryResult>,
    ) {
        let (in_sender, in_receiver) = watch::channel((Query::None, "".to_string()));
        let (out_sender, out_receiver) = mpsc::channel(32);

        let _self = Self {
            routers,
            cache,
            config,
            in_receiver,
            out_sender,
        };

        (_self, in_sender, out_receiver)
    }

    pub async fn run(this: Arc<TMutex<Self>>, thread_size: usize) {
        for i in 0..thread_size {
            let this = this.clone();
            spawn(async move {
                let mut this = this.lock().await;
                loop {
                    if this.in_receiver.changed().await.is_err() {
                        break;
                    }

                    let query = this.in_receiver.borrow_and_update().clone();

                    let result = this.query(query.0, query.1);

                    this.out_sender.send(result).await.unwrap();
                }
            });
        }
    }

    pub fn query(&self, query: Query, database: String) -> QueryResult {
        match query {
            Query::Delete(query) => self.delete(query, database),
            Query::Insert(query) => self.insert(query, database),
            Query::Get(query) => self.get(query, database),
            Query::Update(query) => self.update(query, database),
            Query::List => self.list(database),
            Query::None => QueryResult {
                message: None,
                error_message: Some("No query provided".to_string()),
                result_type: QueryResultType::SyntaxError as i32,
                bson: None,
            },
        }
    }

    // --

    pub fn insert(&self, query: query::parser::InsertQuery, database: String) -> QueryResult {
        let mut routers = self.routers.lock().unwrap();

        if let Some(dd) = routers.get_mut(&database) {
            if dd.contains(&query.key) {
                return QueryResult {
                    error_message: Some(dd_error_code_to_string(dustdata::ErrorCode::KeyExists)),
                    result_type: QueryResultType::Error as i32,
                    bson: None,
                    message: None,
                };
            }

            let insert = dd.insert(&query.key, query.value);

            if insert.is_err() {
                return QueryResult {
                    result_type: QueryResultType::Error as i32,
                    error_message: Some(dd_error_code_to_string(insert.err().unwrap().code)),
                    bson: None,
                    message: None,
                };
            }
        } else {
            let mut dd = create_dustdata(
                Path::new(&self.config.database.path)
                    .join(database.clone())
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
            dd.insert(&query.key, query.value).unwrap();

            routers.insert(database.clone(), dd);
        }

        QueryResult {
            error_message: None,
            result_type: QueryResultType::Ok as i32,
            bson: None,
            message: None,
        }
    }

    pub fn get(&self, query: query::parser::GetQuery, database: String) -> QueryResult {
        let mut cache = self.cache.lock().unwrap();

        let cache_key = format!("{}:{}", database, query.key);

        if cache.contains(cache_key.clone()) {
            let v = cache.get(&cache_key).unwrap().clone();

            return QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: Some(to_vec(&v).unwrap()),
                message: None,
            };
        }

        let mut routers = self.routers.lock().unwrap();

        if !routers.contains_key(&database) {
            return QueryResult {
                error_message: Some("database.notFound".to_string()),
                result_type: QueryResultType::NotFound as i32,
                bson: None,
                message: None,
            };
        }

        let dd = routers.get_mut(&database).unwrap();

        let value = dd.get(&query.key).unwrap();

        if let Some(value) = value {
            cache.insert(cache_key, value.clone()).unwrap();

            QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: Some(to_vec(&value).unwrap()),
                message: None,
            }
        } else {
            QueryResult {
                result_type: QueryResultType::NotFound as i32,
                error_message: Some(dd_error_code_to_string(dustdata::ErrorCode::KeyNotExists)),
                bson: None,
                message: None,
            }
        }
    }

    pub fn update(&self, query: query::parser::UpdateQuery, database: String) -> QueryResult {
        let mut cache = self.cache.lock().unwrap();

        if cache.contains(query.key.clone()) {
            cache.remove(&query.key).unwrap();
        }

        let mut routers = self.routers.lock().unwrap();

        if routers.contains_key(&database) {
            let dd = routers.get_mut(&database).unwrap();

            if !dd.contains(&query.key) {
                return QueryResult {
                    error_message: Some(dd_error_code_to_string(dustdata::ErrorCode::KeyNotExists)),
                    result_type: QueryResultType::NotFound as i32,
                    bson: None,
                    message: None,
                };
            }

            let update = dd.update(&query.key, query.value.clone());

            if update.is_err() {
                return QueryResult {
                    error_message: Some(dd_error_code_to_string(update.err().unwrap().code)),
                    result_type: QueryResultType::Error as i32,
                    bson: None,
                    message: None,
                };
            }

            cache.insert(query.key, query.value).unwrap();

            QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: None,
                message: None,
            }
        } else {
            QueryResult {
                error_message: Some("database.notFound".to_string()),
                result_type: QueryResultType::NotFound as i32,
                bson: None,
                message: None,
            }
        }
    }

    pub fn delete(&self, query: query::parser::DeleteQuery, database: String) -> QueryResult {
        let mut routers = self.routers.lock().unwrap();

        if routers.contains_key(&database) {
            let dd = routers.get_mut(&database).unwrap();

            if !dd.contains(&query.key) {
                return QueryResult {
                    error_message: Some(dd_error_code_to_string(dustdata::ErrorCode::KeyNotExists)),
                    result_type: QueryResultType::NotFound as i32,
                    bson: None,
                    message: None,
                };
            }

            let delete = dd.delete(&query.key);

            if delete.is_err() {
                return QueryResult {
                    error_message: Some(dd_error_code_to_string(delete.err().unwrap().code)),
                    result_type: QueryResultType::Error as i32,
                    bson: None,
                    message: None,
                };
            }

            let mut cache = self.cache.lock().unwrap();
            let cache_key = format!("{}:{}", database, query.key);

            if cache.contains(cache_key.clone()) {
                cache.remove(&cache_key).unwrap();
            }

            QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: None,
                message: None,
            }
        } else {
            QueryResult {
                error_message: Some("database.notFound".to_string()),
                result_type: QueryResultType::NotFound as i32,
                bson: None,
                message: None,
            }
        }
    }

    pub fn list(&self, database: String) -> QueryResult {
        let mut routers = self.routers.lock().unwrap();

        if routers.contains_key(&database) {
            let dd = routers.get_mut(&database).unwrap();
            let list = dd.list_keys().unwrap();

            let doc = bson::doc! {
                "_l": list
            };

            QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: Some(to_vec(&doc).unwrap()),
                message: None,
            }
        } else {
            QueryResult {
                error_message: Some("database.notFound".to_string()),
                result_type: QueryResultType::NotFound as i32,
                bson: None,
                message: None,
            }
        }
    }
}

pub fn dd_error_code_to_string(code: dustdata::ErrorCode) -> String {
    match code {
        dustdata::ErrorCode::KeyExists => "key.alreadyExists".to_string(),
        dustdata::ErrorCode::KeyNotExists => "key.notExists".to_string(),
        dustdata::ErrorCode::NotFound => "notFound".to_string(),
    }
}
