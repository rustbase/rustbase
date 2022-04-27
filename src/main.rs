mod config;
mod db;
mod utils;
mod crypto;

use proctitle::set_title;
use std::fs;
use std::io::Write;
use std::path;

use utils::{config as utils_config};

fn get_exec_name() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
        .and_then(|s| s.into_string().ok())
}

#[tokio::main]
async fn main() {
    set_title("Rustbase Database");

    let config = load_config();

    // Database instance
    let database = db::initalize_database(config.database);

    // To test the database use:
    // let data = bson::bson!({
    //     "some_test_key": "some_test_value"
    // });

    // database.create_collection("test_create_collection".to_string(), data).unwrap();
    // let collection = database.get_collection("test_create_collection".to_string()).unwrap();
}

fn load_config() -> config::Config {
    if !path::Path::new("data").exists() {
        fs::create_dir_all(&path::Path::new("data")).expect("Failed to create database path");
    }

    // Default config
    let mut config = config::Config {
        net: config::Net {
            host: "127.0.0.1".to_string(),
            port: "23561".to_string(),
        },
        database: config::Database {
            path: "./data".to_string(),
            log_path: "./logs".to_string(),
        },
    };

    // If has rustbase config, load it. Otherwise, use default config (and create a rustbase config).
    if !path::Path::new("./data/config.json").exists() {
        println!("Creating config file...");

        let mut file = fs::File::create("./data/config.json").expect("Unable to create file");
        let json_string = serde_json::to_string_pretty(&config).expect("Unable to serialize");

        file.write(json_string.as_bytes())
            .expect("Unable to write to file");
    } else {
        config = utils_config::get_config();
    }

    return config;
}