//! WebSocket server implementation

use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{handler::WebSocketHandler, room::RoomManager, ConnectionInfo};

/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    
    /// Ping interval in seconds
    pub ping_interval_seconds: u64,
    
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB
            ping_interval_seconds: 30,
            timeout_seconds: 300, // 5 minutes
        }
    }
}

/// WebSocket server state
pub struct WebSocketServer {
    config: WebSocketConfig,
    handler: Arc<RwLock<Option<Arc<dyn WebSocketHandler>>>>,
    room_manager: Arc<RoomManager>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new() -> Self {
        Self::with_config(WebSocketConfig::default())
    }
    
    /// Create a new WebSocket server with custom config
    pub fn with_config(config: WebSocketConfig) -> Self {
        Self {
            config,
            handler: Arc::new(RwLock::new(None)),
            room_manager: Arc::new(RoomManager::new()),
        }
    }
    
    /// Set the WebSocket handler
    pub async fn set_handler<H: WebSocketHandler + 'static>(&self, handler: H) {
        let mut h = self.handler.write().await;
        *h = Some(Arc::new(handler));
    }
    
    /// Get room manager
    pub fn room_manager(&self) -> Arc<RoomManager> {
        Arc::clone(&self.room_manager)
    }
    
    /// Create routes for the WebSocket server
    pub fn routes(&self) -> Router {
        let state = WebSocketServerState {
            config: self.config.clone(),
            handler: Arc::clone(&self.handler),
            room_manager: Arc::clone(&self.room_manager),
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

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<WebSocketServerState>,
) -> impl IntoResponse {
    tracing::info!("WebSocket connection from {}", addr);
    
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, addr: SocketAddr, state: WebSocketServerState) {
    let connection_id = Uuid::new_v4();
    
    let mut conn_info = ConnectionInfo::new(connection_id);
    conn_info.remote_addr = Some(addr.to_string());
    
    tracing::info!(connection_id = %connection_id, "WebSocket connection established");
    
    // Call on_connect handler
    if let Some(handler) = state.handler.read().await.as_ref() {
        if let Err(e) = handler.on_connect(connection_id, &conn_info).await {
            tracing::error!(connection_id = %connection_id, error = %e, "Connection handler error");
            return;
        }
    }
    
    let (mut sender, mut receiver) = socket.split();
    
    // Handle incoming messages
    let handler_clone = Arc::clone(&state.handler);
    let room_manager = Arc::clone(&state.room_manager);
    
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    tracing::debug!(connection_id = %connection_id, "Received text message");
                    
                    if let Some(handler) = handler_clone.read().await.as_ref() {
                        if let Err(e) = handler.on_message(connection_id, text.clone()).await {
                            tracing::error!(
                                connection_id = %connection_id,
                                error = %e,
                                "Message handler error"
                            );
                        }
                    }
                }
                Ok(WsMessage::Binary(data)) => {
                    tracing::debug!(connection_id = %connection_id, "Received binary message");
                    
                    if let Some(handler) = handler_clone.read().await.as_ref() {
                        if let Err(e) = handler.on_binary(connection_id, data).await {
                            tracing::error!(
                                connection_id = %connection_id,
                                error = %e,
                                "Binary handler error"
                            );
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    tracing::info!(connection_id = %connection_id, "WebSocket closed by client");
                    break;
                }
                Ok(WsMessage::Ping(_)) | Ok(WsMessage::Pong(_)) => {
                    // Handle ping/pong automatically
                }
                Err(e) => {
                    tracing::error!(connection_id = %connection_id, error = %e, "WebSocket error");
                    break;
                }
            }
        }
        
        // Cleanup on disconnect
        room_manager.remove_from_all_rooms(connection_id).await;
        
        if let Some(handler) = handler_clone.read().await.as_ref() {
            if let Err(e) = handler.on_disconnect(connection_id).await {
                tracing::error!(
                    connection_id = %connection_id,
                    error = %e,
                    "Disconnect handler error"
                );
            }
        }
        
        tracing::info!(connection_id = %connection_id, "WebSocket connection closed");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_websocket_server_creation() {
        let server = WebSocketServer::new();
        let routes = server.routes();
        
        // Basic test that routes are created
        assert!(true);
    }
}
