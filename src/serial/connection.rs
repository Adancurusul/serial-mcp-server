use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use uuid::Uuid;

use super::error::SerialError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DataBits {
    #[serde(rename = "5")]
    Five,
    #[serde(rename = "6")]
    Six,
    #[serde(rename = "7")]
    Seven,
    #[serde(rename = "8")]
    Eight,
}

impl From<DataBits> for serialport::DataBits {
    fn from(bits: DataBits) -> Self {
        match bits {
            DataBits::Five => serialport::DataBits::Five,
            DataBits::Six => serialport::DataBits::Six,
            DataBits::Seven => serialport::DataBits::Seven,
            DataBits::Eight => serialport::DataBits::Eight,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StopBits {
    #[serde(rename = "1")]
    One,
    #[serde(rename = "2")]
    Two,
}

impl From<StopBits> for serialport::StopBits {
    fn from(bits: StopBits) -> Self {
        match bits {
            StopBits::One => serialport::StopBits::One,
            StopBits::Two => serialport::StopBits::Two,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Parity {
    None,
    Odd,
    Even,
}

impl From<Parity> for serialport::Parity {
    fn from(parity: Parity) -> Self {
        match parity {
            Parity::None => serialport::Parity::None,
            Parity::Odd => serialport::Parity::Odd,
            Parity::Even => serialport::Parity::Even,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowControl {
    None,
    Software,
    Hardware,
}

impl From<FlowControl> for serialport::FlowControl {
    fn from(flow: FlowControl) -> Self {
        match flow {
            FlowControl::None => serialport::FlowControl::None,
            FlowControl::Software => serialport::FlowControl::Software,
            FlowControl::Hardware => serialport::FlowControl::Hardware,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub port: String,
    pub baud_rate: u32,
    #[serde(default = "default_data_bits")]
    pub data_bits: DataBits,
    #[serde(default = "default_stop_bits")]
    pub stop_bits: StopBits,
    #[serde(default = "default_parity")]
    pub parity: Parity,
    #[serde(default = "default_flow_control")]
    pub flow_control: FlowControl,
}

fn default_data_bits() -> DataBits {
    DataBits::Eight
}
fn default_stop_bits() -> StopBits {
    StopBits::One
}
fn default_parity() -> Parity {
    Parity::None
}
fn default_flow_control() -> FlowControl {
    FlowControl::None
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatus {
    pub id: String,
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub connected: bool,
    pub created_at: DateTime<Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Debug)]
pub struct SerialConnection {
    id: String,
    config: ConnectionConfig,
    stream: Arc<Mutex<SerialStream>>,
    created_at: DateTime<Utc>,
    bytes_sent: Arc<Mutex<u64>>,
    bytes_received: Arc<Mutex<u64>>,
}

impl SerialConnection {
    pub async fn new(config: ConnectionConfig) -> Result<Self, SerialError> {
        // Validate baud rate
        if config.baud_rate == 0 || config.baud_rate > 4_000_000 {
            return Err(SerialError::InvalidBaudRate(config.baud_rate));
        }

        // Build serial port
        let builder = tokio_serial::new(&config.port, config.baud_rate)
            .data_bits(config.data_bits.into())
            .stop_bits(config.stop_bits.into())
            .parity(config.parity.into())
            .flow_control(config.flow_control.into());

        // Open the port
        let stream = builder
            .open_native_async()
            .map_err(|e| SerialError::ConnectionFailed(format!("{}: {}", config.port, e)))?;

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            config,
            stream: Arc::new(Mutex::new(stream)),
            created_at: Utc::now(),
            bytes_sent: Arc::new(Mutex::new(0)),
            bytes_received: Arc::new(Mutex::new(0)),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn write(&self, data: &[u8]) -> Result<usize, SerialError> {
        use tokio::io::AsyncWriteExt;

        let mut stream = self.stream.lock().await;
        let written = stream.write(data).await?;
        stream.flush().await?;

        let mut sent = self.bytes_sent.lock().await;
        *sent += written as u64;

        Ok(written)
    }

    pub async fn read(
        &self,
        buffer: &mut [u8],
        timeout_ms: Option<u64>,
    ) -> Result<usize, SerialError> {
        use tokio::io::AsyncReadExt;

        let mut stream = self.stream.lock().await;

        let read_result = if let Some(ms) = timeout_ms {
            match timeout(Duration::from_millis(ms), stream.read(buffer)).await {
                Ok(result) => result,
                Err(_) => return Err(SerialError::ReadTimeout),
            }
        } else {
            stream.read(buffer).await
        };

        let bytes_read = read_result?;

        let mut received = self.bytes_received.lock().await;
        *received += bytes_read as u64;

        Ok(bytes_read)
    }

    pub async fn status(&self) -> ConnectionStatus {
        ConnectionStatus {
            id: self.id.clone(),
            port: self.config.port.clone(),
            baud_rate: self.config.baud_rate,
            data_bits: self.config.data_bits,
            stop_bits: self.config.stop_bits,
            parity: self.config.parity,
            flow_control: self.config.flow_control,
            connected: true,
            created_at: self.created_at,
            bytes_sent: *self.bytes_sent.lock().await,
            bytes_received: *self.bytes_received.lock().await,
        }
    }

    pub async fn set_rts(&self, level: bool) -> Result<(), SerialError> {
        use serialport::SerialPort;
        let mut stream = self.stream.lock().await;
        stream.write_request_to_send(level)?;
        Ok(())
    }

    pub async fn set_dtr(&self, level: bool) -> Result<(), SerialError> {
        use serialport::SerialPort;
        let mut stream = self.stream.lock().await;
        stream.write_data_terminal_ready(level)?;
        Ok(())
    }

    pub async fn reconfigure(&self, new_baud_rate: Option<u32>) -> Result<(), SerialError> {
        if let Some(baud_rate) = new_baud_rate {
            if baud_rate == 0 || baud_rate > 4_000_000 {
                return Err(SerialError::InvalidBaudRate(baud_rate));
            }

            let stream = self.stream.lock().await;
            // Note: tokio-serial doesn't support runtime reconfiguration
            // This would require closing and reopening the port
            drop(stream);

            return Err(SerialError::InvalidConfig(
                "Runtime reconfiguration not supported. Please close and reopen the connection."
                    .to_string(),
            ));
        }

        Ok(())
    }
}
