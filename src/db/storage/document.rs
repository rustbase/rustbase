use chrono::prelude::*;
use hex::ToHex;

use crate::crypto;

pub fn create_document(name: String, data: bson::Bson) -> bson::Document {
    let mut document = bson::Document::new();

    document = insert_default_values_to_document(document, name, data);

    return document;
}

pub fn write_document(document: bson::Document, data: bson::Bson) -> bson::Document {
    let mut document = document.clone();

    document.insert("data", data);

    return document;
}

pub fn parse_document_to_bson(document: bson::Document) -> Vec<u8> {
    return bson::to_vec(&document).unwrap();
}

pub fn insert_default_values_to_document(document: bson::Document, name: String, data: bson::Bson) -> bson::Document {
    let mut document = document;
    let id = crypto::generate_bytes::generate_random_bytes();

    document.insert("created_at".to_string(), Utc::now());
    document.insert("updated_at".to_string(), Utc::now());
    document.insert("id".to_string(), id.encode_hex::<String>());
    document.insert("name".to_string(), name);

    document.insert("data", data);

    return document;
}