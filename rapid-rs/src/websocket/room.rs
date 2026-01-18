//! WebSocket room management for group messaging

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::handler::ConnectionId;

/// Room manager for organizing connections into groups
pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new room
    pub async fn create_room(&self, room_id: &str) -> Room {
        let mut rooms = self.rooms.write().await;
        let room = Room::new(room_id.to_string());
        rooms.insert(room_id.to_string(), room.clone());
        tracing::info!(room_id = %room_id, "Room created");
        room
    }
    
    /// Get a room by ID
    pub async fn get_room(&self, room_id: &str) -> Option<Room> {
        let rooms = self.rooms.read().await;
        rooms.get(room_id).cloned()
    }
    
    /// Join a room
    pub async fn join_room(&self, room_id: &str, conn_id: ConnectionId) {
        let mut rooms = self.rooms.write().await;
        
        let room = rooms
            .entry(room_id.to_string())
            .or_insert_with(|| Room::new(room_id.to_string()));
        
        room.add_connection(conn_id);
        
        tracing::info!(
            room_id = %room_id,
            connection_id = %conn_id,
            "Connection joined room"
        );
    }
    
    /// Leave a room
    pub async fn leave_room(&self, room_id: &str, conn_id: ConnectionId) {
        let mut rooms = self.rooms.write().await;
        
        if let Some(room) = rooms.get_mut(room_id) {
            room.remove_connection(conn_id);
            
            tracing::info!(
                room_id = %room_id,
                connection_id = %conn_id,
                "Connection left room"
            );
            
            // Remove empty rooms
            if room.is_empty() {
                rooms.remove(room_id);
                tracing::info!(room_id = %room_id, "Empty room removed");
            }
        }
    }
    
    /// Remove connection from all rooms
    pub async fn remove_from_all_rooms(&self, conn_id: ConnectionId) {
        let mut rooms = self.rooms.write().await;
        let room_ids: Vec<String> = rooms.keys().cloned().collect();
        
        for room_id in room_ids {
            if let Some(room) = rooms.get_mut(&room_id) {
                room.remove_connection(conn_id);
                
                // Remove empty rooms
                if room.is_empty() {
                    rooms.remove(&room_id);
                }
            }
        }
    }
    
    /// Get all connections in a room
    pub async fn get_room_connections(&self, room_id: &str) -> Vec<ConnectionId> {
        let rooms = self.rooms.read().await;
        rooms
            .get(room_id)
            .map(|room| room.connections().to_vec())
            .unwrap_or_default()
    }
    
    /// List all rooms
    pub async fn list_rooms(&self) -> Vec<RoomInfo> {
        let rooms = self.rooms.read().await;
        let mut result = Vec::new();
        
        for room in rooms.values() {
            let conn_count = room.connections.read().await.len();
            result.push(RoomInfo {
                id: room.id.clone(),
                connection_count: conn_count,
            });
        }
        
        result
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual room
#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
    connections: Arc<RwLock<HashSet<ConnectionId>>>,
}

impl Room {
    pub fn new(id: String) -> Self {
        Self {
            id,
            connections: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    pub async fn add_connection(&self, conn_id: ConnectionId) {
        let mut connections = self.connections.write().await;
        connections.insert(conn_id);
    }
    
    pub fn remove_connection(&self, conn_id: ConnectionId) {
        let mut connections = self.connections.blocking_write();
        connections.remove(&conn_id);
    }
    
    pub fn is_empty(&self) -> bool {
        let connections = self.connections.blocking_read();
        connections.is_empty()
    }
    
    pub fn connections(&self) -> Vec<ConnectionId> {
        let connections = self.connections.blocking_read();
        connections.iter().copied().collect()
    }
    
    pub fn connection_count(&self) -> usize {
        let connections = self.connections.blocking_read();
        connections.len()
    }
}

/// Room information
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoomInfo {
    pub id: String,
    pub connection_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_room_management() {
        let manager = RoomManager::new();
        let conn_id = Uuid::new_v4();
        
        manager.join_room("test_room", conn_id).await;
        
        let connections = manager.get_room_connections("test_room").await;
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0], conn_id);
        
        manager.leave_room("test_room", conn_id).await;
        
        let connections = manager.get_room_connections("test_room").await;
        assert_eq!(connections.len(), 0);
    }
}