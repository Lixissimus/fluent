use serde::{Deserialize, Serialize};

use crate::{Connection, Socket};

pub type ClientConnection = Connection<ServerMessage, ClientMessage>;
pub type ServerConnection = Connection<ClientMessage, ServerMessage>;
pub type Server = Socket<ClientMessage, ServerMessage>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientMessage {
    #[serde(flatten)]
    pub kind: ClientMessageKind,
}

impl ClientMessage {
    pub fn get_status() -> Self {
        Self {
            kind: ClientMessageKind::GetStatus,
        }
    }

    pub fn set_active(pid: u32, val: bool) -> Self {
        Self {
            kind: ClientMessageKind::SetActive { pid, val },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessageKind {
    GetStatus,
    SetActive { pid: u32, val: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMessage {
    #[serde(flatten)]
    pub kind: ServerMessageKind,
}

impl ServerMessage {
    pub fn status(instances: Vec<InstanceStatus>) -> Self {
        Self {
            kind: ServerMessageKind::Status { instances },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessageKind {
    Status { instances: Vec<InstanceStatus> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstanceStatus {
    pub pid: u32,
    pub active: bool,
}
