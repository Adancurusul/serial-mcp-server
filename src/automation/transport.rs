use std::collections::VecDeque;
use std::sync::Arc;

use async_trait::async_trait;

use super::{AutomationError, AutomationResult};
use crate::serial::{LocalSerialError, SerialConnection};

#[async_trait]
pub trait MacroTransport: Send {
    async fn write(&mut self, data: &[u8]) -> AutomationResult<usize>;
    async fn read(&mut self, max_bytes: usize, timeout_ms: u64) -> AutomationResult<Vec<u8>>;
}

pub struct SerialMacroTransport {
    connection: Arc<SerialConnection>,
}

impl SerialMacroTransport {
    pub fn new(connection: Arc<SerialConnection>) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl MacroTransport for SerialMacroTransport {
    async fn write(&mut self, data: &[u8]) -> AutomationResult<usize> {
        self.connection
            .write(data)
            .await
            .map_err(|error| AutomationError::Transport(error.to_string()))
    }

    async fn read(&mut self, max_bytes: usize, timeout_ms: u64) -> AutomationResult<Vec<u8>> {
        let mut buffer = vec![0_u8; max_bytes];
        match self.connection.read(&mut buffer, Some(timeout_ms)).await {
            Ok(bytes_read) => {
                buffer.truncate(bytes_read);
                Ok(buffer)
            }
            Err(LocalSerialError::ReadTimeout) => Ok(Vec::new()),
            Err(error) => Err(AutomationError::Transport(error.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimulatedMacroTransport {
    reads: VecDeque<Vec<u8>>,
    writes: Vec<Vec<u8>>,
}

impl SimulatedMacroTransport {
    pub fn new(reads: Vec<Vec<u8>>) -> Self {
        Self {
            reads: reads.into(),
            writes: Vec::new(),
        }
    }

    pub fn writes(&self) -> &[Vec<u8>] {
        &self.writes
    }
}

#[async_trait]
impl MacroTransport for SimulatedMacroTransport {
    async fn write(&mut self, data: &[u8]) -> AutomationResult<usize> {
        self.writes.push(data.to_vec());
        Ok(data.len())
    }

    async fn read(&mut self, max_bytes: usize, _timeout_ms: u64) -> AutomationResult<Vec<u8>> {
        let Some(mut data) = self.reads.pop_front() else {
            return Ok(Vec::new());
        };
        data.truncate(max_bytes);
        Ok(data)
    }
}
