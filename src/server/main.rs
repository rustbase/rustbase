use super::engine::DatabaseEngine;
use crate::config::schema;
use crate::query;
use crate::server::route;
use dustdata::{DustData, DustDataConfig, LsmConfig, Size};
use query::parser::Query;
use rustbase::rustbase_server::{Rustbase, RustbaseServer};
use rustbase::{QueryMessage, QueryResult, QueryResultType};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TMutex;
use tonic::{transport::Server, Request, Response, Status};

pub mod rustbase {
    tonic::include_proto!("rustbase");
}

pub struct Database {
    pub engine: Arc<TMutex<DatabaseEngine>>,
    pub in_sender: tokio::sync::watch::Sender<(Query, String)>,
    pub out_receiver: TMutex<tokio::sync::mpsc::Receiver<QueryResult>>,
}

#[tonic::async_trait]
impl Rustbase for Database {
    async fn query(&self, request: Request<QueryMessage>) -> Result<Response<QueryResult>, Status> {
        let request = request.into_inner();

        let message = request.query;
        let database = request.database;

        if let Err(error) = query::parser::parse(message.clone()) {
            return Ok(Response::new(QueryResult {
                message: None,
                error_message: Some(error.1),
                result_type: QueryResultType::SyntaxError as i32,
                bson: None,
            }));
        }

        let query = query::parser::parse(message).unwrap();

        self.in_sender.send((query, database)).unwrap();
        let result = self.out_receiver.lock().await.recv().await.unwrap();

        Ok(Response::new(result))
    }
}

pub async fn initalize_server(config: schema::RustbaseConfig) {
    let addr = format!("{}:{}", config.net.host, config.net.port)
        .parse()
        .unwrap();

    let routers = route::initialize_dustdata(config.clone().database.path);
    let cache = Arc::new(Mutex::new(super::cache::Cache::new(
        config.database.cache_size,
    )));

    let engine = DatabaseEngine::new(routers, cache, config.clone()).await;

    let am_engine = Arc::new(TMutex::new(engine.0));

    DatabaseEngine::run(am_engine.clone(), config.database.threads).await;

    let database_server = Database {
        engine: am_engine,
        in_sender: engine.1,
        out_receiver: TMutex::new(engine.2),
    };

    println!("[Server] Listening on rustbase://{}", addr);

    Server::builder()
        .add_service(RustbaseServer::new(database_server))
        .serve(addr)
        .await
        .unwrap();
}

pub fn default_dustdata_config(data_path: String) -> DustDataConfig {
    DustDataConfig {
        path: data_path,
        verbose: true,
        lsm_config: LsmConfig {
            flush_threshold: Size::Megabytes(256),
        },
    }
}

pub fn create_dustdata(path: String) -> DustData {
    dustdata::initialize(default_dustdata_config(path))
}
