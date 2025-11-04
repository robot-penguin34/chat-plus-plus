use tokio_tungstenite::tungstenite::{Utf8Bytes};
use log::{debug, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast::Sender;

pub struct CommandParseErr {
    pub message: String,
    pub fatal: bool,
}

impl CommandParseErr {
    pub fn from(msg: &str, fatal: bool) -> Self {
        CommandParseErr {
            message: msg.to_string(),
            fatal: fatal
        }
    }
}

impl From<String> for CommandParseErr {
    fn from(s: String) -> Self {
        CommandParseErr { message: s, fatal: false }
    }
}


/// Parse a command from a client. Expected to be json formatted to RelayMessage
/// Returns a boolean

// TODO: remove the option from option sender (debug)
pub async fn parse_command(
        tx: Arc<Sender<RelayMessage>>,
        message: &Utf8Bytes,
        sender: Option<MessageSender>,
        active_channel: &Arc<Mutex<u8>>,
    ) -> Result<(), CommandParseErr> {

    debug!("Handling client request");
    let (command, msg) = match message.split_once(' ') {
        Some((command, msg)) => (command, msg),
        None => (message.as_str(), ""),
    };
    
    match command {
        // commands that the client doesn't recognize just become a message
        // if they are a default one it gets translated to one of these
        "MESSAGE" => handle_send_message(tx, msg.to_string()).await?,
        "JOIN" => {
            let channel: u8 = {
                let tmp_str = msg.to_string();
                match tmp_str.parse::<u8>() {
                    Err(_) => {
                        return Err("Bad channel id, expected unsigned 8-bit int".to_string().into());
                    }
                    Ok(res) => { res },
                }
            };
            *active_channel.lock().await = channel;
        },
        "QUIT" => todo!(),
        "ALIVE" => todo!(), // list members in channel
        "TOPIC" => todo!(), // set the channel topic
        "DM" => todo!(),
        "QUERY" => todo!(), // get info
        _ => return Err("unrecognized command".to_string().into()),
    }
    
    Ok(())
}

pub async fn handle_send_message(tx: Arc<Sender<RelayMessage>>, content: String) -> Result<(), String>{
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
        Err(_) => {warn!("Error sending message."); return Err("Internal error".to_string());},
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
