use crate::config;
use bson::Document;
use rustbase::rustbase_server::{Rustbase, RustbaseServer};
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};

pub mod rustbase {
    tonic::include_proto!("rustbase");
}

pub struct DatabaseServer {
    database: Arc<Mutex<dustdata::DustData>>,
    cache: Arc<Mutex<super::cache::Cache>>,
}

#[tonic::async_trait]
impl Rustbase for DatabaseServer {
    async fn get(
        &self,
        request: Request<rustbase::Key>,
    ) -> Result<Response<rustbase::Bson>, Status> {
        let mut cache = self.cache.lock().unwrap();
        let key = &request.get_ref().key;

        if cache.contains(key.to_string()) {
            let value = cache.get(key).unwrap();
            let response = rustbase::Bson {
                bson: bson::to_vec(&Some(value.clone())).unwrap(),
            };
            return Ok(Response::new(response));
        } else {
            let dd = self.database.lock().unwrap();
            let value = dd.get(&request.get_ref().key).unwrap();

            if value.is_none() {
                return Err(Status::new(
                    tonic::Code::NotFound,
                    format!("Key not found: {}", request.get_ref().key),
                ));
            }

            let response = rustbase::Bson {
                bson: bson::to_vec(&value.clone().unwrap()).unwrap(),
            };

            let cache_insert = cache.insert(key.to_string(), value.unwrap());
            cache_insert.ok();

            Ok(Response::new(response))
        }
    }

    async fn insert(
        &self,
        request: Request<rustbase::KeyValue>,
    ) -> Result<Response<rustbase::Key>, Status> {
        let value: Document = bson::from_slice(&request.get_ref().value).unwrap();
        let key = &request.get_ref().key;
        let mut dd = self.database.lock().unwrap();

        if dd.contains(key) {
            return Err(Status::new(
                tonic::Code::AlreadyExists,
                format!("Key already exists: {}", key),
            ));
        }

        dd.insert(&request.get_ref().key, value).unwrap();

        let response = rustbase::Key {
            key: request.get_ref().key.clone(),
        };

        Ok(Response::new(response))
    }

    async fn update(
        &self,
        request: Request<rustbase::KeyValue>,
    ) -> Result<Response<rustbase::Key>, Status> {
        let value: Document = bson::from_slice(&request.get_ref().value).unwrap();
        let key = &request.get_ref().key;
        let mut dd = self.database.lock().unwrap();

        if !dd.contains(key) {
            return Err(Status::new(
                tonic::Code::NotFound,
                format!("Key not found: {}", request.get_ref().key),
            ));
        }

        dd.update(&request.get_ref().key, value).unwrap();

        let response = rustbase::Key {
            key: request.get_ref().key.clone(),
        };

        Ok(Response::new(response))
    }

    async fn delete(
        &self,
        request: Request<rustbase::Key>,
    ) -> Result<Response<rustbase::Void>, Status> {
        let key = &request.get_ref().key;
        let mut dd = self.database.lock().unwrap();

        if !dd.contains(key) {
            return Err(Status::new(
                tonic::Code::NotFound,
                format!("Key not found: {}", request.get_ref().key),
            ));
        }

        let mut cache = self.cache.lock().unwrap();

        if cache.contains(key.to_string()) {
            cache.remove(key).ok();
        }

        dd.delete(key).unwrap();

        Ok(Response::new(rustbase::Void {}))
    }
}

pub async fn initalize_server(config: config::Config) {
    let addr = format!("{}:{}", config.net.host, config.net.port)
        .parse()
        .unwrap();

    let database_server = DatabaseServer {
        database: Arc::new(Mutex::new(dustdata::initialize(dustdata::DustDataConfig {
            path: config.database.path.clone(),
            lsm_config: dustdata::LsmConfig {
                flush_threshold: dustdata::Size::Megabytes(128),
            },
        }))),
        cache: Arc::new(Mutex::new(super::cache::Cache::new(
            config.database.cache_size,
        ))),
    };

    println!("[Server] Listening on rustbase://{}", addr);

    Server::builder()
        .add_service(RustbaseServer::new(database_server))
        .serve(addr)
        .await
        .unwrap();
}
