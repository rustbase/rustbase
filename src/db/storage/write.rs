use crate::db::sharding;

use std::fs;
use std::path::Path;

pub fn document(document: bson::Document, database_path: String) -> Result<(), &'static str> {
    let shards = sharding::sharding_document::shard(document);
    for shard in shards {
        fs::write(Path::new(&database_path).join(shard.name), shard.content).expect("Failed to write document");
    }
    
    Ok(())
}