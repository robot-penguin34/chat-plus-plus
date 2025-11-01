/*
 * Chat++ the new relay chat
 * > Made by robot-penguin34 on Github, see LICENSE.md for more info
 *
 * shoutout to RGGH on github who (through his video) helped me understand how to make websockets
 * in rust.
 */
use log::{info, warn};
use server::Server;

mod messages;
mod server;
mod authentication;

#[warn(missing_docs)]
 
#[tokio::main]
async fn main() {
    let level = env_logger::Env::default().filter_or("RUST_LOG", "info");
    // Initialize logger
    env_logger::Builder::from_env(level).init();
    info!("Starting up...");
    let server = Server::new("127.0.0.1:9000".to_string());
    server.run().await;
}
