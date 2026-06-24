pub mod connection;
pub mod error;
pub mod port;

#[cfg(test)]
mod tests;

pub use connection::{
    ConnectionConfig, ConnectionStatus, DataBits, FlowControl, Parity, SerialConnection, StopBits,
};
pub use error::SerialError as LocalSerialError;
pub use port::PortInfo;

use crate::error::SerialError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConnectParams<'a> {
    pub port_name: &'a str,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: &'a str,
    pub parity: &'a str,
    pub flow_control: &'a str,
    pub timeout_ms: u64,
}

#[derive(Debug)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, Arc<SerialConnection>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to a serial port with individual parameters (for compatibility with session manager)
    pub async fn connect(
        &self,
        params: ConnectParams<'_>,
    ) -> Result<SerialConnection, SerialError> {
        use connection::{DataBits, FlowControl, Parity, StopBits};

        let data_bits = match params.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            8 => DataBits::Eight,
            _ => return Err(SerialError::InvalidDataBits(params.data_bits)),
        };

        let stop_bits = match params.stop_bits.to_lowercase().as_str() {
            "one" | "1" => StopBits::One,
            "two" | "2" => StopBits::Two,
            _ => return Err(SerialError::InvalidStopBits(params.stop_bits.to_string())),
        };

        let parity = match params.parity.to_lowercase().as_str() {
            "none" => Parity::None,
            "even" => Parity::Even,
            "odd" => Parity::Odd,
            _ => return Err(SerialError::InvalidParity(params.parity.to_string())),
        };

        let flow_control = match params.flow_control.to_lowercase().as_str() {
            "none" => FlowControl::None,
            "software" => FlowControl::Software,
            "hardware" => FlowControl::Hardware,
            _ => {
                return Err(SerialError::InvalidFlowControl(
                    params.flow_control.to_string(),
                ))
            }
        };

        let config = ConnectionConfig {
            port: params.port_name.to_string(),
            baud_rate: params.baud_rate,
            data_bits,
            stop_bits,
            parity,
            flow_control,
        };

        SerialConnection::new(config)
            .await
            .map_err(|e| SerialError::ConnectionFailed(e.to_string()))
    }

    pub async fn open(&self, config: ConnectionConfig) -> Result<String, LocalSerialError> {
        let connection = Arc::new(SerialConnection::new(config.clone()).await?);
        let id = connection.id().to_string();

        let mut connections = self.connections.write().await;

        // Check if port is already in use
        for (_, conn) in connections.iter() {
            if conn.status().await.port == config.port {
                return Err(LocalSerialError::ConnectionExists(config.port));
            }
        }

        connections.insert(id.clone(), connection);
        Ok(id)
    }

    pub async fn close(&self, id: &str) -> Result<(), LocalSerialError> {
        let mut connections = self.connections.write().await;
        connections
            .remove(id)
            .ok_or_else(|| LocalSerialError::InvalidConnection(id.to_string()))?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Arc<SerialConnection>, LocalSerialError> {
        let connections = self.connections.read().await;
        connections
            .get(id)
            .cloned()
            .ok_or_else(|| LocalSerialError::InvalidConnection(id.to_string()))
    }

    pub async fn list(&self) -> Vec<ConnectionStatus> {
        let connections = self.connections.read().await;
        let mut statuses = Vec::new();

        for connection in connections.values() {
            statuses.push(connection.status().await);
        }

        statuses
    }

    /// Close all open connections. Used during graceful shutdown.
    pub async fn close_all(&self) -> usize {
        let mut connections = self.connections.write().await;
        let count = connections.len();
        connections.clear();
        count
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
