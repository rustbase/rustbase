use serde::{Deserialize, Serialize};
use crate::crypto::hash;

// Current only support string values
#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub key: String,
    pub value: Types,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Types {
    String(String),
    Float(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<Types>),
    Date(chrono::DateTime<chrono::Utc>),
    Map(),
    // Advanced types
    Hash(Vec<u8>),
}

impl Data {
    pub fn new(key: String, value: Types) -> Data {
        if let Types::Hash(hash) = value {
            let hash = hash::hash_content(hash);

            return Data {
                key,
                value: Types::Hash(hash),
            }
        };

        Data {
            key,
            value,
        }
    }
}
