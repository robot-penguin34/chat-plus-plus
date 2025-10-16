/*
 * Chat++ the new relay chat
 * > Made by robot-penguin34 on Github, see LICENSE.md for more info
 */
use std::env;
use env_logger::Env;
use log::{info, warn, debug, trace};
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};

#[warn(missing_docs)]

async fn handle_client(mut stream: TcpStream) {
    let peer_addr = stream.peer_addr()
        .map_or_else(|_| "unknown".to_string(), |addr| addr.to_string());

    info!("Handling client from: {}", peer_addr);

    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer).await {
            Ok(n) => {
                // EOF sanity check (client disconnected)
                if n == 0 {warn!("sanity trolled"); break; }
                
                if let Err(e) = stream.write_all(&buffer[0..n]).await {
                    warn!("Error sending message to client: {}", e);
                    break;
                } 
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
            tokio::spawn(handle_client(stream.0));
        } else {
            warn!("Failed to accept connection from client.");
        }
    }
}
