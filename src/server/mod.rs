pub mod cache;
pub mod main;
use crate::config;

pub async fn initalize_server(config: config::Config) {
    let mut config = config.clone();

    let exe = std::env::current_exe().unwrap();
    let mut ancestors = std::path::Path::new(&exe).ancestors();
    let config_path = format!(
        "{}/{}",
        ancestors.nth(1).unwrap().to_str().unwrap(),
        config.database.path
    );

    config.database.path = config_path;

    main::initalize_server(config).await;
}
