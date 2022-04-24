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

    pub fn create_document(&self, document_name: String, data: Vec<storage::types::Data>) -> Result<(), &'static str> {
        if self.check_document_exists(document_name.clone()) {
            return Err("Document already exists");
        }

        let document = storage::document::create_document(document_name.clone(), data);
        storage::write_document_to_database(&document, document_name, self.database_path.clone());

        return Ok(());
    }

    pub fn get_document(&self, document_name: String) -> Result<storage::document::Document, &'static str> {
        if !self.check_document_exists(document_name.clone()) {
            return Err("Document do not already exists");
        }

        return Ok(storage::get_document_from_database(document_name, self.database_path.clone()));
    }

    pub fn write_document(&self, document_name: String, data: Vec<storage::types::Data>) -> Result<(), &'static str> {
        if !self.check_document_exists(document_name.clone()) {
            return Err("Document do not already exists");
        }

        let document = storage::read::document(document_name.clone(), self.database_path.clone()).unwrap();

        let new_document = storage::document::write_document(document, data);

        storage::write_document_to_database(&new_document, document_name, self.database_path.clone());
    
        return Ok(())
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
        let data = vec![storage::types::Data::new(
            "test_data".to_string(),
            db::storage::types::Types::String("test_data".to_string()),
        )];

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
        let data = vec![storage::types::Data::new(
            "test_data".to_string(),
            db::storage::types::Types::String("test_data".to_string()),
        )];

        database
            .create_document(document_name.clone(), data)
            .unwrap();

        let document = database.get_document(document_name.clone()).unwrap();

        assert_eq!(document.name, document_name);
    }

    #[test]
    #[should_panic]
    fn write_document() {
        let database_path = path::Path::new(TEST_DB_PATH);

        let database = Database::new(database_path.to_str().unwrap().to_string());

        let document_name = "test_write_document".to_string();
        let data = vec![storage::types::Data::new(
            "test_data".to_string(),
            db::storage::types::Types::String("test_data".to_string()),
        )];

        database
            .create_document(document_name.clone(), data)
            .unwrap();

        let document = database.get_document(document_name.clone()).unwrap();

        let new_data = vec![storage::types::Data::new(
            "test_new_data".to_string(),
            db::storage::types::Types::String("test_new_data".to_string()),
        )];

        database.write_document(document_name.clone(), new_data).unwrap();

        let new_document = database.get_document(document_name.clone()).unwrap();

        assert_eq!(document.content[0].value, new_document.content[0].value);
    }
}