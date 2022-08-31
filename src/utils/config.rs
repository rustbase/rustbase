use crate::config;
use std::fs;
use std::io::Read;

pub fn get_config() -> config::Config {
    let config_path = format!(
        "{:?}/{}",
        std::env::current_dir().unwrap(),
        crate::spec::DEFAULT_CONFIG_NAME
    );
    let mut file = fs::File::open(config_path).expect("Unable to open file");
    let mut config_data = String::new();

    file.read_to_string(&mut config_data)
        .expect("Unable to read file");

    serde_json::from_str(&config_data).expect("Unable to deserialize")
}
