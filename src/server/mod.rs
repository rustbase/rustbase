pub mod cache;
pub mod main;
use crate::config;

pub async fn initalize_server(config: config::Config) {
    let mut config = config.clone();
    config.database.path = std::path::Path::new(&std::env::current_dir().unwrap())
        .join(config.database.path)
        .to_str()
        .unwrap()
        .to_string();

    main::initalize_server(config).await;
}
