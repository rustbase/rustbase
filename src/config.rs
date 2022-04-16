use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub net: Net,
    pub database: Database,
}

#[derive(Serialize, Deserialize)]
pub struct Net {
    pub host: String,
    pub port: String,
}

#[derive(Serialize, Deserialize)]
pub struct Database {
    pub path: String,
    pub log_path: String,
}