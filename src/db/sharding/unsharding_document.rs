pub struct DocumentShard {
    pub content: Vec<u8>,
    pub name: String,
}

pub fn unshard(mut sharded_documents: Vec<DocumentShard>) -> Vec<u8> {
    let mut document_content: Vec<u8> = Vec::new();

    sharded_documents.sort_by(|a, b| a.name.cmp(&b.name));
    
    for mut sharded_document in sharded_documents {
        document_content.append(&mut sharded_document.content);
    }

    return document_content;
}