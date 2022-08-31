pub mod cache;
pub mod main;
use crate::config;

pub async fn initalize_server(config: config::Config) {
    main::initalize_server(config).await;
}
