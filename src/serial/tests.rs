use crate::serial::error::SerialError;
use crate::serial::{
    ConnectionConfig, ConnectionManager, DataBits, FlowControl, Parity, PortInfo, StopBits,
};

#[tokio::test]
async fn test_connection_manager_new() {
    let manager = ConnectionManager::new();
    let connections = manager.list().await;
    assert_eq!(connections.len(), 0);
}

#[tokio::test]
async fn test_connection_manager_open_invalid_port() {
    let manager = ConnectionManager::new();
    let config = ConnectionConfig {
        port: "INVALID_PORT_NAME".to_string(),
        baud_rate: 9600,
        data_bits: DataBits::Eight,
        stop_bits: StopBits::One,
        parity: Parity::None,
        flow_control: FlowControl::None,
    };

    let result = manager.open(config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_connection_manager_close_invalid_id() {
    let manager = ConnectionManager::new();
    let result = manager.close("invalid_id").await;
    assert!(result.is_err());

    match result {
        Err(SerialError::InvalidConnection(id)) => {
            assert_eq!(id, "invalid_id");
        }
        _ => panic!("Expected InvalidConnection error"),
    }
}

#[tokio::test]
async fn test_connection_manager_get_invalid_id() {
    let manager = ConnectionManager::new();
    let result = manager.get("invalid_id").await;
    assert!(result.is_err());
}

#[test]
fn test_data_bits_conversion() {
    assert_eq!(
        serialport::DataBits::from(DataBits::Five),
        serialport::DataBits::Five
    );
    assert_eq!(
        serialport::DataBits::from(DataBits::Six),
        serialport::DataBits::Six
    );
    assert_eq!(
        serialport::DataBits::from(DataBits::Seven),
        serialport::DataBits::Seven
    );
    assert_eq!(
        serialport::DataBits::from(DataBits::Eight),
        serialport::DataBits::Eight
    );
}

#[test]
fn test_stop_bits_conversion() {
    assert_eq!(
        serialport::StopBits::from(StopBits::One),
        serialport::StopBits::One
    );
    assert_eq!(
        serialport::StopBits::from(StopBits::Two),
        serialport::StopBits::Two
    );
}

#[test]
fn test_parity_conversion() {
    assert_eq!(
        serialport::Parity::from(Parity::None),
        serialport::Parity::None
    );
    assert_eq!(
        serialport::Parity::from(Parity::Odd),
        serialport::Parity::Odd
    );
    assert_eq!(
        serialport::Parity::from(Parity::Even),
        serialport::Parity::Even
    );
}

#[test]
fn test_flow_control_conversion() {
    assert_eq!(
        serialport::FlowControl::from(FlowControl::None),
        serialport::FlowControl::None
    );
    assert_eq!(
        serialport::FlowControl::from(FlowControl::Software),
        serialport::FlowControl::Software
    );
    assert_eq!(
        serialport::FlowControl::from(FlowControl::Hardware),
        serialport::FlowControl::Hardware
    );
}

#[test]
fn test_invalid_baud_rate() {
    use crate::serial::SerialConnection;

    let config = ConnectionConfig {
        port: "COM1".to_string(),
        baud_rate: 0,
        data_bits: DataBits::Eight,
        stop_bits: StopBits::One,
        parity: Parity::None,
        flow_control: FlowControl::None,
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(SerialConnection::new(config));

    assert!(result.is_err());
    match result {
        Err(SerialError::InvalidBaudRate(rate)) => {
            assert_eq!(rate, 0);
        }
        _ => panic!("Expected InvalidBaudRate error"),
    }
}

#[test]
fn test_error_display() {
    let err = SerialError::PortNotFound("COM99".to_string());
    assert_eq!(err.to_string(), "Port not found: COM99");

    let err = SerialError::ConnectionExists("COM1".to_string());
    assert_eq!(err.to_string(), "Connection already exists: COM1");

    let err = SerialError::InvalidBaudRate(999999999);
    assert_eq!(err.to_string(), "Invalid baud rate: 999999999");

    let err = SerialError::InvalidConnection("conn_123".to_string());
    assert_eq!(err.to_string(), "Invalid connection ID: conn_123");

    let err = SerialError::ReadTimeout;
    assert_eq!(err.to_string(), "Read timeout");

    let err = SerialError::ConnectionFailed("Test error".to_string());
    assert_eq!(err.to_string(), "Connection failed: Test error");

    let err = SerialError::InvalidConfig("Bad config".to_string());
    assert_eq!(err.to_string(), "Invalid configuration: Bad config");
}

// Mock tests for PortInfo - these would need actual serial ports to test properly
#[test]
fn test_port_info_list() {
    // This test will pass even if no ports are found
    let result = PortInfo::list_ports();
    assert!(result.is_ok());

    if let Ok(ports) = result {
        for port in ports {
            assert!(!port.name.is_empty());
            assert!(!port.description.is_empty());
            assert!(port.available);
        }
    }
}
