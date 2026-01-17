//! WebSocket handler trait

use async_trait::async_trait;
use uuid::Uuid;

use super::ConnectionInfo;

pub type ConnectionId = Uuid;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// WebSocket event handler trait
#[async_trait]
pub trait WebSocketHandler: Send + Sync {
    /// Called when a new connection is established
    async fn on_connect(&self, conn_id: ConnectionId, info: &ConnectionInfo) -> HandlerResult {
        tracing::info!(connection_id = %conn_id, "New WebSocket connection");
        Ok(())
    }
    
    /// Called when a text message is received
    async fn on_message(&self, conn_id: ConnectionId, message: String) -> HandlerResult;
    
    /// Called when a binary message is received
    async fn on_binary(&self, conn_id: ConnectionId, data: Vec<u8>) -> HandlerResult {
        tracing::debug!(connection_id = %conn_id, size = data.len(), "Binary message received");
        Ok(())
    }
    
    /// Called when a connection is closed
    async fn on_disconnect(&self, conn_id: ConnectionId) -> HandlerResult {
        tracing::info!(connection_id = %conn_id, "WebSocket connection closed");
        Ok(())
    }
    
    /// Called on connection error
    async fn on_error(&self, conn_id: ConnectionId, error: Box<dyn std::error::Error + Send + Sync>) {
        tracing::error!(connection_id = %conn_id, error = %error, "WebSocket error");
    }
}

/// Example handler implementation
pub struct EchoHandler;

#[async_trait]
impl WebSocketHandler for EchoHandler {
    async fn on_message(&self, conn_id: ConnectionId, message: String) -> HandlerResult {
        tracing::debug!(connection_id = %conn_id, message = %message, "Echo handler received message");
        // In a real implementation, you would send the message back
        Ok(())
    }
}
