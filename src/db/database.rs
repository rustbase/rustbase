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

    pub fn create_document(&self, document_name: String, data: bson::Bson) -> Result<(), &'static str> {
        if self.check_document_exists(document_name.clone()) {
            return Err("Document already exists");
        }

        let document = storage::document::create_document(document_name.clone(), data);
        storage::write_document_to_database(document, self.database_path.clone());

        return Ok(());
    }

    pub fn get_document(&self, document_name: String) -> Result<bson::Document, &'static str> {
        if !self.check_document_exists(document_name.clone()) {
            return Err("Document do not already exists");
        }

        return Ok(storage::get_document_from_database(document_name, self.database_path.clone()));
    }

    pub fn write_document(&self, document_name: String, data: bson::Bson) -> Result<(), &'static str> {
        if !self.check_document_exists(document_name.clone()) {
            return Err("Document do not already exists");
        }

        let document = storage::read::document(document_name.clone(), self.database_path.clone()).unwrap();

        let new_document = storage::document::write_document(document, data);

        storage::write_document_to_database(new_document, self.database_path.clone());
    
        return Ok(())
    }

    pub fn insert_document() {
        unimplemented!();
    }

    pub fn read_document() {
        unimplemented!();
    }
    
    pub fn list_document() {
        unimplemented!();
    }

    fn check_document_exists(&self, name: String) -> bool {
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
    fn create_document() {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let document_name = "test_create_document".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });

        database
            .create_document(document_name.clone(), data)
            .unwrap();

        assert!(database_path.join(document_name + "_0").exists());
    }

    #[test]
    fn get_document()  {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let document_name = "test_get_document".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });

        database
            .create_document(document_name.clone(), data)
            .unwrap();

        let document = database.get_document(document_name.clone()).unwrap();

        assert_eq!(document.get_str("name").unwrap(), document_name);
    }

    #[test]
    #[should_panic]
    fn write_document() {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let document_name = "test_write_document".to_string();
        let data = bson::bson!({
            "some_test_key": "some_test_value"
        });

        database
            .create_document(document_name.clone(), data)
            .unwrap();

        let document = database.get_document(document_name.clone()).unwrap();

        let new_data = bson::bson!({
            "new_data": "new_data"
        });

        database.write_document(document_name.clone(), new_data).unwrap();

        let new_document = database.get_document(document_name.clone()).unwrap();

        assert_eq!(document.get_str("some_test_key").unwrap(), new_document.get_str("some_test_key").unwrap());
    }
}