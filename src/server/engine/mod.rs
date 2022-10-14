use super::main::{
    create_dustdata,
    rustbase::{QueryResult, QueryResultType},
};
use crate::{
    config::schema,
    query::{self, parser::Query},
    utils::crypto::generate_random_string,
};
use bson::to_vec;
use dustdata::DustData;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::spawn;
use tokio::sync::Mutex as TMutex;

#[derive(Clone)]
pub struct Workers {
    pub routers: Arc<Mutex<BTreeMap<String, DustData>>>,
    pub cache: Arc<Mutex<super::cache::Cache>>,
    pub config: schema::RustbaseConfig,
    pub queue: Arc<TMutex<Vec<QueueItem>>>,
    pub processed_queue: Arc<TMutex<BTreeMap<String, QueryResult>>>,
}

pub struct QueueItem {
    pub id: String,
    pub query: Query,
    pub database: String,
}

impl Workers {
    pub async fn new(
        routers: Arc<Mutex<BTreeMap<String, DustData>>>,
        cache: Arc<Mutex<super::cache::Cache>>,
        config: schema::RustbaseConfig,
    ) -> Arc<TMutex<Self>> {
        let _s = Self {
            routers,
            cache,
            config: config.clone(),
            queue: Arc::new(TMutex::new(Vec::new())),
            processed_queue: Arc::new(TMutex::new(BTreeMap::new())),
        };

        let _s = Arc::new(TMutex::new(_s));

        Workers::work(_s.clone(), config.database.threads);

        _s
    }

    pub fn work(this: Arc<TMutex<Self>>, thread_size: usize) {
        for _ in 0..thread_size {
            let this = this.clone();
            spawn(async move {
                loop {
                    let this = this.lock().await;
                    let mut queue = this.queue.lock().await;

                    if queue.len() > 0 {
                        let item = queue.remove(0);

                        let database = item.database;
                        let query = item.query;
                        let id = item.id;

                        let response = match query {
                            Query::Insert(query) => this.insert(query, database),
                            Query::Get(query) => this.get(query, database),
                            Query::Update(query) => this.update(query, database),
                            Query::Delete(query) => this.delete(query, database),
                            Query::List => this.list(database),
                        };

                        this.processed_queue.lock().await.insert(id, response);
                    }
                }
            });
        }
    }

    pub async fn add_to_worker(&self, database: String, query: Query) -> String {
        let mut queue = self.queue.lock().await;
        let id = generate_random_string(20);
        queue.push(QueueItem {
            query,
            database,
            id: id.clone(),
        });

        id
    }

    pub async fn process(this: Arc<TMutex<Self>>, database: String, query: Query) -> QueryResult {
        let id = this
            .clone()
            .lock()
            .await
            .add_to_worker(database, query)
            .await;

        spawn(async move {
            loop {
                let this = this.lock().await;
                let queue = this.processed_queue.lock().await;

                if queue.contains_key(&id) {
                    return queue.get(&id).unwrap().clone();
                }
            }
        })
        .await
        .unwrap()
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
