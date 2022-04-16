pub fn unshard(sharded_documents: Vec<Vec<u8>>) -> String {
    let mut document_content: Vec<u8> = Vec::new();
    for mut sharded_document in sharded_documents {
        document_content.append(&mut sharded_document);
    }

    let document = String::from_utf8_lossy(&document_content);

    return document.to_string();
}