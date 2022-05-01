use crate::crypto;
use hex::ToHex;

pub fn create_document(collection: bson::Document, data: bson::Bson) -> Vec<bson::Bson> {
    if data.as_document().is_none() {
        panic!("Data is not a document");
    }
    
    let data_array = collection.get_array("data").unwrap();

    let mut new_data_array = bson::Array::new();

    new_data_array.append(&mut data_array.clone());

    let data = prepare_document(data);
    new_data_array.push(data);

    new_data_array
}

pub fn create_documents(collection: bson::Document, data: bson::Bson) -> Vec<bson::Bson> {
    if data.as_array().is_none() && data.as_array().unwrap().iter().all(|x| x.as_document().is_none()) {
        panic!("Data is not an array of documents");
    }
    
    let data_array = collection.get_array("data").unwrap();

    let mut new_data_array = bson::Array::new();

    new_data_array.append(&mut data_array.clone());

    for document in data.as_array().unwrap() {
        let document = prepare_document(document.clone());
        new_data_array.push(document);
    }

    new_data_array
}

pub fn prepare_document(document: bson::Bson) -> bson::Bson {
    let mut document = document;
    let id = crypto::generate_bytes::generate_random_bytes();

    if document.as_document().is_none() {
        panic!("Data is not a document");
    }

    let document = document.as_document_mut().unwrap();
    document.insert("id", id.encode_hex::<String>());

    bson::Bson::Document(document.clone())
}