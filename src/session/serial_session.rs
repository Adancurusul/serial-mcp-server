//! Serial session implementation
//!
//! Defines the core session structure and state management for serial connections.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{Result, SerialError};
use crate::serial::SerialConnection;
use crate::utils::SessionIdGenerator;

/// Session state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is being created
    Creating,
    /// Session is active and ready
    Active,
    /// Session is temporarily suspended
    Suspended,
    /// Session is being closed
    Closing,
    /// Session is closed
    Closed,
    /// Session encountered an error
    Error(String),
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Creating => write!(f, "Creating"),
            SessionState::Active => write!(f, "Active"),
            SessionState::Suspended => write!(f, "Suspended"),
            SessionState::Closing => write!(f, "Closing"),
            SessionState::Closed => write!(f, "Closed"),
            SessionState::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: String,
    pub parity: String,
    pub flow_control: String,
    pub timeout_ms: u64,
    pub buffer_size: usize,
    pub auto_reconnect: bool,
    pub max_reconnect_attempts: u32,
    pub line_ending: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            port_name: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: "One".to_string(),
            parity: "None".to_string(),
            flow_control: "None".to_string(),
            timeout_ms: 1000,
            buffer_size: 8192,
            auto_reconnect: false,
            max_reconnect_attempts: 3,
            line_ending: "\n".to_string(),
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub errors_count: u64,
    pub reconnections: u32,
    pub last_activity: Option<DateTime<Utc>>,
}

impl SessionStats {
    pub fn record_send(&mut self, bytes: usize) {
        self.bytes_sent += bytes as u64;
        self.messages_sent += 1;
        self.last_activity = Some(Utc::now());
    }

    pub fn record_receive(&mut self, bytes: usize) {
        self.bytes_received += bytes as u64;
        self.messages_received += 1;
        self.last_activity = Some(Utc::now());
    }

    pub fn record_error(&mut self) {
        self.errors_count += 1;
        self.last_activity = Some(Utc::now());
    }

    pub fn record_reconnection(&mut self) {
        self.reconnections += 1;
        self.last_activity = Some(Utc::now());
    }
}

/// Serial session structure
#[derive(Debug)]
pub struct SerialSession {
    /// Unique session identifier
    pub session_id: String,

    /// Session configuration
    pub config: SessionConfig,

    /// Current session state
    pub state: SessionState,

    /// Session creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last access timestamp
    pub last_accessed: DateTime<Utc>,

    /// Session statistics
    pub stats: SessionStats,

    /// Optional serial connection (wrapped in `Arc<Mutex>` for thread safety)
    connection: Option<Arc<Mutex<SerialConnection>>>,

    /// Current reconnection attempt count
    reconnect_attempts: u32,
}

impl SerialSession {
    /// Create a new session with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        let session_id = SessionIdGenerator::generate();
        let now = Utc::now();

        Self {
            session_id,
            config,
            state: SessionState::Creating,
            created_at: now,
            last_accessed: now,
            stats: SessionStats::default(),
            connection: None,
            reconnect_attempts: 0,
        }
    }

    /// Get session ID
    pub fn id(&self) -> &str {
        &self.session_id
    }

    /// Get port name
    pub fn port_name(&self) -> &str {
        &self.config.port_name
    }

    /// Get current state
    pub fn state(&self) -> &SessionState {
        &self.state
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }

    /// Check if session has a connection
    pub fn has_connection(&self) -> bool {
        self.connection.is_some()
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Set connection
    pub fn set_connection(&mut self, connection: SerialConnection) -> Result<()> {
        if matches!(self.state, SessionState::Closed) {
            return Err(SerialError::InvalidSession(
                "Cannot set connection on closed session".to_string(),
            ));
        }

        self.connection = Some(Arc::new(Mutex::new(connection)));
        self.state = SessionState::Active;
        self.touch();
        Ok(())
    }

    /// Get connection (clone of Arc)
    pub fn get_connection(&self) -> Option<Arc<Mutex<SerialConnection>>> {
        self.connection.clone()
    }

    /// Remove connection
    pub fn remove_connection(&mut self) {
        self.connection = None;
        if matches!(self.state, SessionState::Active) {
            self.state = SessionState::Suspended;
        }
        self.touch();
    }

    /// Set session state
    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
        self.touch();
    }

    /// Set error state
    pub fn set_error(&mut self, error: String) {
        self.state = SessionState::Error(error);
        self.stats.record_error();
        self.touch();
    }

    /// Record data sent
    pub fn record_send(&mut self, bytes: usize) {
        self.stats.record_send(bytes);
        self.touch();
    }

    /// Record data received
    pub fn record_receive(&mut self, bytes: usize) {
        self.stats.record_receive(bytes);
        self.touch();
    }

    /// Attempt reconnection
    pub fn attempt_reconnect(&mut self) -> bool {
        if self.reconnect_attempts >= self.config.max_reconnect_attempts {
            return false;
        }

        self.reconnect_attempts += 1;
        self.stats.record_reconnection();
        self.touch();
        true
    }

    /// Reset reconnection counter
    pub fn reset_reconnect_attempts(&mut self) {
        self.reconnect_attempts = 0;
    }

    /// Close session
    pub fn close(&mut self) {
        self.state = SessionState::Closing;
        self.connection = None;
        self.touch();

        // Final state
        self.state = SessionState::Closed;
    }

    /// Get session age in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.created_at)
            .num_seconds()
    }

    /// Get seconds since last access
    pub fn idle_seconds(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.last_accessed)
            .num_seconds()
    }

    /// Check if session is idle for more than the specified duration
    pub fn is_idle(&self, max_idle_seconds: i64) -> bool {
        self.idle_seconds() > max_idle_seconds
    }

    /// Get session info as JSON-serializable structure
    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            session_id: self.session_id.clone(),
            port_name: self.config.port_name.clone(),
            state: self.state.clone(),
            created_at: self.created_at,
            last_accessed: self.last_accessed,
            age_seconds: self.age_seconds(),
            idle_seconds: self.idle_seconds(),
            has_connection: self.has_connection(),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Session information for external consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub port_name: String,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub age_seconds: i64,
    pub idle_seconds: i64,
    pub has_connection: bool,
    pub config: SessionConfig,
    pub stats: SessionStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let config = SessionConfig {
            port_name: "/dev/ttyUSB0".to_string(),
            ..Default::default()
        };

        let session = SerialSession::new(config);

        assert!(!session.id().is_empty());
        assert_eq!(session.port_name(), "/dev/ttyUSB0");
        assert!(matches!(session.state(), SessionState::Creating));
        assert!(!session.is_active());
        assert!(!session.has_connection());
    }

    #[test]
    fn test_session_state_transitions() {
        let config = SessionConfig::default();
        let mut session = SerialSession::new(config);

        session.set_state(SessionState::Active);
        assert!(session.is_active());

        session.set_error("Test error".to_string());
        assert!(matches!(session.state(), SessionState::Error(_)));
        assert!(!session.is_active());

        session.close();
        assert!(matches!(session.state(), SessionState::Closed));
    }

    #[test]
    fn test_session_stats() {
        let config = SessionConfig::default();
        let mut session = SerialSession::new(config);

        session.record_send(100);
        session.record_receive(50);

        assert_eq!(session.stats.bytes_sent, 100);
        assert_eq!(session.stats.bytes_received, 50);
        assert_eq!(session.stats.messages_sent, 1);
        assert_eq!(session.stats.messages_received, 1);
    }
}
