use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TerminationNotification {
    pub supi: String,
    pub pdu_session_id: i32,
    pub ts: u128,
}

impl TerminationNotification {
    pub fn ser(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).unwrap()
    }

    pub fn de(data: &[u8]) -> Self {
        serde_cbor::from_slice(data).unwrap()
    }
}

pub fn get_epoch_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
