pub mod database;
pub(crate) mod storage;
mod crypto;
mod sharding;
use super::config;

pub fn initalize_database(database_config: config::Database) -> database::Database {
    return database::Database::new(database_config.path);
}
