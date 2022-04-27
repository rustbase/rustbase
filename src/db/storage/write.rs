use crate::db::sharding;

use std::fs;
use std::path::Path;

pub fn collection(collection: bson::Document, database_path: String) -> Result<(), &'static str> {
    let shards = sharding::sharding_collection::shard(collection);
    for shard in shards {
        fs::write(Path::new(&database_path).join(shard.name), shard.content).expect("Failed to write collection");
    }
    
    Ok(())
}