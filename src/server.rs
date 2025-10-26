use log::{info, warn, debug};
use futures::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async};
use futures::StreamExt;
use tokio_tungstenite::tungstenite::{Message};
use std::sync::Arc;
use tokio::sync::broadcast::{Sender, Receiver};
use tokio::sync::broadcast;
use tokio::select;

use crate::messages::{self, RelayMessage};

const MAX_CHANNEL_BUFF: usize = 100; // maximum messages a client can ignore before it crashes

/// enum for the type of client connected, relays will be sent messages regaurdless of the
/// channel, while clients are only sent ones in their matching channel 
#[derive(PartialEq, Debug)] // to do `==` comparisons
#[allow(dead_code)] // because relay isn't "truely used yet"
enum ClientType {
    RELAY,
    CLIENT,
}

/// An instance of the actual chat++ server. Intended to be spawned in main.
pub struct Server {
    pub events: Arc<Sender<RelayMessage>>,
    address: String,
} 

impl Server {
        pub fn new(address: String) -> Self {
            let (tx, _) = broadcast::channel(MAX_CHANNEL_BUFF);
            let tx = Arc::new(tx);
            Self {
                events: tx,
                address: address
            }
        }
        
        /// run the server, MAKE SURE YOU INSTANTIATE ALL PARAMETERS with Server::new("".to_string());
        pub async fn run(mut self) {
            info!("Starting server...");
            
            if self.address.trim() == "" {
                self.address = "127.0.0.1:9000".to_string();
            }
            let addr = &self.address; // just for clarity

            info!("WebSocket server started and listening on {}", addr);
            let listener = TcpListener::bind(addr).await.unwrap();
            
            while let Ok((stream, _)) = listener.accept().await {
                let events = self.events.clone();
                tokio::spawn(Self::handle_client(stream, events.subscribe(), events));
            }
    }

    async fn handle_client(stream: tokio::net::TcpStream, mut rx: Receiver<RelayMessage>, tx: Arc<Sender<RelayMessage>>) {
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                warn!("Websocket handshake failed {:?}", e);
                return;
            }
        };

        let (mut write, mut read) = ws_stream.split();
        info!("New WebSocket connection established");
        
        loop {
            select! {
                msg = rx.recv() => {
                    match msg {
                        Ok(m) => write.send(Message::from(m.content)).await.unwrap(),
                        Err(_) => {warn!("Client hit message buffer limit! Consider scaling. Kicking client to reduce load."); break;}
                    }
            },
            msg = read.next() => {
                if let Some(msg) = msg {
                    match msg {
                        Ok(Message::Close(_)) => break,
                        Ok(Message::Ping(_)) => info!("pinged"),
                        Ok(Message::Text(ref m)) => {
                            if m.len() > 3000 { 
                                break; 
                            }
                            debug!("handling message");
                            messages::parse_command(tx.clone(), m, None).await;
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
        info!("Client disconnected.");
    }
}
