use crate::db::storage::document::Document;

pub struct DocumentShard {
    pub content: Vec<u8>,
    pub name: String,
}

pub fn shard(document: Document) -> Vec<DocumentShard> {
    let mut documents_sharded: Vec<DocumentShard> = vec![];

    if document.content.len() == 0 {
        return documents_sharded;
    }
    // Shard the document content into 3 files
    let shard_size = document.content.len() / 3;	// 3 shards
    document.content.chunks(shard_size).enumerate().for_each(|(index, chunk)| {
        let vec_chunk = chunk.to_vec();
        let document_shard = DocumentShard {
            content: vec_chunk,
            name: format!("{}_{}", document.name, index),
        };

        documents_sharded.push(document_shard);
    });

    return documents_sharded;
}