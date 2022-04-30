use bson::Document;
use chrono::prelude::*;
use hex::ToHex;

use crate::crypto;

pub fn create_collection(name: String, data: bson::Bson) -> bson::Document {
    let mut collection = bson::Document::new();

    collection = insert_default_values_to_collection(collection, name, data);

    collection
}

pub fn write_collection(collection: bson::Document, data: bson::Bson) -> bson::Document {
    let mut collection = collection;

    if data.as_document().is_some() {
        collection.insert("data", vec![data]);

    } else if data.as_array().is_some() && data.as_array().unwrap().iter().all(|x| x.as_document().is_some()) {
        collection.insert("data", data.as_array().unwrap().clone());

    } else {
        panic!("Data is not a document");
    }

    collection
}

pub fn parse_collection_to_bson(collection: bson::Document) -> Vec<u8> {
    bson::to_vec(&collection).unwrap()
}

pub fn insert_default_values_to_collection(collection: bson::Document, name: String, data: bson::Bson) -> bson::Document {
    let mut collection = collection;
    let id = crypto::generate_bytes::generate_random_bytes();

    collection.insert("created_at".to_string(), Utc::now());
    collection.insert("updated_at".to_string(), Utc::now());
    collection.insert("id".to_string(), id.encode_hex::<String>());
    collection.insert("name".to_string(), name);

    if data.as_document().is_some() {
        collection.insert("data", bson::bson!([data]));

    } else if data.as_array().is_some() && data.as_array().unwrap().iter().all(|x| x.as_document().is_some()) {
        collection.insert("data", data.as_array().unwrap().clone());

    } else {
        panic!("Data is not a document");
    }

    collection
}

pub fn create_document(collection: bson::Document, data: bson::Bson) -> bson::Document {
    let mut collection = collection;

    if data.as_document().is_none() {
        panic!("Data is not a document");
    }
    
    let data_array = collection.get_array("data").unwrap();

    let mut new_data_array = bson::Array::new();

    for data in data_array {
        new_data_array.push(data.clone());
    }
    
    new_data_array.push(data);

    collection.insert("data", new_data_array);

    collection
}

pub fn create_documents(collection: bson::Document, mut data: bson::Bson) -> bson::Document {
    let mut collection = collection;

    if data.as_array().is_none() && data.as_array().unwrap().iter().all(|x| x.as_document().is_none()) {
        panic!("Data is not an array of documents");
    }
    
    let data_array = collection.get_array("data").unwrap();

    let mut new_data_array = bson::Array::new();

    for data in data_array {
        new_data_array.push(data.clone());
    }
    
    new_data_array.append(data.as_array_mut().unwrap());

    collection.insert("data", new_data_array);

    collection
}