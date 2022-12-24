use super::super::{cache::Cache, main::create_dustdata, wirewave};
use crate::{config::schema, query::parser::Query, server::route};
use bson::Bson;
use dustdata::DustData;
use std::{collections::BTreeMap, sync::Arc};
use std::{path::Path, sync::Mutex};
use tokio::{
    spawn,
    sync::{mpsc, watch, Mutex as TMutex, RwLock},
};
use wirewave::server::{Response, Status};

pub struct WorkerManager {
    pub workers: Vec<(Worker, InSendOutRecv)>,
}

type InSendOutRecv = (
    watch::Sender<(Query, String)>,
    ArcTMutex<mpsc::Receiver<Response>>,
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

    pub async fn process(&self, query: Query, database: String) -> Response {
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

        Response {
            status: Status::Error,
            message: Some("workers.notAvailable".to_string()),
            body: None,
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
    pub out_sender: mpsc::Sender<Response>,
    pub is_available: Arc<RwLock<bool>>,
}

impl Worker {
    pub fn insert(
        query: (String, Bson),
        database: String,
        routers: TRouters,
        config: schema::RustbaseConfig,
    ) -> Response {
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
            println!("[Engine] created database {}", database);
        }

        let dd = routers.get_mut(&database).unwrap();

        if dd.contains(&query.0) {
            drop(routers);
            return Response {
                status: Status::KeyAlreadyExists,
                body: None,
                message: None,
            };
        }

        let insert = dd.insert(&query.0, query.1);

        if insert.is_err() {
            return Response {
                status: Status::Error,
                message: Some(dd_error_code_to_string(insert.err().unwrap().code)),
                body: None,
            };
        }

        Response {
            status: Status::Ok,
            body: None,
            message: None,
        }
    }

    pub fn get(query: String, database: String, cache: TCache, routers: TRouters) -> Response {
        let mut cache = cache.lock().unwrap();

        let cache_key = format!("{}:{}", database, query);

        if cache.contains(cache_key.clone()) {
            let v = cache.get(&cache_key).unwrap().clone();

            return Response {
                status: Status::Ok,
                body: Some(v),
                message: None,
            };
        }

        let mut routers = routers.lock().unwrap();

        if !routers.contains_key(&database) {
            return Response {
                status: Status::DatabaseNotFound,
                body: None,
                message: None,
            };
        }

        let dd = routers.get_mut(&database).unwrap();

        let value = dd.get(&query).unwrap();

        if let Some(value) = value {
            cache.insert(cache_key, value.clone()).unwrap();

            Response {
                status: Status::Ok,
                body: Some(value),
                message: None,
            }
        } else {
            Response {
                status: Status::KeyNotExists,
                body: None,
                message: None,
            }
        }
    }

    pub fn update(
        query: (String, Bson),
        database: String,
        cache: TCache,
        routers: TRouters,
    ) -> Response {
        let mut cache = cache.lock().unwrap();

        if cache.contains(query.0.clone()) {
            cache.remove(&query.0).unwrap();
        }

        let mut routers = routers.lock().unwrap();

        if routers.contains_key(&database) {
            let dd = routers.get_mut(&database).unwrap();

            if !dd.contains(&query.0) {
                return Response {
                    status: Status::KeyNotExists,
                    body: None,
                    message: None,
                };
            }

            let update = dd.update(&query.0, query.1.clone());

            if update.is_err() {
                return Response {
                    message: Some(dd_error_code_to_string(update.err().unwrap().code)),
                    status: Status::Error,
                    body: None,
                };
            }

            cache.insert(query.0, query.1).unwrap();

            Response {
                status: Status::Ok,
                body: None,
                message: None,
            }
        } else {
            Response {
                status: Status::DatabaseNotFound,
                body: None,
                message: None,
            }
        }
    }

    pub fn delete(
        query: (String, bool),
        database: String,
        config: schema::RustbaseConfig,
        cache: TCache,
        routers: TRouters,
    ) -> Response {
        let mut routers = routers.lock().unwrap();

        if routers.contains_key(&database) {
            if !query.1 {
                let dd = routers.get_mut(&database).unwrap();

                if !dd.contains(&query.0) {
                    return Response {
                        status: Status::KeyNotExists,
                        body: None,
                        message: None,
                    };
                }

                let delete = dd.delete(&query.0);

                if delete.is_err() {
                    return Response {
                        message: Some(dd_error_code_to_string(delete.err().unwrap().code)),
                        status: Status::Error,
                        body: None,
                    };
                }

                let mut cache = cache.lock().unwrap();
                let cache_key = format!("{}:{}", database, query.0);

                if cache.contains(cache_key.clone()) {
                    cache.remove(&cache_key).unwrap();
                }

                Response {
                    status: Status::Ok,
                    body: None,
                    message: None,
                }
            } else {
                let database = if query.0.is_empty() {
                    database
                } else {
                    query.0
                };

                let mut dd = routers.remove(&database).unwrap();
                dd.lsm.drop();

                drop(dd);

                route::remove_dustdata(config.database.path, database.clone());

                println!("[Engine] database {} deleted", database);

                Response {
                    status: Status::Ok,
                    body: None,
                    message: None,
                }
            }
        } else {
            Response {
                status: Status::DatabaseNotFound,
                body: None,
                message: None,
            }
        }
    }

    pub fn list(database: String, routers: TRouters) -> Response {
        let mut routers = routers.lock().unwrap();

        if routers.contains_key(&database) {
            let dd = routers.get_mut(&database).unwrap();
            let list = dd.list_keys().unwrap();

            Response {
                status: Status::Ok,
                body: Some(list.into()),
                message: None,
            }
        } else {
            Response {
                status: Status::DatabaseNotFound,
                body: None,
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
    ) -> Response {
        match query {
            Query::Delete(query, is_database) => {
                Worker::delete((query, is_database), database, config, cache, routers)
            }
            Query::Insert(key, value) => Worker::insert((key, value), database, routers, config),
            Query::Get(query) => Worker::get(query, database, cache, routers),
            Query::Update(key, value) => Worker::update((key, value), database, cache, routers),
            Query::List(opt_database) => {
                if let Some(database) = opt_database {
                    Worker::list(database, routers)
                } else {
                    Worker::list(database, routers)
                }
            }
            Query::None => Response {
                message: None,
                status: Status::InvalidQuery,
                body: None,
            },
        }
    }
}

impl Worker {
    pub fn new(
        in_receiver: watch::Receiver<(Query, String)>,
        out_sender: mpsc::Sender<Response>,
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
