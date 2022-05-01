use crate::config;
use std::io::{Read};
use std::fs;

pub fn get_config() -> config::Config {
    let mut file = fs::File::open("./data/config.json").expect("Unable to open file");
    let mut config_data = String::new();

    file.read_to_string(&mut config_data)
        .expect("Unable to read file");

    serde_json::from_str(&config_data).expect("Unable to deserialize")
}
