use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerialError {
    #[error("Port not found: {0}")]
    PortNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid connection ID: {0}")]
    InvalidConnection(String),

    #[error("Connection already exists: {0}")]
    ConnectionExists(String),

    #[error("Invalid baud rate: {0}")]
    InvalidBaudRate(u32),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Read timeout")]
    ReadTimeout,

    #[error("Write timeout")]
    WriteTimeout,

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serial port error: {0}")]
    SerialPortError(#[from] serialport::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
