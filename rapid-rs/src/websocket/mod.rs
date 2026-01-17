//! WebSocket support for real-time communication

pub mod server;
pub mod handler;
pub mod room;
pub mod message;

pub use server::{WebSocketServer, WebSocketConfig};
pub use handler::{WebSocketHandler, ConnectionId};
pub use room::{RoomManager, Room};
pub use message::{Message, MessageType, BroadcastOptions};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WebSocket connection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub remote_addr: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ConnectionInfo {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            user_id: None,
            connected_at: chrono::Utc::now(),
            remote_addr: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}