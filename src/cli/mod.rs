use std::process;

use crate::SubCommand;

pub fn run_subcommands(subcommands: Option<SubCommand>) {
    if subcommands.is_none() {
        return;
    }

    let subcommands = subcommands.unwrap();

    match subcommands {}

    process::exit(0);
}
