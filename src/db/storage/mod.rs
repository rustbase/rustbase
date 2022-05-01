pub mod collection;
pub mod write;
pub mod read;
pub mod types;
pub mod document;

pub fn write_collection_to_database(collection: bson::Document, database_path: String) {
    return write::collection(collection, database_path).unwrap();
}

pub fn get_collection_from_database(name: String, database_path: String) -> bson::Document {
    return read::collection(name, database_path).unwrap();
}