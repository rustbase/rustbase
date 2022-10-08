mod config;
mod query;
mod server;
mod utils;

use colored::Colorize;
use proctitle::set_title;

extern crate dustdata;

#[tokio::main]
async fn main() {
    set_title("Rustbase Database Server");

    println!();
    println!("{}", "Welcome to Rustbase Database Server!".bold());
    println!(
        "Current version: {} ({})",
        std::env::var("VERSION_CODE").unwrap().green(),
        env!("CARGO_PKG_VERSION").cyan()
    );
    println!();

    let config = config::load_configuration();

    server::initalize_server(config).await;
}
