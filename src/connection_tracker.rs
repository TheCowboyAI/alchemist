//! Connection tracking for monitoring active connections

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Types of connections we track
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ConnectionType {
    Nats,
    Redis,
    Http,
    WebSocket,
    Database,
    Other(String),
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: String,
    pub conn_type: ConnectionType,
    pub remote_addr: String,
    pub established_at: Instant,
    pub last_activity: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub is_active: bool,
}

/// Connection tracker for monitoring active connections
pub struct ConnectionTracker {
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    cleanup_interval: Duration,
}

impl ConnectionTracker {
    pub fn new() -> Self {
        let tracker = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval: Duration::from_secs(60),
        };
        
        // Start cleanup task
        let connections = tracker.connections.clone();
        let cleanup_interval = tracker.cleanup_interval;
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(cleanup_interval).await;
                
                let now = Instant::now();
                let mut conns = connections.write().await;
                
                // Remove inactive connections older than 5 minutes
                conns.retain(|id, info| {
                    if !info.is_active && now.duration_since(info.last_activity) > Duration::from_secs(300) {
                        debug!("Removing inactive connection: {}", id);
                        false
                    } else {
                        true
                    }
                });
            }
        });
        
        tracker
    }
    
    /// Register a new connection
    pub async fn register_connection(
        &self,
        id: String,
        conn_type: ConnectionType,
        remote_addr: String,
    ) {
        let info = ConnectionInfo {
            id: id.clone(),
            conn_type: conn_type.clone(),
            remote_addr,
            established_at: Instant::now(),
            last_activity: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            is_active: true,
        };
        
        let mut connections = self.connections.write().await;
        connections.insert(id.clone(), info);
        
        info!("Connection registered: {} ({:?})", id, conn_type);
    }
    
    /// Update connection activity
    pub async fn update_activity(
        &self,
        id: &str,
        bytes_sent: u64,
        bytes_received: u64,
    ) {
        let mut connections = self.connections.write().await;
        
        if let Some(info) = connections.get_mut(id) {
            info.last_activity = Instant::now();
            info.bytes_sent += bytes_sent;
            info.bytes_received += bytes_received;
        }
    }
    
    /// Mark connection as inactive
    pub async fn mark_inactive(&self, id: &str) {
        let mut connections = self.connections.write().await;
        
        if let Some(info) = connections.get_mut(id) {
            info.is_active = false;
            info.last_activity = Instant::now();
            debug!("Connection marked inactive: {}", id);
        }
    }
    
    /// Get current connection count
    pub async fn get_connection_count(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        
        let mut stats = ConnectionStats::default();
        stats.total = connections.len() as u32;
        
        for (_, info) in connections.iter() {
            if info.is_active {
                stats.active += 1;
                
                match &info.conn_type {
                    ConnectionType::Nats => stats.nats += 1,
                    ConnectionType::Redis => stats.redis += 1,
                    ConnectionType::Http => stats.http += 1,
                    ConnectionType::WebSocket => stats.websocket += 1,
                    ConnectionType::Database => stats.database += 1,
                    ConnectionType::Other(_) => stats.other += 1,
                }
            }
        }
        
        stats
    }
    
    /// Get detailed connection information
    pub async fn get_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }
    
    /// Get connection by ID
    pub async fn get_connection(&self, id: &str) -> Option<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }
    
    /// Clear all connections (for testing)
    pub async fn clear(&self) {
        let mut connections = self.connections.write().await;
        connections.clear();
    }
}

/// Connection statistics
#[derive(Debug, Default, Clone)]
pub struct ConnectionStats {
    pub total: u32,
    pub active: u32,
    pub nats: u32,
    pub redis: u32,
    pub http: u32,
    pub websocket: u32,
    pub database: u32,
    pub other: u32,
}

impl ConnectionStats {
    /// Get total active connections
    pub fn total_active(&self) -> u32 {
        self.active
    }
    
    /// Format as summary string
    pub fn summary(&self) -> String {
        format!(
            "Connections: {} total ({} active) - NATS: {}, Redis: {}, HTTP: {}, WS: {}, DB: {}, Other: {}",
            self.total, self.active, self.nats, self.redis, 
            self.http, self.websocket, self.database, self.other
        )
    }
}

/// Global connection tracker instance
static GLOBAL_TRACKER: once_cell::sync::OnceCell<ConnectionTracker> = once_cell::sync::OnceCell::new();

/// Get the global connection tracker
pub fn global_tracker() -> &'static ConnectionTracker {
    GLOBAL_TRACKER.get_or_init(|| ConnectionTracker::new())
}

/// Helper macros for easy connection tracking
#[macro_export]
macro_rules! track_connection {
    ($id:expr, $conn_type:expr, $addr:expr) => {
        $crate::connection_tracker::global_tracker()
            .register_connection($id.to_string(), $conn_type, $addr.to_string())
            .await
    };
}

#[macro_export]
macro_rules! update_connection {
    ($id:expr, $sent:expr, $recv:expr) => {
        $crate::connection_tracker::global_tracker()
            .update_activity($id, $sent, $recv)
            .await
    };
}

#[macro_export]
macro_rules! close_connection {
    ($id:expr) => {
        $crate::connection_tracker::global_tracker()
            .mark_inactive($id)
            .await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_tracking() {
        let tracker = ConnectionTracker::new();
        
        // Register connections
        tracker.register_connection(
            "conn1".to_string(),
            ConnectionType::Nats,
            "127.0.0.1:4222".to_string(),
        ).await;
        
        tracker.register_connection(
            "conn2".to_string(),
            ConnectionType::Redis,
            "127.0.0.1:6379".to_string(),
        ).await;
        
        // Update activity
        tracker.update_activity("conn1", 1024, 2048).await;
        
        // Check stats
        let stats = tracker.get_connection_count().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.active, 2);
        assert_eq!(stats.nats, 1);
        assert_eq!(stats.redis, 1);
        
        // Mark one inactive
        tracker.mark_inactive("conn2").await;
        
        let stats = tracker.get_connection_count().await;
        assert_eq!(stats.active, 1);
        
        // Get connection details
        let connections = tracker.get_connections().await;
        assert_eq!(connections.len(), 2);
        
        let conn1 = connections.iter().find(|c| c.id == "conn1").unwrap();
        assert_eq!(conn1.bytes_sent, 1024);
        assert_eq!(conn1.bytes_received, 2048);
    }
    
    #[tokio::test]
    async fn test_global_tracker() {
        let tracker = global_tracker();
        
        // Clear any existing connections
        tracker.clear().await;
        
        // Test macros
        track_connection!("test_conn", ConnectionType::Http, "192.168.1.1:80");
        update_connection!("test_conn", 100, 200);
        
        let stats = tracker.get_connection_count().await;
        assert!(stats.total >= 1);
        
        close_connection!("test_conn");
    }
}