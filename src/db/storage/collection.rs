use chrono::prelude::*;
use hex::ToHex;

use crate::crypto;

pub fn create_collection(name: String, data: bson::Bson) -> bson::Document {
    let mut collection = bson::Document::new();

    collection = insert_default_values_to_collection(collection, name, data);

    collection
}

pub fn write_collection(collection: bson::Document, data: bson::Array) -> bson::Document {
    let mut collection = collection;

    for chunk in data.clone().into_iter() {
        if chunk.as_document().is_none() {
            panic!("Data is not a document");
        }
    }

    collection.insert("data", data);

    collection
}

pub fn parse_collection_to_bson(collection: bson::Document) -> Vec<u8> {
    bson::to_vec(&collection).unwrap()
}

fn insert_default_values_to_collection(collection: bson::Document, name: String, data: bson::Bson) -> bson::Document {
    let mut collection = collection;
    let id = crypto::generate_bytes::generate_random_bytes();
    let now = Utc::now();

    collection.insert("created_at", now);
    collection.insert("updated_at", now);
    collection.insert("id", id.encode_hex::<String>());
    collection.insert("name", name);

    if data.as_document().is_some() {
        let data = super::document::prepare_document(data);

        collection.insert("data", vec![data]);
    } else if data.as_array().is_some() && data.as_array().unwrap().iter().all(|x| x.as_document().is_some()) {
        let mut data_array = bson::Array::new();
        
        for document in data.as_array().unwrap() {
            let document = super::document::prepare_document(document.clone());
            data_array.push(document);
        }

        collection.insert("data", data_array);
    } else {
        panic!("Data is not a document or an array of documents");
    }

    collection
}