use tokio_tungstenite::tungstenite::Utf8Bytes;
use log::{debug};

pub fn handle_client_input(message: &Utf8Bytes, ) -> bool {
    debug!("Handling client request");
    let msg = message.to_string();
    match msg {
        _ => todo!(),
    }
    return true;
}

/// struct for a relay message
#[derive(Debug, Clone)]
pub struct RelayMessage {
    pub content: String,
    pub channel: u8
}
