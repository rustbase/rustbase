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

        Ok(())
    }

    pub fn get_document(&self, document_name: String) -> Result<storage::document::Document, &'static str> {
        if !self.check_document_exists(document_name.clone()) {
            return Err("Document do not already exists");
        }

        Ok(storage::read_document_from_database(document_name, self.database_path.clone()))
    }

    pub fn write_document() {
        unimplemented!();
    }

    pub fn read_document() {
        unimplemented!();
    }

    pub fn list_document() {
        unimplemented!();
    }

    fn check_document_exists(&self, name: String) -> bool {
        path::Path::new(&self.database_path).join(name + "_0").exists()
    }
}
