use super::engine::Workers;
use crate::config::schema;
use crate::query;
use crate::server::route;
use dustdata::{DustData, DustDataConfig, LsmConfig, Size};
use query::parser::Query;
use rustbase::rustbase_server::{Rustbase, RustbaseServer};
use rustbase::{QueryMessage, QueryResult, QueryResultType};
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};

pub mod rustbase {
    tonic::include_proto!("rustbase");
}

pub struct Database {
    workers: Arc<tokio::sync::Mutex<Workers>>,
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

        let response =
            Workers::process(self.workers.clone(), database.clone(), query.clone()).await;

        Ok(Response::new(response))
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

    let database_server = Database {
        workers: Workers::new(routers, cache, config).await,
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
