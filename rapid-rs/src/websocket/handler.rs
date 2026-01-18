//! WebSocket handler trait and implementations

use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

use super::{ConnectionInfo, Message};

pub type ConnectionId = Uuid;
pub type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

/// WebSocket message handler trait
#[async_trait]
pub trait WebSocketHandler: Send + Sync {
    /// Called when a new connection is established
    async fn on_connect(&self, conn_id: ConnectionId, _info: &ConnectionInfo) -> HandlerResult {
        tracing::info!(connection_id = %conn_id, "WebSocket connection established");
        Ok(())
    }
    
    /// Called when a message is received - NOW TAKES Message TYPE
    async fn on_message(&self, conn_id: ConnectionId, message: Message) -> HandlerResult {
        tracing::debug!(connection_id = %conn_id, "Received message: {:?}", message);
        Ok(())
    }
    
    /// Called when a connection is closed
    async fn on_disconnect(&self, conn_id: ConnectionId) -> HandlerResult {
        tracing::info!(connection_id = %conn_id, "WebSocket connection closed");
        Ok(())
    }
}

/// Default echo handler implementation
pub struct EchoHandler;

#[async_trait]
impl WebSocketHandler for EchoHandler {
    async fn on_message(&self, conn_id: ConnectionId, message: Message) -> HandlerResult {
        tracing::info!(connection_id = %conn_id, "Echo: {:?}", message);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_echo_handler() {
        let handler = EchoHandler;
        let conn_id = Uuid::new_v4();
        let message = Message::text("test");
        
        assert!(handler.on_message(conn_id, message).await.is_ok());
    }
}