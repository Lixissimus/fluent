use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub version: u16,
    #[serde(flatten)]
    pub kind: MessageKind,
}

impl Message {
    pub fn status(pid: u32) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            kind: MessageKind::Status { pid },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageKind {
    Status { pid: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_message_has_stable_json_shape() {
        let message = Message::status(42);
        assert_eq!(message.version, PROTOCOL_VERSION);
        assert_eq!(message.kind, MessageKind::Status { pid: 42 });
        assert_eq!(
            serde_json::to_string(&message).expect("serialize message"),
            r#"{"version":1,"type":"status","pid":42}"#
        );
    }
}
