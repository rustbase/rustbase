use self::document::Document;

pub mod document;
pub mod write;
pub mod read;
pub mod types;

pub fn write_document_to_database(value: &[u8], document_name: String, database_path: String) {
    return write::document(value, document_name, database_path).unwrap();
}

pub fn get_document_from_database(name: String, database_path: String) -> Document {
    return read::document(name, database_path).unwrap();
}