//! Error types for the serial MCP server
//!
//! This module provides comprehensive error handling for all serial communication
//! and MCP server operations, inspired by embedded-debugger-mcp's error design.

use thiserror::Error;

/// Main error type for the serial MCP server
#[derive(Error, Debug)]
pub enum SerialError {
    // Connection related errors
    #[error("Port not found: {0}")]
    PortNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid connection ID: {0}")]
    InvalidConnection(String),

    #[error("Connection already exists: {0}")]
    ConnectionExists(String),

    #[error("Connection limit exceeded (max: {0})")]
    ConnectionLimitExceeded(usize),

    #[error("Operation timeout")]
    OperationTimeout,

    // Communication related errors
    #[error("Read timeout")]
    ReadTimeout,

    #[error("Write timeout")]
    WriteTimeout,

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Buffer overflow")]
    BufferOverflow,

    #[error("Buffer underflow")]
    BufferUnderflow,

    // Configuration related errors
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid baud rate: {0}")]
    InvalidBaudRate(u32),

    #[error("Invalid data bits: {0}")]
    InvalidDataBits(u8),

    #[error("Invalid stop bits: {0}")]
    InvalidStopBits(String),

    #[error("Invalid parity: {0}")]
    InvalidParity(String),

    #[error("Invalid flow control: {0}")]
    InvalidFlowControl(String),

    // Session related errors
    #[error("Invalid session ID: {0}")]
    InvalidSession(String),

    #[error("Session limit exceeded (max: {0})")]
    SessionLimitExceeded(usize),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session already exists: {0}")]
    SessionExists(String),

    // Data encoding/decoding errors
    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Hex decoding error: {0}")]
    HexError(#[from] hex::FromHexError),

    #[error("Base64 decoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    // System and I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serial port error: {0}")]
    SerialPortError(#[from] serialport::Error),

    #[error("Tokio serial error: {0}")]
    TokioSerialError(String),

    // JSON and serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    // Internal and unexpected errors
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, SerialError>;

/// Connection specific errors
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Failed to open port {port}: {reason}")]
    OpenFailed { port: String, reason: String },

    #[error("Failed to configure port {port}: {reason}")]
    ConfigureFailed { port: String, reason: String },

    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    #[error("Connection busy")]
    ConnectionBusy,

    #[error("Connection not established")]
    NotConnected,

    #[error("Invalid connection state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },
}

impl From<ConnectionError> for SerialError {
    fn from(error: ConnectionError) -> Self {
        SerialError::ConnectionFailed(error.to_string())
    }
}

/// Protocol specific errors
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Unknown protocol: {0}")]
    UnknownProtocol(String),

    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),

    #[error("Checksum mismatch: expected {expected:02x}, got {actual:02x}")]
    ChecksumMismatch { expected: u8, actual: u8 },

    #[error("Invalid frame format: {0}")]
    InvalidFrameFormat(String),

    #[error("Frame too large: {size} bytes (max: {max_size})")]
    FrameTooLarge { size: usize, max_size: usize },

    #[error("Frame too small: {size} bytes (min: {min_size})")]
    FrameTooSmall { size: usize, min_size: usize },

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

impl From<ProtocolError> for SerialError {
    fn from(error: ProtocolError) -> Self {
        SerialError::ProtocolError(error.to_string())
    }
}

/// Session management errors
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session creation failed: {0}")]
    CreationFailed(String),

    #[error("Session not active: {0}")]
    NotActive(String),

    #[error("Session timeout: {0}")]
    Timeout(String),

    #[error("Session conflict: {0}")]
    Conflict(String),

    #[error("Session cleanup failed: {0}")]
    CleanupFailed(String),
}

impl From<SessionError> for SerialError {
    fn from(error: SessionError) -> Self {
        SerialError::InternalError(error.to_string())
    }
}

/// Configuration validation errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },

    #[error("Value out of range for {field}: {value} (range: {min}-{max})")]
    ValueOutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },

    #[error("Conflicting settings: {0}")]
    ConflictingSettings(String),

    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Configuration file not readable: {0}")]
    FileNotReadable(String),
}

impl From<ConfigError> for SerialError {
    fn from(error: ConfigError) -> Self {
        SerialError::InvalidConfig(error.to_string())
    }
}

/// Data processing errors
#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },

    #[error("Data corruption detected: {0}")]
    DataCorruption(String),

    #[error("Buffer size exceeded: {size} bytes (max: {max_size})")]
    BufferSizeExceeded { size: usize, max_size: usize },

    #[error("Incomplete data: expected {expected} bytes, got {actual}")]
    IncompleteData { expected: usize, actual: usize },

    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),
}

impl From<DataError> for SerialError {
    fn from(error: DataError) -> Self {
        SerialError::EncodingError(error.to_string())
    }
}

// Note: tokio_serial::Error is the same as serialport::Error, so we don't need a separate impl

// Conversion from anyhow::Error for external integrations
impl From<anyhow::Error> for SerialError {
    fn from(error: anyhow::Error) -> Self {
        SerialError::InternalError(error.to_string())
    }
}

/// Helper functions for error handling
impl SerialError {
    /// Check if error is recoverable (connection can be retried)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SerialError::OperationTimeout
                | SerialError::ReadTimeout
                | SerialError::WriteTimeout
                | SerialError::CommunicationError(_)
                | SerialError::BufferOverflow
                | SerialError::BufferUnderflow
                | SerialError::TokioSerialError(_)
        )
    }

    /// Check if error is a configuration issue
    pub fn is_configuration_error(&self) -> bool {
        matches!(
            self,
            SerialError::InvalidConfig(_)
                | SerialError::InvalidBaudRate(_)
                | SerialError::InvalidDataBits(_)
                | SerialError::InvalidStopBits(_)
                | SerialError::InvalidParity(_)
                | SerialError::InvalidFlowControl(_)
        )
    }

    /// Check if error is related to connection management
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            SerialError::PortNotFound(_)
                | SerialError::ConnectionFailed(_)
                | SerialError::InvalidConnection(_)
                | SerialError::ConnectionExists(_)
                | SerialError::ConnectionLimitExceeded(_)
        )
    }

    /// Get error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            SerialError::PortNotFound(_)
            | SerialError::ConnectionFailed(_)
            | SerialError::InvalidConnection(_)
            | SerialError::ConnectionExists(_)
            | SerialError::ConnectionLimitExceeded(_) => "connection",

            SerialError::OperationTimeout
            | SerialError::ReadTimeout
            | SerialError::WriteTimeout
            | SerialError::CommunicationError(_)
            | SerialError::ProtocolError(_)
            | SerialError::BufferOverflow
            | SerialError::BufferUnderflow => "communication",

            SerialError::InvalidConfig(_)
            | SerialError::InvalidBaudRate(_)
            | SerialError::InvalidDataBits(_)
            | SerialError::InvalidStopBits(_)
            | SerialError::InvalidParity(_)
            | SerialError::InvalidFlowControl(_) => "configuration",

            SerialError::InvalidSession(_)
            | SerialError::SessionLimitExceeded(_)
            | SerialError::SessionNotFound(_)
            | SerialError::SessionExists(_) => "session",

            SerialError::EncodingError(_)
            | SerialError::Utf8Error(_)
            | SerialError::HexError(_)
            | SerialError::Base64Error(_) => "encoding",

            SerialError::IoError(_)
            | SerialError::SerialPortError(_)
            | SerialError::TokioSerialError(_) => "system",

            SerialError::SerializationError(_) | SerialError::TomlError(_) => "serialization",

            SerialError::InternalError(_) => "internal",
        }
    }
}
