/*
 * Chat++ the new relay chat
 * > Made by robot-penguin34 on Github, see LICENSE.md for more info
 */
use std::env;
use std::collections::HashMap;
use std::sync::Arc;
use env_logger::Env;
use log::{info, warn, trace};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use tokio::sync::Mutex;

mod tools;

#[warn(missing_docs)]

#[derive(Debug, PartialEq)]
#[allow(dead_code)] // because relay is unused as of yet
enum NodeMode {
    Client,
    Relay,
}

#[allow(dead_code)]
/// Struct to represent all Clients or parent
/// the Node struct needs all details to be verified for security
struct Node {
    mode: NodeMode,
    // Authorized titles is a map of names that this specified node is allowed to broadcast
    // every message has a `user@adress` field, this just ensures that this specific node
    // is allowed to broadcast messages from a specified `address` as a client of this node.
    // (Clients are untrusted, parents are trusted)
    authorized_titles: Option<HashMap<String, String>>,
    title: String, // the child's own title if this is a client it's the `title@...`
                   // if this is a Relay it would be `...@title`
    stream: Arc<Mutex<OwnedWriteHalf>>,
}

impl Node {
    /// send a tcp (byte) message to a specified client
    pub async fn send(&self, msg: &[u8]) -> Result<(), std::io::Error> {
        let mut writer = self.stream.lock().await;
        writer.write_all(msg).await?;

        Ok(())
    }
}

#[allow(dead_code)]
struct Message {
    last_hop_id: u32,
    sender: String, // username (or)
    first_relay_name: String, // this is self verified from log in, there is a certain level of trust
                              // the name of the first relay to touch this message.
    content: String,
}

async fn handle_login(writer: OwnedWriteHalf, clients: Arc<Mutex<HashMap<usize, Node>>>,
    last_insert_id: Arc<Mutex<usize>>, challenge: String) -> Result<usize, String>{
    //TODO: authentication
    trace!("attempted login with {}", challenge);
    
    // instantiate a new Node object (child)
    let client = Node {
            mode: NodeMode::Client,
            authorized_titles: None,
            title: "Test".to_string(),
            stream: Arc::new(Mutex::new(writer))
    };
    
    // increment the global index
    let mut i_lock = last_insert_id.lock().await;
    *i_lock += 1;
    let id = *i_lock;
    drop(i_lock); // efficiency++
    
    // append the client to the group
    let mut lock = clients.lock().await;
    lock.insert(id, client);
    return Ok(id);
}

async fn perform_login_task(reader: &mut OwnedReadHalf, writer: OwnedWriteHalf,
    clients: &Arc<Mutex<HashMap<usize, Node>>>, last_insert_id: Arc<Mutex<usize>>) -> Result<usize, String> {
    
    let mut buffer = [0; 1024];
    match reader.read(&mut buffer).await {
        Ok(n) => {
            if n == 0 { return Err("Sanity check failed".to_string()); }
            return handle_login(writer, clients.clone(), last_insert_id, String::from_utf8_lossy(&buffer).to_string()).await;
        },
        Err(e) => return Err(e.to_string()),
    }
}

async fn handle_client(stream: TcpStream, 
    clients: Arc<Mutex<HashMap<usize, Node>>>, last_insert_id: Arc<Mutex<usize>>) {

    let peer_addr = stream.peer_addr()
        .map_or_else(|_| "unknown".to_string(), |addr| addr.to_string());
    
    info!("Client joined at: {}", peer_addr);
    
    let mut buffer = [0; 1024];
    let (mut reader, writer) = stream.into_split();
    
    // log in the client and use the mapid to send messages
    let mapid: usize;
    match perform_login_task(&mut reader, writer, &clients, last_insert_id).await {
        Ok(id) => mapid = id,
        Err(m) => return warn!("Client failed to authenticate: {}", m),
    }

    loop {
        match reader.read(&mut buffer).await {
            Ok(n) => {
                // EOF sanity check (client disconnected)
                if n == 0 {warn!("sanity trolled"); break; }
                
                let lock = clients.lock().await;
                let current = match lock.get(&mapid) {
                    Some(n) => n,
                    None => break,
                };

                match current.send(b"wassup, I got a message").await {
                    Err(_) => break,
                    _ => {},
                };
                
            },
            Err(e) => {
                match e.kind() {
                    ErrorKind::Interrupted => { continue; }
                    ErrorKind::ConnectionReset => {
                        info!("Client {} reset their connection.", peer_addr);
                        break;
                    }
                    _ => {
                        warn!("Read error from client {}: {}", peer_addr, e);
                        break;
                    }
                }
            }
            
        }
    }

    info!("Client at {} disconnected.", peer_addr);
    // stream closes itself (dropped)
}

#[tokio::main]
async fn main() {
    let last_inserted: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let clients: Arc<Mutex<HashMap<usize, Node>>> = Arc::new(Mutex::new(HashMap::new()));

    env_logger::Builder::from_env(Env::default().filter_or("RUST_LOG", "info")).init();
    
    info!("Chat++ starting up...");

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    info!("binding node to {}", addr);
    let listener = TcpListener::bind(&addr).await
        .expect("Failed to bind node to adress");
    info!("Successfully bound node!");

    info!("Node successfully set up and listening on {}", addr);
    
    // wait until a client attempts to connect to the server
    loop {
        if let Ok(stream) = listener.accept().await {
            tokio::spawn(handle_client(stream.0, clients.clone(), last_inserted.clone()));
        } else {
            warn!("Failed to accept connection from client.");
        }
    }
}
