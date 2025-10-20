/*
 * Chat++ the new relay chat
 * > Made by robot-penguin34 on Github, see LICENSE.md for more info
 *
 * shoutout to RGGH on github who (through his video) helped me understand how to make websockets
 * in rust.
 */
use log::{info, warn, debug};
use tokio::sync::Mutex;
use futures::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures::StreamExt;
use tokio_tungstenite::tungstenite::{self, Message};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::select;


mod info;
mod broadcaster;

#[warn(missing_docs)]

const MAX_MPSC_BUFF: usize = 30; // maximum messages a client can ignore before it crashes


/// enum for the type of client connected, relays will be sent messages regaurdless of the
/// channel, while clients are only sent ones in their matching channel 
#[derive(PartialEq, Debug)] // to do `==` comparisons
#[allow(dead_code)] // because relay isn't "truely used yet"
enum ClientType {
    RELAY,
    CLIENT,
}

 
#[tokio::main]
async fn main() {
    let level = env_logger::Env::default().filter_or("RUST_LOG", "info");
    // Initialize logger
    env_logger::Builder::from_env(level).init();
    info!("Starting up...");
    let addr = "127.0.0.1:9000"; // if you get opcode or frame errors change the port
    info!("WebSocket server started and listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    
    let senders = Arc::new(RwLock::new(Vec::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let (sender, reciever): (mpsc::Sender<RelayMessage>, mpsc::Receiver<RelayMessage>) = mpsc::channel(MAX_MPSC_BUFF);
        senders.write().await.push(sender);

        tokio::spawn(handle_client(stream, reciever, senders.clone()));
    }
}

/// struct for a relay message
#[derive(Debug, Clone)]
pub struct RelayMessage {
    content: String,
    channel: u8
}

/// broadcast message to all connected clients with matching channel
pub async fn broadcast(broadcastgroup: Arc<RwLock<Vec<Sender<RelayMessage>>>>, message: RelayMessage) 
    -> Result<(), tokio::sync::mpsc::error::SendError<RelayMessage>>{
    let snapshot = {
        let r = broadcastgroup.read().await;
        r.clone() // sorry for memory efficiency, but dang could that take forever
    };

    for tx in snapshot {
        tx.send(message.clone()).await?;
    }

    Ok(())
}

/// handler for after a client connects
async fn handle_client(stream: tokio::net::TcpStream, mut events: Receiver<RelayMessage>, eventgroup: Arc<RwLock<Vec<Sender<RelayMessage>>>>) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            warn!("Websocket handshake failed {:?}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    info!("New WebSocket connection established");

    match broadcast(eventgroup.clone(), RelayMessage { content: "New Client joined YAyyyy".to_string(), channel: 0 }).await {
        Ok(_) => {},
        Err(e) => {warn!("client disconnected with error: {:?}", e); return;}
    }
        loop {
            select! {
                msg = events.recv() => {
                if let Some(msg) = msg {
                    write.send(Message::from(msg.content)).await.unwrap();
                } else {
                    break; // events channel closed
                }
            },
            msg = read.next() => {
                if let Some(msg) = msg {
                    match msg {
                        Ok(Message::Close(_)) => break,
                        Ok(Message::Ping(_)) => info!("pinged"),
                        Ok(Message::Text(ref m)) => {
                            debug!("Sending message");
                            match broadcast(eventgroup.clone(), RelayMessage { content: m.to_string(), channel: 0 }).await {
                                Ok(_) => {},
                                Err(_e) => warn!("Failed to send message!"),
                            }
                        },
                        Err(e) => warn!("Error receiving client's message: {:?}", e),
                        _ => info!("message of other type received"),
                    }
                } else {
                    break; // read stream ended
                }
            }
        }
    }
    debug!("Client disconnected.");
}

