/*
 * Chat++ the new relay chat
 * > Made by robot-penguin34 on Github, see LICENSE.md for more info
 *
 * shoutout to RGGH on github who (through his video) helped me understand how to make websockets
 * in rust.
 */


use log::info;
use std::time::Duration;
use tokio::time;
use futures::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Utf8Bytes};
use futures::StreamExt;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    let addr = "127.0.0.1:8080";
    info!("WebSocket server started and listening on port 8080");
    let listener = TcpListener::bind(addr).await.unwrap();
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_client(stream));
    }
}

async fn handle_client(stream: tokio::net::TcpStream) {
    let ws_stream = accept_async(stream).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    info!("New WebSocket connection established");

    // Create a task to periodically send updates
    tokio::spawn(async move {
        let mut message = 0;
        let mut interval = time::interval(Duration::from_secs(3));
        
        loop {
            interval.tick().await;
            message += 1;
            let message = format!("Sending message: {} {}", message, "th message!");
            let encoded = Utf8Bytes::from(message);
            
            if write.send(Message::Text(encoded)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages (if necessary)
    while let Some(Ok(msg)) = read.next().await {
        // In this example, we don't need to handle incoming messages
        info!("Message from client {}", msg);
    }
}

