mod config;
mod server;
mod spec;
mod utils;

use proctitle::set_title;
use std::fs;
use std::io::Write;
use std::path;

extern crate dustdata;

use utils::config as utils_config;

#[tokio::main]
async fn main() {
    set_title("Rustbase Database Server");

    let config = load_config();

    server::initalize_server(config).await;
}

fn load_config() -> config::Config {
    // Default config
    let mut config = config::Config {
        net: config::Net {
            host: "0.0.0.0".to_string(),
            port: "23561".to_string(),
        },
        database: config::Database {
            path: "./data".to_string(),
            cache_size: spec::DEFAULT_CACHE_SIZE,
        },
    };

    // If has rustbase config, load it. Otherwise, use default config (and create a rustbase config).
    let exe = std::env::current_exe().unwrap();
    let mut ancestors = std::path::Path::new(&exe).ancestors();
    let config_path = format!(
        "{}/{}",
        ancestors.nth(1).unwrap().to_str().unwrap(),
        spec::DEFAULT_CONFIG_NAME
    );

    if !path::Path::new(&config_path).exists() {
        println!("Creating config file...");

        let mut file = fs::File::create(config_path).expect("Unable to create file");
        let json_string = serde_json::to_string_pretty(&config).expect("Unable to serialize");

        file.write_all(json_string.as_bytes())
            .expect("Unable to write to file");
    } else {
        config = utils_config::get_config();
    }

    config
}
