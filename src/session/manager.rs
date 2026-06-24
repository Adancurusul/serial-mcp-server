//! Session manager implementation
//!
//! Manages multiple serial sessions with lifecycle management, cleanup, and monitoring.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Interval;
use tracing::{debug, error, info, warn};

use super::serial_session::{SerialSession, SessionConfig, SessionInfo, SessionState};
use crate::config::Config;
use crate::error::{Result, SerialError, SessionError};
use crate::serial::{ConnectParams, ConnectionManager, SerialConnection};

/// Session manager for handling multiple serial sessions
#[derive(Debug)]
pub struct SessionManager {
    /// Active sessions indexed by session ID
    sessions: Arc<RwLock<HashMap<String, SerialSession>>>,

    /// Connection manager for creating new connections
    connection_manager: Arc<ConnectionManager>,

    /// Configuration
    config: Config,

    /// Cleanup interval timer
    cleanup_interval: Option<Interval>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: Config) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            connection_manager: Arc::new(ConnectionManager::new()),
            config,
            cleanup_interval: None,
        }
    }

    /// Start the session manager (begins cleanup task)
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting session manager");

        // Start cleanup task
        let cleanup_interval_secs = 60; // Run cleanup every minute
        let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval_secs));

        let sessions_clone = Arc::clone(&self.sessions);
        let max_idle_seconds = self.config.server.connection_timeout_seconds as i64;

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                Self::cleanup_idle_sessions(&sessions_clone, max_idle_seconds).await;
            }
        });

        // Store a new interval for potential cleanup later (though not actually used)
        self.cleanup_interval = Some(tokio::time::interval(Duration::from_secs(
            cleanup_interval_secs,
        )));
        Ok(())
    }

    /// Create a new session with the given configuration
    pub async fn create_session(&self, config: SessionConfig) -> Result<String> {
        // Validate configuration
        self.validate_session_config(&config)?;

        // Check session limits
        let sessions = self.sessions.read().await;
        if sessions.len() >= self.config.server.max_connections {
            return Err(SerialError::SessionLimitExceeded(
                self.config.server.max_connections,
            ));
        }
        drop(sessions);

        // Check if port is already in use (if port sharing is disabled)
        if !self.config.serial.allow_port_sharing && self.is_port_in_use(&config.port_name).await {
            return Err(SerialError::ConnectionExists(config.port_name.clone()));
        }

        // Create new session
        let session = SerialSession::new(config);
        let session_id = session.id().to_string();

        debug!(
            "Creating session {} for port {}",
            session_id,
            session.port_name()
        );

        // Add to sessions map
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        info!("Session {} created successfully", session_id);
        Ok(session_id)
    }

    /// Connect a session to its serial port
    pub async fn connect_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SerialError::SessionNotFound(session_id.to_string()))?;

        if session.has_connection() {
            return Err(SerialError::ConnectionExists(
                "Session already connected".to_string(),
            ));
        }

        debug!(
            "Connecting session {} to port {}",
            session_id,
            session.port_name()
        );

        // Create connection
        let connection = self
            .connection_manager
            .connect(ConnectParams {
                port_name: &session.config.port_name,
                baud_rate: session.config.baud_rate,
                data_bits: session.config.data_bits,
                stop_bits: &session.config.stop_bits,
                parity: &session.config.parity,
                flow_control: &session.config.flow_control,
                timeout_ms: session.config.timeout_ms,
            })
            .await
            .map_err(|e| {
                error!("Failed to connect session {}: {}", session_id, e);
                SessionError::CreationFailed(e.to_string())
            })?;

        // Set connection in session
        session.set_connection(connection)?;
        session.reset_reconnect_attempts();

        info!("Session {} connected successfully", session_id);
        Ok(())
    }

    /// Disconnect a session
    pub async fn disconnect_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SerialError::SessionNotFound(session_id.to_string()))?;

        debug!("Disconnecting session {}", session_id);

        session.remove_connection();

        info!("Session {} disconnected", session_id);
        Ok(())
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(mut session) = sessions.remove(session_id) {
            debug!("Removing session {}", session_id);
            session.close();
            info!("Session {} removed", session_id);
            Ok(())
        } else {
            Err(SerialError::SessionNotFound(session_id.to_string()))
        }
    }

    /// Get session information
    pub async fn get_session_info(&self, session_id: &str) -> Result<SessionInfo> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| SerialError::SessionNotFound(session_id.to_string()))?;

        Ok(session.info())
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.values().map(|s| s.info()).collect()
    }

    /// List sessions for a specific port
    pub async fn list_sessions_for_port(&self, port_name: &str) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.port_name() == port_name)
            .map(|s| s.info())
            .collect()
    }

    /// Get session connection for data operations
    pub async fn get_session_connection(
        &self,
        session_id: &str,
    ) -> Result<Arc<tokio::sync::Mutex<SerialConnection>>> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SerialError::SessionNotFound(session_id.to_string()))?;

        session.touch(); // Update last accessed time

        session
            .get_connection()
            .ok_or_else(|| SerialError::InvalidConnection("Session not connected".to_string()))
    }

    /// Record data sent for a session
    pub async fn record_session_send(&self, session_id: &str, bytes: usize) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SerialError::SessionNotFound(session_id.to_string()))?;

        session.record_send(bytes);
        Ok(())
    }

    /// Record data received for a session
    pub async fn record_session_receive(&self, session_id: &str, bytes: usize) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SerialError::SessionNotFound(session_id.to_string()))?;

        session.record_receive(bytes);
        Ok(())
    }

    /// Handle session error
    pub async fn handle_session_error(&self, session_id: &str, error: String) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SerialError::SessionNotFound(session_id.to_string()))?;

        warn!("Session {} error: {}", session_id, error);
        session.set_error(error);

        // Attempt reconnection if configured
        if session.config.auto_reconnect && session.attempt_reconnect() {
            debug!("Attempting to reconnect session {}", session_id);
            // Note: Actual reconnection would be handled by the caller
            session.set_state(SessionState::Creating);
        }

        Ok(())
    }

    /// Check if a port is currently in use
    pub async fn is_port_in_use(&self, port_name: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .any(|s| s.port_name() == port_name && s.has_connection())
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Get active session count
    pub async fn active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.values().filter(|s| s.is_active()).count()
    }

    /// Get manager statistics
    pub async fn get_stats(&self) -> SessionManagerStats {
        let sessions = self.sessions.read().await;

        let total_sessions = sessions.len();
        let active_sessions = sessions.values().filter(|s| s.is_active()).count();
        let connected_sessions = sessions.values().filter(|s| s.has_connection()).count();
        let error_sessions = sessions
            .values()
            .filter(|s| matches!(s.state(), SessionState::Error(_)))
            .count();

        let total_bytes_sent = sessions.values().map(|s| s.stats.bytes_sent).sum();
        let total_bytes_received = sessions.values().map(|s| s.stats.bytes_received).sum();
        let total_messages_sent = sessions.values().map(|s| s.stats.messages_sent).sum();
        let total_messages_received = sessions.values().map(|s| s.stats.messages_received).sum();

        SessionManagerStats {
            total_sessions,
            active_sessions,
            connected_sessions,
            error_sessions,
            total_bytes_sent,
            total_bytes_received,
            total_messages_sent,
            total_messages_received,
        }
    }

    /// Validate session configuration
    fn validate_session_config(&self, config: &SessionConfig) -> Result<()> {
        if config.port_name.is_empty() {
            return Err(SerialError::InvalidConfig(
                "Port name cannot be empty".to_string(),
            ));
        }

        // Validate baud rate
        crate::utils::Validator::validate_baud_rate(config.baud_rate)?;
        crate::utils::Validator::validate_data_bits(config.data_bits)?;
        crate::utils::Validator::validate_stop_bits(&config.stop_bits)?;
        crate::utils::Validator::validate_parity(&config.parity)?;
        crate::utils::Validator::validate_flow_control(&config.flow_control)?;

        // Check security restrictions
        if self.config.security.restrict_ports {
            if !self.config.security.allowed_ports.is_empty()
                && !self
                    .config
                    .security
                    .allowed_ports
                    .iter()
                    .any(|pattern| config.port_name.contains(pattern))
            {
                return Err(SerialError::InvalidConfig(format!(
                    "Port {} is not in allowed ports list",
                    config.port_name
                )));
            }

            if self
                .config
                .security
                .blocked_ports
                .iter()
                .any(|pattern| config.port_name.contains(pattern))
            {
                return Err(SerialError::InvalidConfig(format!(
                    "Port {} is blocked",
                    config.port_name
                )));
            }
        }

        Ok(())
    }

    /// Cleanup idle sessions
    async fn cleanup_idle_sessions(
        sessions: &Arc<RwLock<HashMap<String, SerialSession>>>,
        max_idle_seconds: i64,
    ) {
        let mut sessions_to_remove = Vec::new();

        {
            let sessions_read = sessions.read().await;
            for (session_id, session) in sessions_read.iter() {
                if session.is_idle(max_idle_seconds) && !session.is_active() {
                    debug!(
                        "Session {} is idle for {} seconds, marking for cleanup",
                        session_id,
                        session.idle_seconds()
                    );
                    sessions_to_remove.push(session_id.clone());
                }
            }
        }

        if !sessions_to_remove.is_empty() {
            let mut sessions_write = sessions.write().await;
            for session_id in sessions_to_remove {
                if let Some(mut session) = sessions_write.remove(&session_id) {
                    info!("Cleaning up idle session {}", session_id);
                    session.close();
                }
            }
        }
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        // Note: In a real implementation, we might want to gracefully close all sessions
        // However, since this is async, we can't do it in Drop.
        // The cleanup should be handled by calling a shutdown method.
    }
}

