use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    networkname: String,
    nodename: String,
    channelmap: HashMap<String, u8>
}

pub fn client_send_info() {
    todo!()
}
