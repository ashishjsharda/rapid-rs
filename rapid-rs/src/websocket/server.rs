//! WebSocket server implementation

use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{handler::WebSocketHandler, room::RoomManager, ConnectionInfo, Message};

/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub max_message_size: usize,
    pub ping_interval_secs: u64,
    pub timeout_secs: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_message_size: 64 * 1024,
            ping_interval_secs: 30,
            timeout_secs: 60,
        }
    }
}

/// WebSocket server
pub struct WebSocketServer {
    config: WebSocketConfig,
    handler: Arc<RwLock<Option<Arc<dyn WebSocketHandler>>>>,
    room_manager: Arc<RoomManager>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self::with_config(WebSocketConfig::default())
    }
    
    pub fn with_config(config: WebSocketConfig) -> Self {
        Self {
            config,
            handler: Arc::new(RwLock::new(None)),
            room_manager: Arc::new(RoomManager::new()),
        }
    }
    
    pub async fn set_handler(&self, handler: impl WebSocketHandler + 'static) {
        *self.handler.write().await = Some(Arc::new(handler));
    }
    
    pub fn room_manager(&self) -> Arc<RoomManager> {
        self.room_manager.clone()
    }
    
    pub fn routes(&self) -> Router {
        let state = WebSocketServerState {
            config: self.config.clone(),
            handler: self.handler.clone(),
            room_manager: self.room_manager.clone(),
        };
        
        Router::new()
            .route("/ws", get(websocket_handler))
            .with_state(state)
    }
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct WebSocketServerState {
    config: WebSocketConfig,
    handler: Arc<RwLock<Option<Arc<dyn WebSocketHandler>>>>,
    room_manager: Arc<RoomManager>,
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: WebSocketServerState) {
    let connection_id = Uuid::new_v4();
    let conn_info = ConnectionInfo::new(connection_id);
    
    tracing::info!(connection_id = %connection_id, "WebSocket connection established");
    
    if let Some(handler) = state.handler.read().await.as_ref() {
        if let Err(e) = handler.on_connect(connection_id, &conn_info).await {
            tracing::error!(connection_id = %connection_id, error = %e, "Connection handler error");
            return;
        }
    }
    
    let (mut sender, mut receiver) = socket.split();
    
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(WsMessage::Text(text)) => {
                tracing::debug!(connection_id = %connection_id, "Received text: {}", text);
                
                if let Some(handler) = state.handler.read().await.as_ref() {
                    let message = Message::text(text);
                    
                    if let Err(e) = handler.on_message(connection_id, message).await {
                        tracing::error!(connection_id = %connection_id, error = %e, "Message handler error");
                    }
                }
            }
            Ok(WsMessage::Binary(data)) => {
                tracing::debug!(connection_id = %connection_id, "Received binary: {} bytes", data.len());
                
                if let Some(handler) = state.handler.read().await.as_ref() {
                    // Convert binary to JSON message
                    let message = Message::json(serde_json::json!({
                        "type": "binary",
                        "size": data.len()
                    }));
                    
                    if let Err(e) = handler.on_message(connection_id, message).await {
                        tracing::error!(connection_id = %connection_id, error = %e, "Binary handler error");
                    }
                }
            }
            Ok(WsMessage::Ping(data)) => {
                if let Err(e) = sender.send(WsMessage::Pong(data)).await {
                    tracing::error!(connection_id = %connection_id, error = %e, "Failed to send pong");
                    break;
                }
            }
            Ok(WsMessage::Pong(_)) => {}
            Ok(WsMessage::Close(_)) => {
                tracing::info!(connection_id = %connection_id, "WebSocket close received");
                break;
            }
            Err(e) => {
                tracing::error!(connection_id = %connection_id, error = %e, "WebSocket error");
                break;
            }
        }
    }
    
    if let Some(handler) = state.handler.read().await.as_ref() {
        let _ = handler.on_disconnect(connection_id).await;
    }
    
    tracing::info!(connection_id = %connection_id, "WebSocket connection closed");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_websocket_config() {
        let config = WebSocketConfig::default();
        assert_eq!(config.max_message_size, 64 * 1024);
    }
    
    #[tokio::test]
    async fn test_websocket_server() {
        let server = WebSocketServer::new();
        let _routes = server.routes();
    }
}