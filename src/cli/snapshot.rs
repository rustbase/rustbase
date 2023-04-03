use dustdata::snapshots::Snapshot;
use dustdata::storage::lsm::Lsm;
use std::path::Path;

use crate::config;

pub fn restore_snapshot(path: String, db: String) {
    println!("[Restore] Restoring database from {} to {}", path, db);
    let snapshot_path = Path::new(&path);
    let config = config::load_configuration(None);

    let db_path = config.storage.path.join(db);

    let snapshot = Snapshot::load_snapshot(snapshot_path.to_path_buf());

    Lsm::load_snapshot(db_path, snapshot);
    println!("[Snapshot] Done.");
}
