use async_trait::async_trait;
use colored::Colorize;
use dustdata::{DustData, DustDataConfig, LsmConfig, Size};
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::cache;
use super::engine;
use super::wirewave;
use crate::config;
use crate::query;
use crate::server;

use cache::Cache;
use config::schema;
use engine::core::Core;
use server::route;
use wirewave::server::{Request, Response, Server, Status, Wirewave, WirewaveServer};

pub struct Database {
    pool: ThreadPool,
    routers: Arc<Mutex<HashMap<String, DustData>>>,
    config: Arc<schema::RustbaseConfig>,
    cache: Arc<Mutex<Cache>>,
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

        let response = self
            .pool
            .install(move || match query::parser::parse(query) {
                Err(e) => {
                    return Ok(Response {
                        message: Some(e.1),
                        status: Status::SyntaxError,
                        body: None,
                    });
                }

                Ok(query) => {
                    let mut core = Core::new(
                        self.cache.clone(),
                        self.routers.clone(),
                        self.config.clone(),
                        database.to_string(),
                    );

                    core.run(query[0].clone())
                }
            });

        response
    }
}

pub async fn initalize_server(config: schema::RustbaseConfig) {
    let config = Arc::new(config);
    let addr = format!("{}:{}", config.net.host, config.net.port);

    let routers = route::initialize_dustdata(Arc::clone(&config).database.path.clone());
    let cache = Arc::new(Mutex::new(Cache::new(config.database.cache_size)));

    let c_routers = routers.clone();
    ctrlc::set_handler(move || {
        c_routers
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|(route, dd)| {
                println!("[Server] flushing {} to exit", route.yellow());
                dd.flush().unwrap();
            });
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let pool = ThreadPoolBuilder::new()
        .num_threads(config.database.threads)
        .build()
        .unwrap();

    let database = Database {
        pool,
        routers,
        config: Arc::clone(&config),
        cache,
    };

    let svc = WirewaveServer::new(database);

    println!(
        "[Server] listening on {}",
        format!("rustbase://{}", addr).yellow()
    );

    if let Some(tls) = &config.tls {
        Server::new(svc).serve_tls(addr, tls).await;
    } else {
        Server::new(svc).serve(addr).await;
    }
}

pub fn default_dustdata_config(data_path: String) -> DustDataConfig {
    DustDataConfig {
        path: data_path,
        verbose: true,
        lsm_config: LsmConfig {
            detect_exit_signals: false,
            flush_threshold: Size::Megabytes(256),
        },
    }
}
