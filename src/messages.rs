use tokio_tungstenite::tungstenite::Utf8Bytes;
use log::{debug, info, warn};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

/// Parse a command from a client. Expected to be json formatted to RelayMessage
/// Returns a boolean

// TODO: remove the option from option sender (debug)
pub async fn parse_command(tx: Arc<Sender<RelayMessage>>, message: &Utf8Bytes, sender: Option<MessageSender>) -> Result<(), ()> {
    debug!("Handling client request");
    let (command, msg) = match message.split_once(' ') {
        Some((command, msg)) => (command, msg),
        None => (message.as_str(), ""),
    };
    
    match command {
        "message" => handle_send_message(tx, msg.to_string()).await?,
        _ => return Err(()),
    }
    
    Ok(())
}

pub async fn handle_send_message(tx: Arc<Sender<RelayMessage>>, content: String) -> Result<(), ()>{
    //TODO: get the sender
    let sender = MessageSender {
        username: "debug".to_string(),
        last_hop: "root".to_string(),
        claimed_first_hop: None
    };

    let message = RelayMessage {
        content: content,
        channel: 0,
        sender: sender,
        recipient: None,
    };

    match tx.send(message) {
        Err(_) => {warn!("Error sending message."); return Err(());},
        _ => {}
    }

    Ok(())
}

/// struct for a relay message
#[derive(Debug, Clone)]
pub struct RelayMessage {
    pub content: String,
    pub channel: u8,
    pub sender: MessageSender,
    pub recipient: Option<MessageSender>,
}

#[derive(Debug, Clone)]
pub struct MessageSender {
    username: String,
    last_hop: String, //TODO verify this with either JWT signin or pub/priv key signage
    claimed_first_hop: Option<String> // This is very trust based which is why it is only child
}
