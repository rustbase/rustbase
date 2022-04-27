pub mod database;
pub(crate) mod storage;
mod sharding;
use super::config;

pub fn initalize_database(database_config: config::Database) -> database::Database {
    database::Database::new(database_config.path)
}
