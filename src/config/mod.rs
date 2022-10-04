pub mod schema;
pub mod spec;

use crate::utils::get_current_path;
use colored::Colorize;
use path_absolutize::*;
use std::env::var;
use std::fs::File;
use std::io::Write;

pub fn default_configuration() -> schema::RustbaseConfig {
    schema::RustbaseConfig {
        net: schema::Net {
            host: "0.0.0.0".to_string(),
            port: "23561".to_string(),
        },
        database: schema::Database {
            path: "./data".to_string(),
            cache_size: spec::DEFAULT_CACHE_SIZE,
        },
    }
}

pub fn load_configuration() -> schema::RustbaseConfig {
    let mut config = default_configuration();

    let config_path = if let Ok(config_file_name) = var("RUSTBASE_CONFIG_FILE") {
        get_current_path().join(config_file_name)
    } else {
        get_current_path().join(spec::DEFAULT_CONFIG_NAME)
    };

    if !config_path.exists() {
        println!(
            "[Config] {} not found",
            config_path.file_name().unwrap().to_str().unwrap().cyan()
        );

        if var("RUSTBASE_CONFIG_FILE").is_err() {
            // env var not set
            println!(
                "[Config] {} creating a new file",
                spec::DEFAULT_CONFIG_NAME.cyan()
            );
            File::create(config_path)
                .unwrap()
                .write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())
                .unwrap();

            println!("[Config] {} created", spec::DEFAULT_CONFIG_NAME.cyan());
        } else {
            panic!("{} not found", config_path.absolutize().unwrap().display());
        }

        return config;
    }

    let file = File::open(config_path.clone()).unwrap();

    config = match serde_json::from_reader(file) {
        Ok(config) => config,
        Err(e) => {
            if e.is_data() {
                println!("[Config] {}", e.to_string().red());
                println!("[Config] Check the config file for errors.");
                println!("[Config] Using default configuration");

                return config;
            } else {
                println!(
                    "[Config] Error parsing config file, using default configuration: {}",
                    e
                );

                return config;
            }
        }
    };

    config.database.path = get_current_path()
        .join(&config.database.path)
        .absolutize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    println!(
        "[Config] {} loaded",
        config_path.file_name().unwrap().to_str().unwrap().cyan()
    );
    println!("[Config] load config: {}", config);

    config
}
