use crate::db::sharding::unsharding_collection;

use std::fs;
use std::path::Path;

pub fn collection(name: String, database_path: String) -> Result<bson::Document, &'static str> {
    let data_dir = fs::read_dir(&database_path).unwrap();

    let mut collections_path: Vec<String> = Vec::new();
    for entry in data_dir {
        let entry = entry.unwrap();
        let entry_name = entry.file_name().into_string().unwrap();
        if entry_name.starts_with(&name) {
            collections_path.push(entry_name);
        }
    };

    if collections_path.is_empty() {
        return Err("Document not found");
    }

    let mut collection_content: Vec<unsharding_collection::CollectionShard> = Vec::new();

    for document_path in collections_path {
        let content = fs::read(Path::new(&database_path).join(document_path.clone())).unwrap();
        collection_content.push(unsharding_collection::CollectionShard {
            content,
            name: document_path,
        });
    }

    let bson_document = unsharding_collection::unshard(collection_content);

    let bson_data = bson::from_slice(&bson_document).expect("Failed to parse document, maybe the document is corrupted.");

    Ok(bson_data)
}