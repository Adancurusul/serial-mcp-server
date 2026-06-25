use std::time::{Duration, Instant};

use serde::Serialize;

use super::{
    AutomationError, AutomationResult, DataRecord, ExpectOperation, MacroPlan, MacroTransport,
    PlanStep,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockMode {
    System,
    Noop,
}

#[derive(Debug, Clone)]
pub struct MacroExecutor {
    clock_mode: ClockMode,
    mode: &'static str,
}

impl MacroExecutor {
    pub fn real() -> Self {
        Self {
            clock_mode: ClockMode::System,
            mode: "real",
        }
    }

    pub fn simulated() -> Self {
        Self {
            clock_mode: ClockMode::Noop,
            mode: "simulation",
        }
    }

    pub async fn run<T: MacroTransport>(
        &self,
        plan: MacroPlan,
        mut transport: T,
    ) -> AutomationResult<RunReport> {
        let mut report = RunReport::new(&plan, self.mode);

        for (index, step) in plan.steps.iter().enumerate() {
            match step {
                PlanStep::Send { data, encoding, .. } => {
                    let bytes = encoding.decode(data, &format!("steps[{index}].data"))?;
                    let bytes_written = transport.write(&bytes).await?;
                    report.writes.push(DataRecord::from_bytes(&bytes));
                    report
                        .steps
                        .push(StepReport::ok(index, "send", bytes_written, 0));
                }
                PlanStep::Delay { ms, .. } => {
                    self.delay(*ms).await;
                    report.steps.push(StepReport::ok(index, "delay", 0, 0));
                }
                PlanStep::Expect {
                    op,
                    data,
                    encoding,
                    timeout_ms,
                    idle_ms,
                    max_bytes,
                    trim,
                    ..
                } => {
                    let expected = encoding.decode(data, &format!("steps[{index}].data"))?;
                    let read = self
                        .expect_bytes(
                            &mut transport,
                            ExpectConfig {
                                index,
                                op: *op,
                                expected: &expected,
                                timeout_ms: *timeout_ms,
                                idle_ms: *idle_ms,
                                max_bytes: *max_bytes,
                                trim: *trim,
                            },
                        )
                        .await?;
                    report.reads.push(DataRecord::from_bytes(&read));
                    report
                        .steps
                        .push(StepReport::ok(index, "expect", 0, read.len()));
                }
            }
        }

        report.success = true;
        Ok(report)
    }

    async fn delay(&self, ms: u64) {
        if self.clock_mode == ClockMode::System {
            tokio::time::sleep(Duration::from_millis(ms)).await;
        }
    }

    async fn expect_bytes<T: MacroTransport>(
        &self,
        transport: &mut T,
        config: ExpectConfig<'_>,
    ) -> AutomationResult<Vec<u8>> {
        match config.op {
            ExpectOperation::Contains => read_until_contains(transport, config).await,
            ExpectOperation::Equals => read_until_equals(transport, config).await,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunReport {
    pub pack_name: String,
    pub target_kind: String,
    pub target_name: String,
    pub mode: String,
    pub success: bool,
    pub steps: Vec<StepReport>,
    pub writes: Vec<DataRecord>,
    pub reads: Vec<DataRecord>,
}

impl RunReport {
    fn new(plan: &MacroPlan, mode: &str) -> Self {
        Self {
            pack_name: plan.pack_name.clone(),
            target_kind: plan.target_kind.clone(),
            target_name: plan.target_name.clone(),
            mode: mode.to_string(),
            success: false,
            steps: Vec::with_capacity(plan.steps.len()),
            writes: Vec::new(),
            reads: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StepReport {
    pub index: usize,
    pub kind: String,
    pub status: String,
    pub bytes_written: usize,
    pub bytes_read: usize,
}

impl StepReport {
    fn ok(index: usize, kind: &str, bytes_written: usize, bytes_read: usize) -> Self {
        Self {
            index,
            kind: kind.to_string(),
            status: "ok".to_string(),
            bytes_written,
            bytes_read,
        }
    }
}

struct ExpectConfig<'a> {
    index: usize,
    op: ExpectOperation,
    expected: &'a [u8],
    timeout_ms: u64,
    idle_ms: u64,
    max_bytes: usize,
    trim: bool,
}

async fn read_until_contains<T: MacroTransport>(
    transport: &mut T,
    config: ExpectConfig<'_>,
) -> AutomationResult<Vec<u8>> {
    let mut accumulated = Vec::new();
    let started = Instant::now();

    while accumulated.len() < config.max_bytes && elapsed_ms(started) < config.timeout_ms {
        let remaining = config.timeout_ms - elapsed_ms(started);
        let max_read = config.max_bytes - accumulated.len();
        let chunk = transport.read(max_read, remaining).await?;
        if chunk.is_empty() {
            break;
        }
        accumulated.extend_from_slice(&chunk);
        if contains_bytes(&accumulated, config.expected) {
            return Ok(accumulated);
        }
    }

    Err(AutomationError::execution(
        config.index,
        "expected bytes were not contained in the response",
    ))
}

async fn read_until_equals<T: MacroTransport>(
    transport: &mut T,
    config: ExpectConfig<'_>,
) -> AutomationResult<Vec<u8>> {
    let mut accumulated = Vec::new();
    let started = Instant::now();

    while accumulated.len() < config.max_bytes && elapsed_ms(started) < config.timeout_ms {
        let remaining = config.timeout_ms - elapsed_ms(started);
        let per_read_timeout = if accumulated.is_empty() {
            remaining
        } else {
            config.idle_ms.min(remaining)
        };
        let max_read = config.max_bytes - accumulated.len();
        let chunk = transport.read(max_read, per_read_timeout).await?;
        if chunk.is_empty() {
            break;
        }
        accumulated.extend_from_slice(&chunk);
    }

    if bytes_equal(&accumulated, config.expected, config.trim) {
        return Ok(accumulated);
    }

    Err(AutomationError::execution(
        config.index,
        "complete response did not equal the expected bytes",
    ))
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn bytes_equal(actual: &[u8], expected: &[u8], trim: bool) -> bool {
    if trim {
        return trim_ascii(actual) == trim_ascii(expected);
    }
    actual == expected
}

fn trim_ascii(data: &[u8]) -> &[u8] {
    let start = data
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(0);
    let end = data
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|index| index + 1)
        .unwrap_or(start);
    &data[start..end]
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}
