pub mod schema;
pub mod spec;

use crate::utils::get_current_path;
use crate::Args;

use colored::Colorize;
use path_absolutize::*;
use std::fs::File;
use std::io::Write;

pub fn default_configuration() -> schema::RustbaseConfig {
    schema::RustbaseConfig {
        threads: num_cpus::get(),
        cache_size: spec::DEFAULT_CACHE_SIZE,
        net: schema::Net {
            host: "0.0.0.0".to_string(),
            port: "23561".to_string(),
            tls: None,
        },
        auth: None,
        storage: schema::Storage {
            path: get_current_path()
                .join("./data")
                .absolutize()
                .unwrap()
                .to_path_buf(),
            dustdata: None,
        },
    }
}

pub fn load_configuration(args: Option<Args>) -> schema::RustbaseConfig {
    let mut config = default_configuration();

    let default_path = get_current_path().join(spec::DEFAULT_CONFIG_NAME);

    if !default_path.exists() {
        let mut file = File::create(default_path.clone()).unwrap();
        file.write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())
            .unwrap();
    }

    let config_path = if let Some(args) = args {
        if let Some(config_path) = args.config {
            get_current_path().join(config_path)
        } else {
            default_path
        }
    } else {
        default_path
    };

    if !config_path.exists() {
        panic!(
            "[Config] {} not found",
            config_path.file_name().unwrap().to_str().unwrap().cyan()
        );
    }

    let file = File::open(config_path).unwrap();

    config = serde_json::from_reader(file).unwrap();

    config
}