/// Session manager statistics
#[derive(Debug, Clone)]
pub struct SessionManagerStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub connected_sessions: usize,
    pub error_sessions: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let config = Config::default();
        let manager = SessionManager::new(config);

        assert_eq!(manager.session_count().await, 0);
        assert_eq!(manager.active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_creation() {
        let config = Config::default();
        let manager = SessionManager::new(config);

        let session_config = SessionConfig {
            port_name: "/dev/ttyUSB0".to_string(),
            ..Default::default()
        };

        let session_id = manager.create_session(session_config).await.unwrap();
        assert!(!session_id.is_empty());
        assert_eq!(manager.session_count().await, 1);

        let info = manager.get_session_info(&session_id).await.unwrap();
        assert_eq!(info.port_name, "/dev/ttyUSB0");
    }

    #[tokio::test]
    async fn test_session_removal() {
        let config = Config::default();
        let manager = SessionManager::new(config);

        let session_config = SessionConfig {
            port_name: "/dev/ttyUSB0".to_string(),
            ..Default::default()
        };

        let session_id = manager.create_session(session_config).await.unwrap();
        assert_eq!(manager.session_count().await, 1);

        manager.remove_session(&session_id).await.unwrap();
        assert_eq!(manager.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_limits() {
        let mut config = Config::default();
        config.server.max_connections = 2;
        let manager = SessionManager::new(config);

        // Create first session
        let session_config1 = SessionConfig {
            port_name: "/dev/ttyUSB0".to_string(),
            ..Default::default()
        };
        manager.create_session(session_config1).await.unwrap();

        // Create second session
        let session_config2 = SessionConfig {
            port_name: "/dev/ttyUSB1".to_string(),
            ..Default::default()
        };
        manager.create_session(session_config2).await.unwrap();

        // Third session should fail
        let session_config3 = SessionConfig {
            port_name: "/dev/ttyUSB2".to_string(),
            ..Default::default()
        };
        let result = manager.create_session(session_config3).await;
        assert!(result.is_err());

        assert_eq!(manager.session_count().await, 2);
    }
}
