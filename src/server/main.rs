use super::wirewave::server::{Request, Response, Server, Status, Wirewave, WirewaveServer};
use crate::config::schema;
use crate::query;
use crate::server::engine::worker_manager::WorkerManager;
use crate::server::route;
use async_trait::async_trait;
use dustdata::{DustData, DustDataConfig, LsmConfig, Size};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TMutex;

pub struct Database {
    worker_manager: Arc<TMutex<WorkerManager>>,
}

#[async_trait]
impl Wirewave for Database {
    async fn request(&self, request: Request) -> Result<Response, Status> {
        let body = request.body;

        let body = match body.as_document() {
            None => return Err(Status::InvalidBody),
            Some(body) => {
                if body.is_empty() {
                    return Err(Status::InvalidBody);
                }

                if !body.contains_key("query") || !body.contains_key("database") {
                    return Err(Status::InvalidBody);
                }

                body
            }
        };

        let database = body.get_str("database").unwrap();
        let query = body.get_str("query").unwrap();

        if let Err(error) = query::parser::parse(query.to_string()) {
            return Ok(Response {
                message: Some(error.1),
                status: Status::SyntaxError,
                body: None,
            });
        }

        let query = query::parser::parse(query.to_string()).unwrap();

        let result = self
            .worker_manager
            .lock()
            .await
            .process(query, database.to_string())
            .await;

        Ok(result)
    }
}

pub async fn initalize_server(config: schema::RustbaseConfig) {
    let addr = format!("{}:{}", config.net.host, config.net.port);

    let routers = route::initialize_dustdata(config.clone().database.path);
    let cache = Arc::new(Mutex::new(super::cache::Cache::new(
        config.database.cache_size,
    )));

    let manager = WorkerManager::new(routers, cache, config.clone(), config.database.threads).await;

    let database = Database {
        worker_manager: Arc::new(TMutex::new(manager)),
    };

    let svc = WirewaveServer::new(database);

    println!("[Server] Listening on rustbase://{}", addr);

    Server::new(svc).serve(addr).await.unwrap();
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
