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
use tokio_tungstenite::{accept_async, tungstenite::Utf8Bytes, WebSocketStream};
use futures::StreamExt;
use tokio_tungstenite::tungstenite::{self, Message};
use std::sync::Arc;
use std::collections::HashMap;


mod info;

#[warn(missing_docs)]

/// transmit element of a stream
pub type Tx = futures::stream::SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>;

/// list of clients using ClineType
pub type ClientTxGroup = Arc<Mutex<HashMap<usize, Client>>>;

/// enum for the type of client connected, relays will be sent messages regaurdless of the
/// channel, while clients are only sent ones in their matching channel 
#[derive(PartialEq, Debug)] // to do `==` comparisons
#[allow(dead_code)] // because relay isn't "truely used yet"
enum ClientType {
    RELAY,
    CLIENT,
}

/// Struct for standardizing clients, expected to be mutable
#[derive(Debug)]
pub struct Client {
    transmit: Tx,
    activechannel: u8, // make sure the struct is mutable
    config: ClientType,
}

impl Client {
    /// see transmit.send
    pub async fn send(&mut self, msg: Message) -> Result<(), tungstenite::Error> {
        self.transmit.send(msg).await
    }
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
    
    let clients: ClientTxGroup = Arc::new(Mutex::new(HashMap::new()));
    let lastindex = Arc::new(Mutex::new(0usize));

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_client(stream, clients.clone(), lastindex.clone()));
    }
}

/// struct for a relay message
pub struct RelayMessage {
    content: String,
    channel: u8
}

/// broadcast message to all connected clients with matching channel
pub async fn broadcast(clients: &ClientTxGroup, message: RelayMessage) {
    let msg = Message::Text(Utf8Bytes::from(message.content));
    debug!("broadcasting message: {}", msg);

    let mut to_remove: Vec<usize> = vec![]; // broken clients
        
    let mut clients_guard = clients.lock().await;
    // send message to every client
    for (id, client) in clients_guard.iter_mut() {
        if client.config == ClientType::CLIENT && client.activechannel != message.channel {
            continue;
        } 
        if client.send(msg.clone()).await.is_err() {
            to_remove.push(*id);
        }
    }
    
    // log broken clients
    if to_remove.len() > 0 {
        info!("dropping {} broken clients after broadcast", to_remove.len());
    }

    // remove broken clients
    for id in to_remove {
        clients_guard.remove(&id);
    }

}

/// Append a client to the list of clients.
/// Expects a transmit element of a stream and the last appeneded index (for hashmap) as well as the client group (borrowed)
pub async fn append_client(tx: Tx, clients: &ClientTxGroup, newindex: usize) {
    debug!("adding client to list");
    let mut locked = clients.lock().await;
    locked.insert(newindex, Client { 
        transmit: tx,
        activechannel: 0,
        config: ClientType::CLIENT, 
    });
}

/// handler for after a client connects
async fn handle_client(stream: tokio::net::TcpStream, clients: ClientTxGroup, lastindex: Arc<Mutex<usize>>) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            warn!("Websocket handshake failed {:?}", e);
            return;
        }
    };

    let (write, mut read) = ws_stream.split();
    info!("New WebSocket connection established");
    
    let mut i = lastindex.lock().await;
    *i += 1;
    
    append_client(write, &clients, *i).await;
    drop(i); // we really need efficiency here
    
    debug!("New client joined.");
    broadcast(&clients, RelayMessage { content: "New Client joined YAyyyy".to_string(), channel: 0 }).await;

    // Handle incoming messages (if necessary)
    while let Some(msg) = read.next().await {
        // In this example, we don't need to handle incoming messages
        match msg {
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(_)) => info!("pinged"),
            Ok(Message::Text(ref m)) => info!("message from client: {}", m),
            Err(e) => warn!("Error recieving client's message: {:?}", e),
            _ => info!("message of other type recieved"),
        }
    }

    debug!("Client disconnected.");
}

