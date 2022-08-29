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
}

#[tonic::async_trait]
impl Rustbase for DatabaseServer {
    async fn get(
        &self,
        request: Request<rustbase::Key>,
    ) -> Result<Response<rustbase::Bson>, Status> {
        let value = self
            .database
            .lock()
            .unwrap()
            .get(&request.get_ref().key)
            .unwrap();

        if value.is_none() {
            return Err(Status::new(
                tonic::Code::NotFound,
                format!("Key not found: {}", request.get_ref().key),
            ));
        }

        let response = rustbase::Bson {
            bson: bson::to_vec(&value.unwrap()).unwrap(),
        };

        Ok(Response::new(response))
    }

    async fn insert(
        &self,
        request: Request<rustbase::KeyValue>,
    ) -> Result<Response<rustbase::Key>, Status> {
        let value: Document = bson::from_slice(&request.get_ref().value).unwrap();
        self.database
            .lock()
            .unwrap()
            .insert(&request.get_ref().key, value)
            .unwrap();

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
        self.database
            .lock()
            .unwrap()
            .update(&request.get_ref().key, value)
            .unwrap();

        let response = rustbase::Key {
            key: request.get_ref().key.clone(),
        };

        Ok(Response::new(response))
    }

    async fn delete(
        &self,
        request: Request<rustbase::Key>,
    ) -> Result<Response<rustbase::Void>, Status> {
        self.database
            .lock()
            .unwrap()
            .delete(&request.get_ref().key)
            .unwrap();

        let response = rustbase::Void {};

        Ok(Response::new(response))
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
    };

    println!("[Server] Listening on rustbase://{}", addr);

    Server::builder()
        .add_service(RustbaseServer::new(database_server))
        .serve(addr)
        .await
        .unwrap();
}
