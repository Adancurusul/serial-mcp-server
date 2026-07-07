use async_trait::async_trait;
use clap::ValueEnum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::error::SerialError;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum CaptureStartTrigger {
    Immediate,
    #[default]
    FirstByte,
}

impl std::fmt::Display for CaptureStartTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate => f.write_str("immediate"),
            Self::FirstByte => f.write_str("first-byte"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaptureCompletionReason {
    DurationElapsed,
    InitialTimeout,
    IdleTimeout,
    MaxBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CaptureChunk {
    pub offset: usize,
    pub bytes_read: usize,
    pub waited_ms: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureConfig {
    pub timeout_ms: u64,
    pub max_bytes: usize,
    pub duration_ms: u64,
    pub start_trigger: CaptureStartTrigger,
    pub initial_timeout_ms: Option<u64>,
    pub idle_timeout_ms: Option<u64>,
}

impl CaptureConfig {
    pub fn validate(&self) -> Result<(), SerialError> {
        if self.timeout_ms == 0 {
            return Err(SerialError::InvalidConfig(
                "timeout_ms must be greater than zero".to_string(),
            ));
        }
        if self.max_bytes == 0 {
            return Err(SerialError::InvalidConfig(
                "max_bytes must be greater than zero".to_string(),
            ));
        }
        if self.duration_ms == 0 {
            return Err(SerialError::InvalidConfig(
                "duration_ms must be greater than zero".to_string(),
            ));
        }
        if matches!(self.initial_timeout_ms, Some(0)) {
            return Err(SerialError::InvalidConfig(
                "initial_timeout_ms must be greater than zero".to_string(),
            ));
        }
        if matches!(self.idle_timeout_ms, Some(0)) {
            return Err(SerialError::InvalidConfig(
                "idle_timeout_ms must be greater than zero".to_string(),
            ));
        }
        Ok(())
    }

    pub fn effective_initial_timeout_ms(&self) -> u64 {
        self.initial_timeout_ms.unwrap_or(self.timeout_ms)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureReport {
    pub data: Vec<u8>,
    pub chunks: Vec<CaptureChunk>,
    pub waited_ms: u64,
    pub elapsed_ms: u64,
    pub completion_reason: CaptureCompletionReason,
}

impl CaptureReport {
    pub fn bytes_read(&self) -> usize {
        self.data.len()
    }
}

#[async_trait]
pub trait CaptureReader {
    async fn read_once(&mut self, buffer: &mut [u8], timeout_ms: u64)
        -> Result<usize, SerialError>;
}

pub async fn capture_with_reader<R>(
    reader: &mut R,
    config: CaptureConfig,
) -> Result<CaptureReport, SerialError>
where
    R: CaptureReader + Send,
{
    config.validate()?;

    let overall_started = Instant::now();
    let mut capture_started = match config.start_trigger {
        CaptureStartTrigger::Immediate => Some(overall_started),
        CaptureStartTrigger::FirstByte => None,
    };
    let mut last_chunk_at: Option<Instant> = None;
    let mut data = Vec::new();
    let mut chunks = Vec::new();

    loop {
        if data.len() >= config.max_bytes {
            return Ok(report(
                data,
                chunks,
                overall_started,
                capture_started,
                CaptureCompletionReason::MaxBytes,
            ));
        }

        if let Some(started) = capture_started {
            if elapsed_ms(started) >= config.duration_ms {
                return Ok(report(
                    data,
                    chunks,
                    overall_started,
                    capture_started,
                    CaptureCompletionReason::DurationElapsed,
                ));
            }
            if let Some(idle_timeout_ms) = config.idle_timeout_ms {
                if let Some(last_chunk_at) = last_chunk_at {
                    if elapsed_ms(last_chunk_at) >= idle_timeout_ms {
                        return Ok(report(
                            data,
                            chunks,
                            overall_started,
                            capture_started,
                            CaptureCompletionReason::IdleTimeout,
                        ));
                    }
                }
            }
        } else if elapsed_ms(overall_started) >= config.effective_initial_timeout_ms() {
            return Ok(report(
                data,
                chunks,
                overall_started,
                capture_started,
                CaptureCompletionReason::InitialTimeout,
            ));
        }

        let timeout_ms = next_timeout_ms(&config, overall_started, capture_started, last_chunk_at);
        if timeout_ms == 0 {
            continue;
        }

        let remaining_bytes = config.max_bytes - data.len();
        let mut buffer = vec![0_u8; remaining_bytes];

        match reader.read_once(&mut buffer, timeout_ms).await {
            Ok(0) | Err(SerialError::ReadTimeout) => {}
            Ok(bytes_read) => {
                let now = Instant::now();
                if capture_started.is_none() {
                    capture_started = Some(now);
                }
                last_chunk_at = Some(now);
                buffer.truncate(bytes_read);
                let offset = data.len();
                data.extend_from_slice(&buffer);
                chunks.push(CaptureChunk {
                    offset,
                    bytes_read,
                    waited_ms: elapsed_ms(overall_started),
                    elapsed_ms: capture_started.map(elapsed_ms).unwrap_or(0),
                });
            }
            Err(error) => return Err(error),
        }
    }
}

fn next_timeout_ms(
    config: &CaptureConfig,
    overall_started: Instant,
    capture_started: Option<Instant>,
    last_chunk_at: Option<Instant>,
) -> u64 {
    let mut timeout_ms = config.timeout_ms;

    if let Some(started) = capture_started {
        timeout_ms = timeout_ms.min(remaining_ms(started, config.duration_ms));
        if let (Some(idle_timeout_ms), Some(last_chunk_at)) =
            (config.idle_timeout_ms, last_chunk_at)
        {
            timeout_ms = timeout_ms.min(remaining_ms(last_chunk_at, idle_timeout_ms));
        }
    } else {
        timeout_ms = timeout_ms.min(remaining_ms(
            overall_started,
            config.effective_initial_timeout_ms(),
        ));
    }

    timeout_ms
}

fn remaining_ms(started: Instant, limit_ms: u64) -> u64 {
    limit_ms.saturating_sub(elapsed_ms(started))
}

fn report(
    data: Vec<u8>,
    chunks: Vec<CaptureChunk>,
    overall_started: Instant,
    capture_started: Option<Instant>,
    completion_reason: CaptureCompletionReason,
) -> CaptureReport {
    CaptureReport {
        data,
        chunks,
        waited_ms: capture_started
            .map(|started| elapsed_between_ms(overall_started, started))
            .unwrap_or_else(|| elapsed_ms(overall_started)),
        elapsed_ms: capture_started.map(elapsed_ms).unwrap_or(0),
        completion_reason,
    }
}

fn elapsed_between_ms(started: Instant, ended: Instant) -> u64 {
    ended
        .duration_since(started)
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::time::Duration;

    struct ScriptedReader {
        chunks: VecDeque<Vec<u8>>,
    }

    impl ScriptedReader {
        fn new(chunks: Vec<&'static [u8]>) -> Self {
            Self {
                chunks: chunks.into_iter().map(Vec::from).collect(),
            }
        }
    }

    #[async_trait]
    impl CaptureReader for ScriptedReader {
        async fn read_once(
            &mut self,
            buffer: &mut [u8],
            timeout_ms: u64,
        ) -> Result<usize, SerialError> {
            let Some(mut chunk) = self.chunks.pop_front() else {
                tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
                return Err(SerialError::ReadTimeout);
            };

            if chunk.len() > buffer.len() {
                let remainder = chunk.split_off(buffer.len());
                self.chunks.push_front(remainder);
            }

            let bytes_read = chunk.len().min(buffer.len());
            buffer[..bytes_read].copy_from_slice(&chunk[..bytes_read]);
            Ok(bytes_read)
        }
    }

    fn config() -> CaptureConfig {
        CaptureConfig {
            timeout_ms: 1,
            max_bytes: 1024,
            duration_ms: 3,
            start_trigger: CaptureStartTrigger::Immediate,
            initial_timeout_ms: None,
            idle_timeout_ms: None,
        }
    }

    #[tokio::test]
    async fn immediate_capture_continues_after_first_chunk() {
        let mut reader = ScriptedReader::new(vec![b"one", b"two"]);
        let report = capture_with_reader(&mut reader, config()).await.unwrap();

        assert_eq!(report.data, b"onetwo");
        assert_eq!(
            report.completion_reason,
            CaptureCompletionReason::DurationElapsed
        );
        assert_eq!(report.chunks.len(), 2);
        assert_eq!(report.chunks[0].offset, 0);
        assert_eq!(report.chunks[1].offset, 3);
    }

    #[tokio::test]
    async fn first_byte_mode_reports_initial_timeout_without_data() {
        let mut reader = ScriptedReader::new(vec![]);
        let mut capture_config = config();
        capture_config.start_trigger = CaptureStartTrigger::FirstByte;
        capture_config.initial_timeout_ms = Some(2);

        let report = capture_with_reader(&mut reader, capture_config)
            .await
            .unwrap();

        assert_eq!(report.data, b"");
        assert_eq!(
            report.completion_reason,
            CaptureCompletionReason::InitialTimeout
        );
        assert_eq!(report.elapsed_ms, 0);
    }

    #[tokio::test]
    async fn idle_timeout_stops_after_capture_starts() {
        let mut reader = ScriptedReader::new(vec![b"one"]);
        let mut capture_config = config();
        capture_config.duration_ms = 50;
        capture_config.idle_timeout_ms = Some(2);

        let report = capture_with_reader(&mut reader, capture_config)
            .await
            .unwrap();

        assert_eq!(report.data, b"one");
        assert_eq!(
            report.completion_reason,
            CaptureCompletionReason::IdleTimeout
        );
    }

    #[tokio::test]
    async fn max_bytes_stops_and_truncates_chunk() {
        let mut reader = ScriptedReader::new(vec![b"abcdef"]);
        let mut capture_config = config();
        capture_config.max_bytes = 2;

        let report = capture_with_reader(&mut reader, capture_config)
            .await
            .unwrap();

        assert_eq!(report.data, b"ab");
        assert_eq!(report.bytes_read(), 2);
        assert_eq!(report.completion_reason, CaptureCompletionReason::MaxBytes);
        assert_eq!(report.chunks[0].bytes_read, 2);
    }
}
