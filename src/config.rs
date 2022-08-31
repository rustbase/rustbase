use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub net: Net,
    pub database: Database,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Net {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub cache_size: usize,
}
