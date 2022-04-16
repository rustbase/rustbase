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

    pub fn create_document(&self, name: String, data: Vec<storage::types::Data>) -> Result<(), &'static str> {
        if path::Path::new(&self.database_path).join(name.clone() + "_0").exists() {
            return Err("Document already exists");
        }

        let document = storage::document::create_document(name.clone(), data);
        storage::create_document_to_database(&document, name, self.database_path.clone());

        return Ok(());
    }

    pub fn get_document(&self, name: String) -> Result<storage::document::Document, &'static str> {
        if !path::Path::new(&self.database_path).join(name.clone() + "_0").exists() {
            return Err("Document do not already exists");
        }

        return Ok(storage::get_document_from_database(name, self.database_path.clone()));
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
