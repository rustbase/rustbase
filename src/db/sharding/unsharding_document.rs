pub struct DocumentShard {
    pub content: Vec<u8>,
    pub name: String,
}


pub fn unshard(sharded_documents: Vec<DocumentShard>) -> String {
    let mut document_content: Vec<u8> = Vec::new();
    let mut index = 0;
    for mut sharded_document in sharded_documents {
        if sharded_document.name.ends_with(index.to_string().as_str()) {
            document_content.append(&mut sharded_document.content);
            index += 1;
        }
    }

    return String::from_utf8(document_content).unwrap()
}