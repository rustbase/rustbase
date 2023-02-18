mod cli;
mod config;
mod query;
mod server;
mod utils;

use colored::Colorize;
use proctitle::set_title;

use clap::{clap_derive, Parser};

extern crate dustdata;

#[derive(clap_derive::Parser)]
#[clap(author, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    sub_commands: Option<SubCommand>,
}

#[derive(clap_derive::Subcommand)]
pub enum SubCommand {
    Restore {
        /// The path to the snapshot file
        #[clap(short, long)]
        path: String,

        /// The name of the database to restore to
        #[clap(short, long)]
        db: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    cli::run_subcommands(args.sub_commands);

    set_title("Rustbase Database Server");

    println!();
    println!("{}", "Welcome to Rustbase Database Server!".bold());
    println!(
        "Current version: {} (v{})",
        env!("VERSION_CODE").green(),
        env!("CARGO_PKG_VERSION").cyan()
    );
    println!();

    let config = config::load_configuration();

    server::initalize_server(config).await;
}
