
pub struct DocumentShard {
    pub content: Vec<u8>,
    pub name: String,
}

pub fn shard(document: &[u8], document_name: String) -> Vec<DocumentShard> {
    let mut documents_sharded: Vec<DocumentShard> = vec![];

    if document.len() <= 32 {
        return document.iter().copied()
            .map(|byte| DocumentShard {
                content: vec![byte],
                name: document_name.clone(),
            })
            .collect();
    }
    // Shard the document content into 4 files
    let shard_size = document.len() / 4; // 4 shards
    document
        .chunks(shard_size)
        .enumerate()
        .for_each(|(index, chunk)| {
            let vec_chunk = chunk.to_vec();
            let document_shard = DocumentShard {
                content: vec_chunk,
                name: format!("{}_{}", document_name, index),
            };

            documents_sharded.push(document_shard);
        });

    documents_sharded
}
