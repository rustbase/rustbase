use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RustbaseConfig {
    pub threads: usize,
    pub cache_size: usize,
    pub net: Net,
    pub storage: Storage,
    pub auth: Option<Auth>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Net {
    pub host: String,
    pub port: String,
    pub tls: Option<Tls>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tls {
    pub ca_file: String,
    pub pem_key_file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Storage {
    pub path: std::path::PathBuf,
    pub dustdata: Option<DustDataStorageConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DustDataStorageConfig {
    pub flush_threshold: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Auth {
    pub enable_auth_bypass: Option<bool>,
    pub auth_type: Option<AuthType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AuthType {
    #[serde(rename = "scram-sha-256")]
    ScramSha256,
}
