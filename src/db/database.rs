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

    pub fn create_collection(&self, collection_name: String, data: bson::Bson) -> Result<(), &'static str> {
        if self.check_collection_exists(collection_name.clone()) {
            return Err("Collection already exists");
        }

        let document = storage::collection::create_collection(collection_name.clone(), data);
        storage::write_collection_to_database(document, self.database_path.clone());

        return Ok(());
    }

    pub fn get_collection(&self, collection_name: String) -> Result<bson::Document, &'static str> {
        if !self.check_collection_exists(collection_name.clone()) {
            return Err("Collection do not already exists");
        }

        return Ok(storage::get_collection_from_database(collection_name, self.database_path.clone()));
    }

    pub fn write_collection(&self, collection_name: String, data: bson::Bson) -> Result<(), &'static str> {
        if !self.check_collection_exists(collection_name.clone()) {
            return Err("Collection do not already exists");
        }

        let collection = storage::read::collection(collection_name.clone(), self.database_path.clone()).unwrap();

        let new_collection = storage::collection::write_collection(collection, data);

        storage::write_collection_to_database(new_collection, self.database_path.clone());
    
        return Ok(())
    }

    pub fn insert_document_to_collection() {
        unimplemented!();
    }

    pub fn read_document() {
        unimplemented!();
    }
    
    pub fn list_documents() {
        unimplemented!();
    }

    fn check_collection_exists(&self, name: String) -> bool {
        return path::Path::new(&self.database_path).join(name + "_0").exists()
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
    #[should_panic]
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

        let new_collection = database.get_collection(collection_name.clone()).unwrap();

        assert_eq!(collection.get_str("some_test_key").unwrap(), new_collection.get_str("some_test_key").unwrap());
    }
}