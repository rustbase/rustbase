pub mod document;
pub mod write;
pub mod types;
use crate::db::storage::document::Document;

pub fn create_document_to_database(value: Document, database_path: String) {
    write::document(value, database_path);
}