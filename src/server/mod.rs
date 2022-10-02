pub mod cache;
pub mod main;
use crate::config::schema;

pub async fn initalize_server(config: schema::RustbaseConfig) {
    main::initalize_server(config).await;
}
