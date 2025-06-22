//! Connection management and cleanup for WebSocket connections

use super::lobby_impl::Lobby;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use uuid::Uuid;

/// Tracks active WebSocket connections and handles cleanup
pub struct ConnectionManager {
    /// Active connections mapped by username
    pub connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    /// Lobby references for cleanup
    pub lobbies: Arc<RwLock<HashMap<String, Arc<Lobby>>>>,
    /// Channel for cleanup notifications
    cleanup_tx: mpsc::UnboundedSender<CleanupRequest>,
    /// Connection pool for rate limiting
    pub pool: ConnectionPool,
    /// Reconnection tokens for graceful reconnection
    pub reconnection_tokens: Arc<RwLock<HashMap<String, ReconnectionInfo>>>,
}

#[derive(Clone)]
pub struct ConnectionInfo {
    username: String,
    lobby_id: String,
    connected_at: Instant,
    last_activity: Instant,
    is_member: bool,
}

#[derive(Clone)]
pub struct ReconnectionInfo {
    pub token: String,
    pub username: String,
    pub lobby_id: String,
    pub expires_at: Instant,
    pub is_member: bool,
}

pub enum CleanupRequest {
    Disconnect { username: String, lobby_id: String },
    PurgeStale { max_idle: Duration },
}

impl ConnectionManager {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<CleanupRequest>) {
        let (cleanup_tx, cleanup_rx) = mpsc::unbounded_channel();
        
        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            lobbies: Arc::new(RwLock::new(HashMap::new())),
            cleanup_tx,
            pool: ConnectionPool::new(3, 100), // 3 per user, 100 total
            reconnection_tokens: Arc::new(RwLock::new(HashMap::new())),
        };
        
        (manager, cleanup_rx)
    }
    
    /// Start the background cleanup task
    pub async fn start_cleanup_task(
        connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
        lobbies: Arc<RwLock<HashMap<String, Arc<Lobby>>>>,
        mut cleanup_rx: mpsc::UnboundedReceiver<CleanupRequest>,
        reconnection_tokens: Arc<RwLock<HashMap<String, ReconnectionInfo>>>,
    ) {
        // Periodic cleanup interval
        let mut cleanup_interval = interval(Duration::from_secs(60));
        
        loop {
            tokio::select! {
                Some(request) = cleanup_rx.recv() => {
                    match request {
                        CleanupRequest::Disconnect { username, lobby_id } => {
                            Self::handle_disconnect(&connections, &lobbies, &username, &lobby_id);
                        }
                        CleanupRequest::PurgeStale { max_idle } => {
                            Self::purge_stale_connections(&connections, &lobbies, max_idle);
                        }
                    }
                }
                _ = cleanup_interval.tick() => {
                    // Periodic cleanup of stale connections
                    Self::purge_stale_connections(&connections, &lobbies, Duration::from_secs(300));
                    
                    // Clean up expired reconnection tokens
                    Self::purge_expired_tokens(&reconnection_tokens);
                }
            }
        }
    }
    
    /// Clean up expired reconnection tokens
    fn purge_expired_tokens(tokens: &Arc<RwLock<HashMap<String, ReconnectionInfo>>>) {
        let now = Instant::now();
        let mut tokens_write = tokens.write();
        tokens_write.retain(|_, info| info.expires_at > now);
    }
    
    /// Register a new connection
    pub fn register_connection(&self, username: String, lobby_id: String, lobby: Arc<Lobby>, is_member: bool) {
        let info = ConnectionInfo {
            username: username.clone(),
            lobby_id: lobby_id.clone(),
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            is_member,
        };
        
        self.connections.write().insert(username.clone(), info);
        self.lobbies.write().insert(format!("{}:{}", lobby_id, username), lobby);
        
        println!("[ConnectionManager] Registered connection for {} in lobby {}", username, lobby_id);
    }
    
    /// Update last activity time
    pub fn update_activity(&self, username: &str) {
        if let Some(mut info) = self.connections.write().get_mut(username) {
            info.last_activity = Instant::now();
        }
    }
    
    /// Request disconnection
    pub fn disconnect(&self, username: &str, lobby_id: &str) {
        let _ = self.cleanup_tx.send(CleanupRequest::Disconnect {
            username: username.to_string(),
            lobby_id: lobby_id.to_string(),
        });
    }
    
    /// Generate a reconnection token for a disconnecting user
    pub fn generate_reconnection_token(&self, username: &str, lobby_id: &str, is_member: bool) -> String {
        let token = Uuid::new_v4().to_string();
        let info = ReconnectionInfo {
            token: token.clone(),
            username: username.to_string(),
            lobby_id: lobby_id.to_string(),
            expires_at: Instant::now() + Duration::from_secs(300), // 5 minute expiry
            is_member,
        };
        
        self.reconnection_tokens.write().insert(token.clone(), info);
        println!("[ConnectionManager] Generated reconnection token for {}", username);
        token
    }
    
    /// Validate and consume a reconnection token
    pub fn validate_reconnection_token(&self, token: &str) -> Option<ReconnectionInfo> {
        let mut tokens = self.reconnection_tokens.write();
        
        if let Some(info) = tokens.remove(token) {
            if info.expires_at > Instant::now() {
                // Valid token, return it
                return Some(info);
            } else {
                // Expired token
                println!("[ConnectionManager] Reconnection token expired for {}", info.username);
            }
        }
        None
    }
    
    /// Handle disconnection with proper cleanup
    fn handle_disconnect(
        connections: &Arc<RwLock<HashMap<String, ConnectionInfo>>>,
        lobbies: &Arc<RwLock<HashMap<String, Arc<Lobby>>>>,
        username: &str,
        lobby_id: &str,
    ) {
        println!("[ConnectionManager] Handling disconnect for {} from lobby {}", username, lobby_id);
        
        // Remove connection info
        let was_member = connections.write()
            .remove(username)
            .map(|info| info.is_member)
            .unwrap_or(false);
        
        // Get lobby and perform cleanup
        let key = format!("{}:{}", lobby_id, username);
        if let Some(lobby) = lobbies.write().remove(&key) {
            if was_member {
                // Set member as disconnected but don't remove them
                // This allows for reconnection
                lobby.state.set_member_connected(username, false);
                
                // Broadcast state update
                lobby.broadcast_lobby_state();
            }
        }
    }
    
    /// Purge stale connections
    fn purge_stale_connections(
        connections: &Arc<RwLock<HashMap<String, ConnectionInfo>>>,
        lobbies: &Arc<RwLock<HashMap<String, Arc<Lobby>>>>,
        max_idle: Duration,
    ) {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        
        // Find stale connections
        {
            let conns = connections.read();
            for (username, info) in conns.iter() {
                if now.duration_since(info.last_activity) > max_idle {
                    to_remove.push((username.clone(), info.lobby_id.clone()));
                }
            }
        }
        
        // Remove stale connections
        for (username, lobby_id) in to_remove {
            println!("[ConnectionManager] Purging stale connection: {} from {}", username, lobby_id);
            Self::handle_disconnect(connections, lobbies, &username, &lobby_id);
        }
    }
    
    /// Get connection statistics
    pub fn get_stats(&self) -> ConnectionStats {
        let connections = self.connections.read();
        let total = connections.len();
        let members = connections.values().filter(|c| c.is_member).count();
        let observers = total - members;
        
        let mut by_lobby: HashMap<String, usize> = HashMap::new();
        for info in connections.values() {
            *by_lobby.entry(info.lobby_id.clone()).or_insert(0) += 1;
        }
        
        ConnectionStats {
            total_connections: total,
            member_connections: members,
            observer_connections: observers,
            connections_by_lobby: by_lobby,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub member_connections: usize,
    pub observer_connections: usize,
    pub connections_by_lobby: HashMap<String, usize>,
}

/// Connection pool to prevent resource exhaustion
pub struct ConnectionPool {
    /// Maximum connections per user
    max_per_user: usize,
    /// Maximum total connections
    max_total: usize,
    /// Current connections by user
    user_connections: Arc<RwLock<HashMap<String, usize>>>,
    /// Total connection count
    total_connections: Arc<RwLock<usize>>,
}

impl ConnectionPool {
    pub fn new(max_per_user: usize, max_total: usize) -> Self {
        Self {
            max_per_user,
            max_total,
            user_connections: Arc::new(RwLock::new(HashMap::new())),
            total_connections: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Try to acquire a connection slot
    pub fn try_acquire(&self, username: &str) -> Result<ConnectionGuard, &'static str> {
        let mut total = self.total_connections.write();
        if *total >= self.max_total {
            return Err("Server at maximum capacity");
        }
        
        let mut user_conns = self.user_connections.write();
        let user_count = user_conns.entry(username.to_string()).or_insert(0);
        if *user_count >= self.max_per_user {
            return Err("Too many connections from this user");
        }
        
        *user_count += 1;
        *total += 1;
        
        Ok(ConnectionGuard {
            username: username.to_string(),
            user_connections: Arc::clone(&self.user_connections),
            total_connections: Arc::clone(&self.total_connections),
        })
    }
}

/// RAII guard for connection slots
pub struct ConnectionGuard {
    username: String,
    user_connections: Arc<RwLock<HashMap<String, usize>>>,
    total_connections: Arc<RwLock<usize>>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let mut user_conns = self.user_connections.write();
        if let Some(count) = user_conns.get_mut(&self.username) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                user_conns.remove(&self.username);
            }
        }
        
        let mut total = self.total_connections.write();
        *total = total.saturating_sub(1);
    }
}

