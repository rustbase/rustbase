use super::storage;

use std::fs;
use std::path;
pub struct Database {
    database_path: String,
}

impl Database {
    pub fn new(path: String) -> Self {
        println!("Initalize database, data path: {}", path);

        let db_path = path::Path::new(&path);

        if !db_path.exists() {
            fs::create_dir_all(&path).expect("Failed to create database path");
        }

        Self {
            database_path: path
        }
    }

    pub fn create_document(&self, name: String, data: Vec<storage::types::Data>) {
        let document = storage::document::create_document(name, data);
        storage::create_document_to_database(document, self.database_path.clone());
    }

    pub fn get_document() {
        unimplemented!();
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
}
