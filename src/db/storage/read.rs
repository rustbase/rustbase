use crate::db::sharding;

use std::fs;
use std::path::Path;

use super::document::Document;


pub fn document(name: String, database_path: String) -> Result<Document, &'static str> {
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

    let mut document_content: Vec<Vec<u8>> = Vec::new();
    let mut index = 0;
    for document_path in documents_path {
        if document_path.ends_with(index.to_string().as_str()) {
            let content = fs::read(Path::new(&database_path).join(document_path)).unwrap();
            document_content.push(content);
            index += 1;
        }
    }

    let string_document = sharding::unsharding_document::unshard(document_content);

    let document: Document = serde_json::from_str(&string_document).unwrap();

    return Ok(document);
}