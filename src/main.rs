mod config;
mod query;
mod server;
mod utils;

use proctitle::set_title;

extern crate dustdata;

#[tokio::main]
async fn main() {
    set_title("Rustbase Database Server");

    let config = config::load_configuration();

    server::initalize_server(config).await;
}
