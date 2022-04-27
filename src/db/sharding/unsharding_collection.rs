pub struct CollectionShard {
    pub content: Vec<u8>,
    pub name: String,
}

pub fn unshard(mut sharded_collections: Vec<CollectionShard>) -> Vec<u8> {
    let mut collection_content: Vec<u8> = Vec::new();

    sharded_collections.sort_by(|a, b| a.name.cmp(&b.name));
    
    for mut sharded_document in sharded_collections {
        collection_content.append(&mut sharded_document.content);
    }

    collection_content
}