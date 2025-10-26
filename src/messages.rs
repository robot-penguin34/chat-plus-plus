use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Utf8Bytes;
use log::{debug, info, warn};

use crate::server::Server;

/// Parse a command from a client. Expected to be json formatted to RelayMessage
/// Returns a boolean
pub async fn parse_command(conn: &Server, message: &Utf8Bytes, client_index: usize) -> Result<(), ()> {
    debug!("Handling client request");
    let msg: ClientCommand;
    match serde_json::from_str(message) {
        Ok(result) => msg = result,
        Err(_) => return Err(()),
    }
    
    match msg.command.as_str() {
        "message" => handle_send_message(conn, msg.content).await?,
        _ => return Err(()),
    }
    
    Ok(())
}

pub async fn handle_send_message(conn: &Server, content: Option<String>) -> Result<(), ()>{
    let message = {
        match content {
            None => return Err(()),
            Some(msg) => msg,
        }
    };
    info!("todo: send message");

    Ok(())
}

/// struct for a relay message
#[derive(Debug, Clone)]
pub struct RelayMessage {
    pub content: String,
    pub channel: u8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCommand {
    command: String, // command to be looked up in match
    content: Option<String>,
}
