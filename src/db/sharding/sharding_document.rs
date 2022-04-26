use bson::Document;

use crate::db::storage::document::parse_document_to_bson;

pub struct DocumentShard {
    pub content: Vec<u8>,
    pub name: String,
}

pub fn shard(document: Document) -> Vec<DocumentShard> {
    let binary_document = parse_document_to_bson(document.clone());

    let document_name = document.get_str("name").unwrap().to_string();

    let mut documents_sharded: Vec<DocumentShard> = vec![];

    // Shard the document content into 3 files
    let shard_size = binary_document.len() / 4;	// 4 shards
    binary_document.chunks(shard_size).enumerate().for_each(|(index, chunk)| {
        let vec_chunk = chunk.to_vec();

        let document_shard = DocumentShard {
            content: vec_chunk,
            name: format!("{}_{}", document_name, index),
        };

        documents_sharded.push(document_shard);
    });

    return documents_sharded;
}