use chrono::{prelude::*, serde::ts_seconds};
use hex::ToHex;
use serde::{Deserialize, Serialize};

use crate::crypto;
use super::types::Data;

#[derive(Serialize, Deserialize, Debug)]
pub struct Document {
    pub id: String,
    pub content: Vec<Data>,
    pub name: String,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn create(name: String, content: Vec<Data>) -> Self {
        let id = crypto::generate_bytes::generate_random_bytes();

        Self {
            id: id.encode_hex::<String>(),
            name,
            content: content,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

pub fn create_document(name: String, data: Vec<Data>) -> Vec<u8> {
    let document = Document::create(name, data);

    let json = serde_json::to_string(&document).unwrap();

    return json.as_bytes().to_vec();
}