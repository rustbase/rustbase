use crate::db::sharding;

use std::fs;
use std::path::Path;

pub fn document(value: &[u8], document_name: String, database_path: String) {
    if value.len() < 2 {
        panic!("Document content is too small");
    }

    let shards = sharding::sharding_document::shard(value, document_name);
    for shard in shards {
        fs::write(Path::new(&database_path).join(shard.name), shard.content).expect("Failed to write document");
    }
    
    Ok(())
}