use serde::{Deserialize, Serialize};

use crate::{Connection, Socket};

pub type ClientConnection = Connection<ServerMessage, ClientMessage>;
pub type ServerConnection = Connection<ClientMessage, ServerMessage>;
pub type Server = Socket<ClientMessage, ServerMessage>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMessage {
    #[serde(flatten)]
    pub kind: ServerMessageKind,
}

impl ServerMessage {
    pub fn status() -> Self {
        Self {
            kind: ServerMessageKind::GetStatus,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessageKind {
    GetStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientMessage {
    #[serde(flatten)]
    pub kind: ClientMessageKind,
}

impl ClientMessage {
    pub fn status(pid: u32) -> Self {
        Self {
            kind: ClientMessageKind::Status { pid },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessageKind {
    Status { pid: u32 },
}
