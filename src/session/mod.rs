//! Session management for serial connections
//!
//! This module provides session management functionality for tracking and
//! managing multiple serial connections and their associated state.

pub mod manager;
pub mod serial_session;

pub use manager::SessionManager;
pub use serial_session::{SerialSession, SessionConfig, SessionState};
