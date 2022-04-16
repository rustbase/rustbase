mod config;
mod db;
mod utils;

use proctitle::set_title;
use std::convert::TryFrom;
use std::fs;
use std::io::Write;
use std::path;
use std::process;
use std::{thread, time};
use sysinfo::{Pid, ProcessExt, System, SystemExt};

use crate::db::storage::types::{Data, Types};

use utils::{config as utils_config};

fn get_exec_name() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
        .and_then(|s| s.into_string().ok())
}

#[tokio::main]
async fn main() {
    set_title("Rustbase Database Server");

    // Check if has another instance running
    let s = System::new_all();
    for process in s.processes_by_exact_name(get_exec_name().unwrap().as_str()) {
        let pid = process.pid();
        if pid != Pid::from(usize::try_from(process::id()).unwrap()) {
            println!("Another instance of the server is already running. Exiting.");

            exit(1);
        }
    }

    let config = load_config();

    // Database instance
    let database = db::initalize_database(config.database);

    // To test the database use:
    // let test = vec![Data {
    //     key: "Hello".to_string(),
    //     value: Box::new("World".to_string()),
    //     type_: Types::Integer,
    // }];

    // database.create_document("my_first_document".to_string(), test);
}

// Wait 1s and then exit
fn exit(code: i32) {
    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);

    std::process::exit(code);
}

fn load_config() -> config::Config {
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

    if !path::Path::new("./rustbase.json").exists() {
        println!("Creating config file...");

        let mut file = fs::File::create("./rustbase.json").expect("Unable to create file");
        let json_string = serde_json::to_string_pretty(&config).expect("Unable to serialize");

        file.write(json_string.as_bytes())
            .expect("Unable to write to file");
    } else {
        config = utils_config::get_config();
    }

    return config;
}