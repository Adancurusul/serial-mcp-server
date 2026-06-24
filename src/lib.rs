//! Serial MCP Server Library
//!
//! A comprehensive Model Context Protocol server for serial port communication.
//! Provides AI assistants with serial communication capabilities including
//! port discovery, connection management, data transmission, and protocol handling.

pub mod cli;
pub mod config;
pub mod error;
pub mod serial;
pub mod session;
pub mod tools;
pub mod utils;

// Re-export main types for convenience
pub use config::{Args, Config};
pub use error::{Result, SerialError};
pub use serial::{ConnectionManager, PortInfo, SerialConnection};
pub use session::{SerialSession, SessionManager, SessionState};
pub use tools::SerialHandler;
pub use utils::{DataConverter, DataFormat, PortType};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Library description
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
