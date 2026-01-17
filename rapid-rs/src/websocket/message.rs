//! WebSocket message types and broadcasting

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::handler::ConnectionId;

/// Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageType {
    /// Text message
    #[serde(rename = "text")]
    Text { content: String },
    
    /// Binary message
    #[serde(rename = "binary")]
    Binary { data: Vec<u8> },
    
    /// JSON message
    #[serde(rename = "json")]
    Json { payload: serde_json::Value },
    
    /// System message
    #[serde(rename = "system")]
    System { message: String },
    
    /// Error message
    #[serde(rename = "error")]
    Error { code: String, message: String },
}

/// WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub from: Option<ConnectionId>,
    pub to: Option<ConnectionId>,
    pub room: Option<String>,
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Message {
    /// Create a new text message
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: None,
            to: None,
            room: None,
            message_type: MessageType::Text {
                content: content.into(),
            },
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Create a new JSON message
    pub fn json(payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: None,
            to: None,
            room: None,
            message_type: MessageType::Json { payload },
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Create a system message
    pub fn system(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: None,
            to: None,
            room: None,
            message_type: MessageType::System {
                message: message.into(),
            },
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Create an error message
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: None,
            to: None,
            room: None,
            message_type: MessageType::Error {
                code: code.into(),
                message: message.into(),
            },
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Set sender
    pub fn from(mut self, conn_id: ConnectionId) -> Self {
        self.from = Some(conn_id);
        self
    }
    
    /// Set recipient
    pub fn to(mut self, conn_id: ConnectionId) -> Self {
        self.to = Some(conn_id);
        self
    }
    
    /// Set room
    pub fn in_room(mut self, room: impl Into<String>) -> Self {
        self.room = Some(room.into());
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Broadcast options
#[derive(Debug, Clone)]
pub struct BroadcastOptions {
    /// Exclude these connections from broadcast
    pub exclude: Vec<ConnectionId>,
    
    /// Only send to these connections
    pub only: Option<Vec<ConnectionId>>,
    
    /// Room to broadcast to
    pub room: Option<String>,
}

impl Default for BroadcastOptions {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            only: None,
            room: None,
        }
    }
}

impl BroadcastOptions {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn exclude(mut self, conn_ids: Vec<ConnectionId>) -> Self {
        self.exclude = conn_ids;
        self
    }
    
    pub fn only(mut self, conn_ids: Vec<ConnectionId>) -> Self {
        self.only = Some(conn_ids);
        self
    }
    
    pub fn in_room(mut self, room: impl Into<String>) -> Self {
        self.room = Some(room.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let msg = Message::text("Hello, World!")
            .from(Uuid::new_v4())
            .in_room("general");
        
        assert!(matches!(msg.message_type, MessageType::Text { .. }));
        assert!(msg.from.is_some());
        assert_eq!(msg.room, Some("general".to_string()));
    }
    
    #[test]
    fn test_message_serialization() {
        let msg = Message::json(serde_json::json!({
            "action": "ping"
        }));
        
        let json = msg.to_json().unwrap();
        let deserialized = Message::from_json(&json).unwrap();
        
        assert!(matches!(deserialized.message_type, MessageType::Json { .. }));
    }
}
