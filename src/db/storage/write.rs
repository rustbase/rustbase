use crate::db::sharding;
use crate::db::storage::document::Document;

use std::fs;
use std::path::Path;

pub fn document(value: Document, database_path: String) {
    let shards = sharding::sharding_document::shard(value);

    for shard in shards {
        fs::write(Path::new(&database_path).join(shard.name), shard.content).expect("Failed to write document");
    }
}