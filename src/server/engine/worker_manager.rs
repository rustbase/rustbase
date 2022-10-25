use super::super::{
    cache::Cache,
    main::{
        create_dustdata,
        rustbase::{QueryResult, QueryResultType},
    },
};
use crate::{
    config::schema,
    query::{self, parser::Query},
};
use dustdata::DustData;
use std::{collections::BTreeMap, sync::Arc};
use std::{path::Path, sync::Mutex};
use tokio::{
    spawn,
    sync::{mpsc, watch, Mutex as TMutex, RwLock},
};

pub struct WorkerManager {
    pub workers: Vec<(Worker, InSendOutRecv)>,
}

type InSendOutRecv = (
    watch::Sender<(Query, String)>,
    ArcTMutex<mpsc::Receiver<QueryResult>>,
);
type ArcMutex<T> = Arc<Mutex<T>>;
type ArcTMutex<T> = Arc<TMutex<T>>;

impl WorkerManager {
    pub async fn new(
        routers: ArcMutex<BTreeMap<String, DustData>>,
        cache: ArcMutex<Cache>,
        config: schema::RustbaseConfig,
        workers_count: usize,
    ) -> Self {
        let mut workers = Vec::new();

        for _ in 0..workers_count {
            let (in_sender, in_receiver) = watch::channel((Query::None, String::new()));
            let (out_sender, out_receiver) = mpsc::channel(100);

            let engine = Engine {
                routers: routers.clone(),
                cache: cache.clone(),
                config: config.clone(),
            };

            let mut worker = Worker::new(in_receiver, out_sender);

            worker.run(engine);

            workers.push((worker, (in_sender, Arc::new(TMutex::new(out_receiver)))));
        }

        Self { workers }
    }

    pub async fn process(&self, query: Query, database: String) -> QueryResult {
        let iter = self.workers.iter();

        for (worker, (in_sender, out_receiver)) in iter {
            if worker.is_available().await {
                let out_receiver = out_receiver.clone();

                in_sender.send((query, database)).unwrap();
                let result = out_receiver.lock().await.recv().await.unwrap();

                return result;
            }
        }

        // critical error
        // when all workers are busy
        // this should never happen
        // we should fix this.
        // hackers can use a DDOS attack to crash the server :(

        QueryResult {
            result_type: QueryResultType::Error as i32,
            error_message: Some("workers.notAvailable".to_string()),
            bson: None,
            message: None,
        }
    }
}

pub struct Engine {
    pub routers: TRouters,
    pub cache: TCache,
    pub config: schema::RustbaseConfig,
}

pub type TCache = Arc<Mutex<Cache>>;
pub type TRouters = Arc<Mutex<BTreeMap<String, DustData>>>;

pub struct Worker {
    pub in_receiver: watch::Receiver<(Query, String)>,
    pub out_sender: mpsc::Sender<QueryResult>,
    pub is_available: Arc<RwLock<bool>>,
}

impl Worker {
    pub fn insert(
        query: query::parser::InsertQuery,
        database: String,
        routers: TRouters,
        config: schema::RustbaseConfig,
    ) -> QueryResult {
        let mut routers = routers.lock().unwrap();

        let dd = routers.get_mut(&database);

        if dd.is_none() {
            let dd = create_dustdata(
                Path::new(&config.database.path)
                    .join(database.clone())
                    .to_str()
                    .unwrap()
                    .to_string(),
            );

            routers.insert(database.clone(), dd);
        }

        let dd = routers.get_mut(&database).unwrap();

        if dd.contains(&query.key) {
            drop(routers);
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

        QueryResult {
            error_message: None,
            result_type: QueryResultType::Ok as i32,
            bson: None,
            message: None,
        }
    }

    pub fn get(
        query: query::parser::GetQuery,
        database: String,
        cache: TCache,
        routers: TRouters,
    ) -> QueryResult {
        let mut cache = cache.lock().unwrap();

        let cache_key = format!("{}:{}", database, query.key);

        if cache.contains(cache_key.clone()) {
            let v = cache.get(&cache_key).unwrap().clone();

            return QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: Some(bson::to_vec(&v).unwrap()),
                message: None,
            };
        }

        let mut routers = routers.lock().unwrap();

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
                bson: Some(bson::to_vec(&value).unwrap()),
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

    pub fn update(
        query: query::parser::UpdateQuery,
        database: String,
        cache: TCache,
        routers: TRouters,
    ) -> QueryResult {
        let mut cache = cache.lock().unwrap();

        if cache.contains(query.key.clone()) {
            cache.remove(&query.key).unwrap();
        }

        let mut routers = routers.lock().unwrap();

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

    pub fn delete(
        query: query::parser::DeleteQuery,
        database: String,
        cache: TCache,
        routers: TRouters,
    ) -> QueryResult {
        let mut routers = routers.lock().unwrap();

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

            let mut cache = cache.lock().unwrap();
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

    pub fn list(database: String, routers: TRouters) -> QueryResult {
        let mut routers = routers.lock().unwrap();

        if routers.contains_key(&database) {
            let dd = routers.get_mut(&database).unwrap();
            let list = dd.list_keys().unwrap();

            let doc = bson::doc! {
                "_l": list
            };

            QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: Some(bson::to_vec(&doc).unwrap()),
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

    pub fn query(
        query: Query,
        database: String,
        cache: TCache,
        routers: TRouters,
        config: schema::RustbaseConfig,
    ) -> QueryResult {
        match query {
            Query::Delete(query) => Worker::delete(query, database, cache, routers),
            Query::Insert(query) => Worker::insert(query, database, routers, config),
            Query::Get(query) => Worker::get(query, database, cache, routers),
            Query::Update(query) => Worker::update(query, database, cache, routers),
            Query::List => Worker::list(database, routers),
            Query::None => QueryResult {
                message: None,
                error_message: Some("query.notProvided".to_string()),
                result_type: QueryResultType::SyntaxError as i32,
                bson: None,
            },
        }
    }
}

impl Worker {
    pub fn new(
        in_receiver: watch::Receiver<(Query, String)>,
        out_sender: mpsc::Sender<QueryResult>,
    ) -> Self {
        Self {
            in_receiver,
            out_sender,
            is_available: Arc::new(RwLock::new(true)),
        }
    }

    pub fn run(&mut self, engine: Engine) {
        let mut in_receiver = self.in_receiver.clone();
        let out_sender = self.out_sender.clone();
        let is_available = self.is_available.clone();
        spawn(async move {
            loop {
                let cache = engine.cache.clone();
                let routers = engine.routers.clone();
                let config = engine.config.clone();

                if in_receiver.changed().await.is_ok() {
                    *is_available.write().await = false;
                    let (query, database) = in_receiver.borrow().clone();
                    let result = Worker::query(query, database, cache, routers, config);

                    out_sender.send(result).await.unwrap();
                    *is_available.write().await = true;
                }
            }
        });
    }

    pub async fn is_available(&self) -> bool {
        *self.is_available.read().await
    }
}

pub fn dd_error_code_to_string(code: dustdata::ErrorCode) -> String {
    match code {
        dustdata::ErrorCode::KeyExists => "key.alreadyExists".to_string(),
        dustdata::ErrorCode::KeyNotExists => "key.notExists".to_string(),
        dustdata::ErrorCode::NotFound => "notFound".to_string(),
    }
}