/// WebSocket health monitoring
pub struct HealthMonitor {
    /// Tracks connection health metrics
    metrics: Arc<RwLock<HealthMetrics>>,
    /// Alert thresholds
    thresholds: HealthThresholds,
}

struct HealthMetrics {
    total_connections: usize,
    failed_sends: usize,
    disconnections: usize,
    reconnections: usize,
    avg_latency_ms: f64,
    last_reset: Instant,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            total_connections: 0,
            failed_sends: 0,
            disconnections: 0,
            reconnections: 0,
            avg_latency_ms: 0.0,
            last_reset: Instant::now(),
        }
    }
}

struct HealthThresholds {
    max_failed_sends_per_minute: usize,
    max_disconnections_per_minute: usize,
    max_avg_latency_ms: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            max_failed_sends_per_minute: 100,
            max_disconnections_per_minute: 50,
            max_avg_latency_ms: 1000.0,
        }
    }
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HealthMetrics {
                last_reset: Instant::now(),
                ..Default::default()
            })),
            thresholds: HealthThresholds::default(),
        }
    }
    
    pub fn record_failed_send(&self) {
        self.metrics.write().failed_sends += 1;
    }
    
    pub fn record_disconnection(&self) {
        self.metrics.write().disconnections += 1;
    }
    
    pub fn record_reconnection(&self) {
        self.metrics.write().reconnections += 1;
    }
    
    pub fn update_connection_count(&self, count: usize) {
        self.metrics.write().total_connections = count;
    }
    
    pub fn check_health(&self) -> HealthStatus {
        let mut metrics = self.metrics.write();
        let elapsed = metrics.last_reset.elapsed();
        
        // Reset metrics every minute
        if elapsed > Duration::from_secs(60) {
            let status = self.calculate_status(&metrics, elapsed);
            
            // Reset counters
            metrics.failed_sends = 0;
            metrics.disconnections = 0;
            metrics.reconnections = 0;
            metrics.last_reset = Instant::now();
            
            return status;
        }
        
        self.calculate_status(&metrics, elapsed)
    }
    
    fn calculate_status(&self, metrics: &HealthMetrics, elapsed: Duration) -> HealthStatus {
        let minutes = elapsed.as_secs_f64() / 60.0;
        let failed_sends_per_minute = (metrics.failed_sends as f64 / minutes) as usize;
        let disconnections_per_minute = (metrics.disconnections as f64 / minutes) as usize;
        
        if failed_sends_per_minute > self.thresholds.max_failed_sends_per_minute {
            return HealthStatus::Critical("High rate of failed sends");
        }
        
        if disconnections_per_minute > self.thresholds.max_disconnections_per_minute {
            return HealthStatus::Warning("High disconnection rate");
        }
        
        if metrics.avg_latency_ms > self.thresholds.max_avg_latency_ms {
            return HealthStatus::Warning("High average latency");
        }
        
        HealthStatus::Healthy
    }
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Warning(&'static str),
    Critical(&'static str),
}