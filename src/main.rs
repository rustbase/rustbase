mod cli;
mod config;
mod query;
mod server;
mod utils;

use colored::Colorize;
use proctitle::set_title;

use clap::{clap_derive, Parser};

extern crate dustdata;

#[derive(clap_derive::Parser, Clone)]
#[clap(author, about, long_about = None)]
pub struct Args {
    /// The path to the configuration file
    /// If not specified, the default configuration file will be used
    /// The default configuration file is located at ~/rustbase/bin/rustbaseconf.json
    /// If the default configuration file does not exist, it will be created
    #[clap(short, long)]
    config: Option<std::path::PathBuf>,

    #[clap(subcommand)]
    sub_commands: Option<SubCommand>,
}

#[derive(clap_derive::Subcommand, Clone)]
pub enum SubCommand {
    /// Manage snapshots
    Snapshot {
        #[clap(subcommand)]
        sub_command: SnapshotSubCommand,
    },

    /// Upgrade the Rustbase server
    Upgrade {
        /// The version to upgrade to
        #[clap(short, long)]
        version: Option<String>,
    },
}

#[derive(clap_derive::Subcommand, Clone)]
pub enum SnapshotSubCommand {
    /// Restore a snapshot with given path and database name
    Restore {
        /// The path to the snapshot file
        #[clap(short, long)]
        path: String,

        /// The name of the database to restore to
        #[clap(short, long)]
        db: String,
    },

    /// Create a snapshot of a database with given name and path
    Create {
        /// The name of the database to create a snapshot of
        #[clap(short, long)]
        db: String,

        /// The path to save the snapshot file to
        #[clap(short, long)]
        path: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    cli::run_subcommands(args.clone().sub_commands).await;

    set_title("Rustbase Database Server");

    println!();
    println!("{}", "Welcome to Rustbase Database Server!".bold());
    println!(
        "Current version: {} ({})",
        env!("VERSION_CODE").blue(),
        format!("v{}", env!("CARGO_PKG_VERSION")).cyan()
    );
    println!();

    let config = config::load_configuration(Some(args));

    server::initalize_server(config).await;
}
