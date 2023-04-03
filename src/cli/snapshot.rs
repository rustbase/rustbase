use dustdata::snapshots::Snapshot;
use dustdata::storage::lsm::{Lsm, LsmConfig};
use std::path::Path;

use crate::{config, SnapshotSubCommand};

pub fn run_snapshots_subcommands(subcommands: SnapshotSubCommand) {
    match subcommands {
        SnapshotSubCommand::Restore { path, db } => restore_snapshot(path, db),
        SnapshotSubCommand::Create { db, path } => create_snapshot(db, path),
    }
}

fn create_snapshot(db: String, path: String) {
    println!("[Snapshot] Creating snapshot of {} to {}", db, path);
    let snapshot_path = Path::new(&path);
    let config = config::load_configuration();

    let db_path = config.database.path.join(db);

    let lsm = Lsm::new(LsmConfig {
        flush_threshold: 0,
        sstable_path: db_path,
    });

    Snapshot::create_snapshot(&lsm, snapshot_path.to_path_buf());
    println!("[Snapshot] Done.");
}

fn restore_snapshot(path: String, db: String) {
    println!("[Restore] Restoring database from {} to {}", path, db);
    let snapshot_path = Path::new(&path);
    let config = config::load_configuration();

    let db_path = config.database.path.join(db);

    let snapshot = Snapshot::load_snapshot(snapshot_path.to_path_buf());

    Lsm::load_snapshot(db_path, snapshot);
    println!("[Snapshot] Done.");
}
