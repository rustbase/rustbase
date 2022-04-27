use chrono::prelude::*;
use hex::ToHex;

use crate::crypto;

pub fn create_collection(name: String, data: bson::Bson) -> bson::Document {
    let mut collection = bson::Document::new();

    collection = insert_default_values_to_collection(collection, name, data);

    return collection;
}

pub fn write_collection(collection: bson::Document, data: bson::Bson) -> bson::Document {
    let mut collection = collection.clone();

    collection.insert("data", data);

    return collection;
}

pub fn parse_collection_to_bson(collection: bson::Document) -> Vec<u8> {
    return bson::to_vec(&collection).unwrap();
}

pub fn insert_default_values_to_collection(collection: bson::Document, name: String, data: bson::Bson) -> bson::Document {
    let mut collection = collection;
    let id = crypto::generate_bytes::generate_random_bytes();

    collection.insert("created_at".to_string(), Utc::now());
    collection.insert("updated_at".to_string(), Utc::now());
    collection.insert("id".to_string(), id.encode_hex::<String>());
    collection.insert("name".to_string(), name);

    collection.insert("data", data);

    return collection;
}