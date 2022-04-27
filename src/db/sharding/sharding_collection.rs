use bson::Document;

use crate::db::storage::collection::parse_collection_to_bson;

pub struct CollectionShard {
    pub content: Vec<u8>,
    pub name: String,
}

pub fn shard(collection: Document) -> Vec<CollectionShard> {
    let binary_collection = parse_collection_to_bson(collection.clone());

    let collection_name = collection.get_str("name").unwrap().to_string();

    let mut collections_sharded: Vec<CollectionShard> = vec![];

    // Shard the document content into 3 files
    let shard_size = binary_collection.len() / 4;	// 4 shards
    binary_collection.chunks(shard_size).enumerate().for_each(|(index, chunk)| {
        let vec_chunk = chunk.to_vec();

        let collection_shard = CollectionShard {
            content: vec_chunk,
            name: format!("{}_{}", collection_name, index),
        };

        collections_sharded.push(collection_shard);
    });

    collections_sharded
}