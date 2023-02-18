use async_trait::async_trait;
use colored::Colorize;
use dustdata::{DustData, DustDataConfig, LsmConfig, Size};
use rayon::{ThreadPool, ThreadPoolBuilder};

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

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
    routers: Arc<RwLock<HashMap<String, DustData>>>,
    config: Arc<schema::RustbaseConfig>,
    cache: Arc<RwLock<Cache>>,
    system_db: Arc<RwLock<dustdata::DustData>>,
}

#[async_trait]
impl Wirewave for Database {
    async fn request(
        &self,
        request: Request,
        username: Option<String>,
    ) -> Result<Response, Status> {
        let body = request.body;

        if body.is_empty() {
            return Err(Status::InvalidBody);
        }

        if !body.contains_key("query") || !body.contains_key("database") {
            return Err(Status::InvalidBody);
        }

        let database = body.get_str("database").unwrap();
        let query = body.get_str("query").unwrap();

        self.pool
            .install(move || match query::parser::parse(query) {
                Err(e) => Ok(Response {
                    message: Some(e.1),
                    status: Status::SyntaxError,
                    body: None,
                }),

                Ok(query) => {
                    let mut core = Core::new(
                        self.cache.clone(),
                        self.routers.clone(),
                        self.config.clone(),
                        self.system_db.clone(),
                        database.to_string(),
                        username,
                    );

                    core.run_ast(query[0].clone())
                }
            })
    }
}

fn current_users(system_db: Arc<RwLock<DustData>>) -> usize {
    let dd = system_db.read().unwrap();

    dd.list_keys().unwrap().len()
}

pub async fn initalize_server(config: schema::RustbaseConfig) {
    let config = Arc::new(config);
    let addr = format!("{}:{}", config.net.host, config.net.port);

    let path = Path::new(&config.database.path);

    let routers = route::initialize_dustdata(path);
    let cache = Arc::new(RwLock::new(Cache::new(config.database.cache_size)));

    let system_db = Arc::new(RwLock::new(DustData::new(default_dustdata_config(
        &path.join("_default"),
    ))));

    let c_routers = routers.clone();
    let c_system_db = system_db.clone();
    ctrlc::set_handler(move || {
        c_routers
            .write()
            .unwrap()
            .iter_mut()
            .for_each(|(route, dd)| {
                println!("[Server] flushing {} to exit", route.yellow());
                dd.flush().unwrap();
            });

        c_system_db.write().unwrap().flush().unwrap();

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
        cache,
        config: Arc::clone(&config),
        system_db: Arc::clone(&system_db),
    };

    let users = current_users(Arc::clone(&system_db));

    let use_auth = users > 0;

    let svc = WirewaveServer::new(database);

    println!(
        "[Server] listening on {}",
        format!("rustbase://{}", addr).yellow()
    );

    let server = Server::new(svc, system_db, use_auth);

    if let Some(tls) = &config.tls {
        server.serve_tls(addr, tls).await;
    } else {
        server.serve(addr).await;
    }
}

pub fn default_dustdata_config(data_path: &Path) -> DustDataConfig {
    DustDataConfig {
        path: data_path.to_path_buf(),
        lsm_config: LsmConfig {
            flush_threshold: Size::Megabytes(256),
        },
    }
}
