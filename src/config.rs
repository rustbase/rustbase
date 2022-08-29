use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub net: Net,
    pub database: Database,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Net {
    pub host: String,
    pub port: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    pub path: String,
}
