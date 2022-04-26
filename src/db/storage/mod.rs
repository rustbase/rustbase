pub mod document;
pub mod write;
pub mod read;
pub mod types;

pub fn write_document_to_database(document: bson::Document, database_path: String) {
    return write::document(document, database_path).unwrap();
}

pub fn get_document_from_database(name: String, database_path: String) -> bson::Document {
    return read::document(name, database_path).unwrap();
}