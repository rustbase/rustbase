use super::storage;

use std::fs;
use std::path;
pub struct Database {
    database_path: String,
}

impl Database {
    pub fn new(path: String) -> Self {
        println!("Initialing database, data path: {}", path);

        let db_path = path::Path::new(&path);

        if !db_path.exists() {
            fs::create_dir_all(&path).expect("Failed to create database path");
        }

        Self {
            database_path: path
        }
    }

    pub fn create_collection(&self, collection_name: String, data: bson::Bson) -> Result<bson::Array, &'static str> {
        if self.check_collection_exists(collection_name.clone()) {
            return Err("Collection already exists");
        }

        if !self.check_if_is_document_or_array_of_documents(data.clone()) {
            return Err("Data is not a document");
        }

        let collection = storage::collection::create_collection(collection_name, data);
        storage::write_collection_to_database(collection.clone(), self.database_path.clone());

        let documents = collection.get_array("data").unwrap();

        Ok(documents.clone())
    }

    pub fn get_collection(&self, collection_name: String) -> Result<bson::Document, &'static str> {
        if !self.check_collection_exists(collection_name.clone()) {
            return Err("Collection do not already exists");
        }

        Ok(storage::get_collection_from_database(collection_name, self.database_path.clone()))
    }

    pub fn write_collection(&self, collection_name: String, data: bson::Bson) -> Result<bson::Array, &'static str> {
        if !self.check_collection_exists(collection_name.clone()) {
            return Err("Collection do not already exists");
        }

        if !self.check_if_is_document_or_array_of_documents(data.clone()) {
            return Err("Data is not a document");
        }

        let collection = storage::read::collection(collection_name, self.database_path.clone()).unwrap();

        let parsed_documents = storage::document::create_document(collection.clone(), data);

        let new_collection = storage::collection::write_collection(collection, parsed_documents.clone());

        storage::write_collection_to_database(new_collection, self.database_path.clone());
    
        Ok(parsed_documents)
    }

    pub fn create_document(&self, collection_name: String, data: bson::Bson) -> Result<bson::Array, &'static str> {
        if !self.check_collection_exists(collection_name.clone()) {
            return Err("Collection do not already exists");
        }

        let collection = storage::read::collection(collection_name, self.database_path.clone()).unwrap();

        let parsed_documents = storage::document::create_document(collection.clone(), data);

        let new_collection = storage::collection::write_collection(collection, parsed_documents.clone());

        storage::write_collection_to_database(new_collection, self.database_path.clone());

        Ok(parsed_documents)
    }

    pub fn create_documents(&self, collection_name: String, data: bson::Bson) -> Result<bson::Array, &'static str> {
        if !self.check_collection_exists(collection_name.clone()) {
            return Err("Collection do not already exists");
        }

        let collection = storage::read::collection(collection_name, self.database_path.clone()).unwrap();

        let parsed_documents = storage::document::create_documents(collection.clone(), data);

        let new_collection = storage::collection::write_collection(collection, parsed_documents.clone());

        storage::write_collection_to_database(new_collection, self.database_path.clone());

        Ok(parsed_documents)
    }

    pub fn get_document(&self, collection_name: String, document_id: String) -> Result<bson::Document, &'static str> {
        if !self.check_collection_exists(collection_name.clone()) {
            return Err("Collection do not already exists");
        }

        let collection = storage::read::collection(collection_name, self.database_path.clone()).unwrap();

        let document = storage::document::get_document(collection, document_id);

        Ok(document)
    }
    
    pub fn list_documents() {
        unimplemented!();
    }

    fn check_collection_exists(&self, name: String) -> bool {
        return path::Path::new(&self.database_path).join(name + "_0").exists()
    }

    fn check_if_is_document_or_array_of_documents(&self, data: bson::Bson) -> bool {
        return data.as_document().is_some() || (data.as_array().is_some() && data.as_array().unwrap().iter().all(|x| x.as_document().is_some()));
    }
}

// Unit Tests
#[cfg(test)]
mod database_test {
    use super::*;
    use crate::db;
    use std::fs;
    use std::path;

    const TEST_DB_PATH: &str = "./data/test_db";

    #[test]
    fn create_database() {
        let database_path = path::Path::new(TEST_DB_PATH);

        if database_path.exists() {
            fs::remove_dir_all(database_path).expect("Failed to remove database path");
        }

        let _database = Database::new(database_path.to_str().unwrap().to_string());

        assert!(database_path.exists());
    }

    #[test]
    fn create_collection() {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let collection_name = "test_create_collection".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });

        database
            .create_collection(collection_name.clone(), data)
            .unwrap();

        assert!(database_path.join(collection_name + "_0").exists());
    }

    #[test]
    fn get_collection()  {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let collection_name = "test_get_collection".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });

        database
            .create_collection(collection_name.clone(), data)
            .unwrap();

        let collection = database.get_collection(collection_name.clone()).unwrap();

        assert_eq!(collection.get_str("name").unwrap(), collection_name);
    }

    #[test]
    fn write_collection() {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let collection_name = "test_write_collection".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });

        database
            .create_collection(collection_name.clone(), data)
            .unwrap();

        let collection = database.get_collection(collection_name.clone()).unwrap();

        let new_data = bson::bson!({
            "new_data": "new_data"
        });

        database.write_collection(collection_name.clone(), new_data).unwrap();

        let new_collection = database.get_collection(collection_name).unwrap();

        let collection_data = new_collection.get_array("data").unwrap();

        assert!(collection_data[1].as_document().unwrap().get_str("new_data").is_ok());
    }

    #[test]
    fn create_document() {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let collection_name = "test_create_document".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });


        database
            .create_collection(collection_name.clone(), data)
            .unwrap();
        
        let new_data = bson::bson!({
            "new_data": "new_data"
        });

        database.create_document(collection_name.clone(), new_data).unwrap();

        let collection = database.get_collection(collection_name).unwrap();

        let collection_data = collection.get_array("data").unwrap();

        assert!(collection_data[1].as_document().unwrap().get_str("new_data").is_ok());
    }

    #[test]
    fn create_documents() {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let collection_name = "test_create_documents".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });


        database
            .create_collection(collection_name.clone(), data)
            .unwrap();
        
        let new_data = bson::bson!([{
            "new_data_1": "new_data_1"
        }, {
            "new_data_2": "new_data_2"
        }]);

        database.create_documents(collection_name.clone(), new_data).unwrap();

        let collection = database.get_collection(collection_name).unwrap();

        let collection_data = collection.get_array("data").unwrap();

        assert!(collection_data[1].as_document().unwrap().get_str("new_data_1").is_ok());
        assert!(collection_data[2].as_document().unwrap().get_str("new_data_2").is_ok());
    }

    #[test]
    fn get_document() {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let collection_name = "test_get_document".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });

        let document = database.create_collection(collection_name.clone(), data).unwrap();

        let collection = database.get_document(collection_name, document[0].as_document().unwrap().get_str("id").unwrap().to_string()).unwrap();
        
        assert!(collection.get_str("some_test_key").is_ok());
    }
}