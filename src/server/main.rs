use crate::config;
use bson::Document;
use dustdata::{DustData, DustDataConfig, LsmConfig, Size};
use rustbase::rustbase_server::{Rustbase, RustbaseServer};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};

pub mod rustbase {
    tonic::include_proto!("rustbase");
}

pub struct DatabaseServer {
    routers: Arc<Mutex<BTreeMap<String, DustData>>>,
    // database: Arc<Mutex<dustdata::DustData>>,
    cache: Arc<Mutex<super::cache::Cache>>,
    config: config::Config,
}
#[tonic::async_trait]
impl Rustbase for DatabaseServer {
    async fn create_database(
        &self,
        request: Request<rustbase::Database>,
    ) -> Result<Response<rustbase::Void>, Status> {
        let database_name = request.into_inner().name;

        let mut routers = self.routers.lock().unwrap();

        if routers.contains_key(&database_name) {
            return Err(Status::already_exists("Database already exists"));
        }

        let config = default_dustdata_config(self.config.database.path.clone());

        let dd = dustdata::initialize(config);

        routers.insert(database_name, dd);

        Ok(Response::new(rustbase::Void {}))
    }

    async fn delete_database(
        &self,
        request: Request<rustbase::Database>,
    ) -> Result<Response<rustbase::Void>, Status> {
        let database_name = request.into_inner().name;
        let mut routers = self.routers.lock().unwrap();

        if !routers.contains_key(&database_name) {
            return Err(Status::not_found("Database not found"));
        }

        routers.remove(&database_name);

        Ok(Response::new(rustbase::Void {}))
    }

    async fn get(
        &self,
        request: Request<rustbase::Key>,
    ) -> Result<Response<rustbase::Bson>, Status> {
        let key = &request.get_ref().key;
        let database_name = &request.get_ref().database;

        let routers = self.routers.lock().unwrap();
        let mut cache = self.cache.lock().unwrap();

        if !routers.contains_key(database_name) {
            return Err(Status::not_found("Database not found"));
        }

        let dd = routers.get(database_name).unwrap();

        if cache.contains(key.to_string()) {
            let value = cache.get(key).unwrap();
            let response = rustbase::Bson {
                bson: bson::to_vec(&Some(value.clone())).unwrap(),
            };
            return Ok(Response::new(response));
        } else {
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
    ) -> Result<Response<rustbase::Void>, Status> {
        let value: Document = bson::from_slice(&request.get_ref().value).unwrap();
        let key = &request.get_ref().key;
        let database_name = &request.get_ref().database;

        let mut routers = self.routers.lock().unwrap();

        if !routers.contains_key(database_name) {
            return Err(Status::not_found("Database not found"));
        }

        let dd = routers.get_mut(database_name).unwrap();

        if dd.contains(key) {
            return Err(Status::new(
                tonic::Code::AlreadyExists,
                format!("Key already exists: {}", key),
            ));
        }

        dd.insert(&request.get_ref().key, value).unwrap();

        let response = rustbase::Void {};

        Ok(Response::new(response))
    }

    async fn update(
        &self,
        request: Request<rustbase::KeyValue>,
    ) -> Result<Response<rustbase::Void>, Status> {
        let value: Document = bson::from_slice(&request.get_ref().value).unwrap();
        let key = &request.get_ref().key;
        let database_name = &request.get_ref().database;

        let mut routers = self.routers.lock().unwrap();

        if !routers.contains_key(database_name) {
            return Err(Status::not_found("Database not found"));
        }

        let dd = routers.get_mut(database_name).unwrap();

        if !dd.contains(key) {
            return Err(Status::new(
                tonic::Code::NotFound,
                format!("Key not found: {}", request.get_ref().key),
            ));
        }

        dd.update(&request.get_ref().key, value).unwrap();

        let response = rustbase::Void {};

        Ok(Response::new(response))
    }

    async fn delete(
        &self,
        request: Request<rustbase::Key>,
    ) -> Result<Response<rustbase::Void>, Status> {
        let key = &request.get_ref().key;
        let database_name = &request.get_ref().database;

        let mut routers = self.routers.lock().unwrap();

        if !routers.contains_key(database_name) {
            return Err(Status::not_found("Database not found"));
        }

        let dd = routers.get_mut(database_name).unwrap();

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

    let mut routers = BTreeMap::new();

    for route in get_existing_routes(config.database.path.clone()) {
        let dd = dustdata::initialize(default_dustdata_config(config.database.path.clone()));

        routers.insert(route, dd);
    }

    let database_server = DatabaseServer {
        cache: Arc::new(Mutex::new(super::cache::Cache::new(
            config.database.cache_size,
        ))),
        routers: Arc::new(Mutex::new(routers)),
        config,
    };

    println!("[Server] Listening on rustbase://{}", addr);

    Server::builder()
        .add_service(RustbaseServer::new(database_server))
        .serve(addr)
        .await
        .unwrap();
}

fn default_dustdata_config(data_path: String) -> DustDataConfig {
    DustDataConfig {
        path: data_path,
        lsm_config: LsmConfig {
            flush_threshold: Size::Megabytes(128),
        },
    }
}

fn get_existing_routes(data_path: String) -> Vec<String> {
    let mut routes = Vec::new();

    for entry in std::fs::read_dir(data_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let route = path.file_name().unwrap().to_str().unwrap().to_string();
        println!("Found existing route: {}", route);
        routes.push(route);
    }

    routes
}
