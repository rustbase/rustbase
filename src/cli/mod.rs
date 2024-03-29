mod snapshot;
mod upgrade;

use std::process;

use crate::SubCommand;

pub async fn run_subcommands(subcommands: Option<SubCommand>) {
    if subcommands.is_none() {
        return;
    }

    let subcommands = subcommands.unwrap();

    match subcommands {
        SubCommand::Snapshot { sub_command } => {
            snapshot::run_snapshots_subcommands(sub_command);
        }

        SubCommand::Upgrade { version } => upgrade::upgrade_rustbase(version).await,
    }

    process::exit(0);
}
