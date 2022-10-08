pub mod cache;
pub mod engine;
pub mod main;
pub mod route;

use crate::config::schema;

pub async fn initalize_server(config: schema::RustbaseConfig) {
    main::initalize_server(config).await;
}
