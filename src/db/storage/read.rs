use crate::db::sharding::unsharding_document;

use std::fs;
use std::path::Path;

pub fn document(name: String, database_path: String) -> Result<bson::Document, &'static str> {
    let data_dir = fs::read_dir(&database_path).unwrap();

    let mut documents_path: Vec<String> = Vec::new();
    for entry in data_dir {
        let entry = entry.unwrap();
        let entry_name = entry.file_name().into_string().unwrap();
        if entry_name.starts_with(&name) {
            documents_path.push(entry_name);
        }
    };

    if documents_path.len() == 0 {
        return Err("Document not found");
    }

    let mut document_content: Vec<unsharding_document::DocumentShard> = Vec::new();

    for document_path in documents_path {
        let content = fs::read(Path::new(&database_path).join(document_path.clone())).unwrap();
        document_content.push(unsharding_document::DocumentShard {
            content,
            name: document_path,
        });
    }

    let bson_document = unsharding_document::unshard(document_content);

    let bson_data = bson::from_slice(&bson_document).expect("Failed to parse document, maybe the document is corrupted.");

    return Ok(bson_data);
}