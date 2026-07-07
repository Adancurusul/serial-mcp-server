//! Configuration management for the serial MCP server
//!
//! This module provides comprehensive configuration handling including command line
//! arguments, configuration files, validation, and logging setup.

use crate::error::{ConfigError, Result, SerialError};
use crate::serial::CaptureStartTrigger;
use clap::{ArgAction, Args as ClapArgs, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "serial-mcp-server")]
#[command(about = "A Model Context Protocol server for serial port communication")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, global = true)]
    pub log_level: Option<String>,

    /// Log file path
    #[arg(long, global = true)]
    pub log_file: Option<PathBuf>,

    /// Maximum number of concurrent connections
    #[arg(long, global = true)]
    pub max_connections: Option<usize>,

    /// Connection timeout in seconds
    #[arg(long, global = true)]
    pub connection_timeout: Option<u64>,

    /// Default baud rate for serial connections
    #[arg(long, global = true)]
    pub default_baud_rate: Option<u32>,

    /// Default timeout for operations in milliseconds
    #[arg(long, global = true)]
    pub default_timeout_ms: Option<u64>,

    /// Maximum buffer size in bytes
    #[arg(long, global = true)]
    pub max_buffer_size: Option<usize>,

    /// Connection retry count
    #[arg(long, global = true)]
    pub retry_count: Option<u32>,

    /// Enable auto-discovery of serial ports
    #[arg(
        long,
        global = true,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true"
    )]
    pub auto_discovery: Option<bool>,

    /// Allow multiple connections to the same port
    #[arg(
        long,
        global = true,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true"
    )]
    pub allow_port_sharing: Option<bool>,

    /// Restrict port access to specific patterns
    #[arg(
        long,
        global = true,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true"
    )]
    pub restrict_ports: Option<bool>,

    /// Generate default configuration file
    #[arg(long)]
    pub generate_config: bool,

    /// Validate configuration and exit
    #[arg(long)]
    pub validate_config: bool,

    /// Show current configuration and exit
    #[arg(long)]
    pub show_config: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Start the MCP stdio server
    Serve,
    /// List available serial ports
    ListPorts(OutputArgs),
    /// Open and close a serial port to verify it is reachable
    Probe(SerialPortArgs),
    /// Write data to a serial port, optionally reading a response
    Write(WriteCommand),
    /// Read data from a serial port
    Read(ReadCommand),
    /// Set RTS and/or DTR control line levels
    SetControlLines(SetControlLinesCommand),
    /// Validate, plan, and run JSON macro packs
    #[command(subcommand)]
    Macro(MacroCommand),
    /// Generate default configuration TOML
    GenerateConfig,
    /// Validate configuration and exit
    ValidateConfig,
    /// Show the merged configuration and exit
    ShowConfig,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct OutputArgs {
    /// Emit machine-readable JSON on stdout
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct SerialPortArgs {
    /// Serial port path or name, such as COM19 or /dev/ttyUSB0
    #[arg(long)]
    pub port: String,

    /// Baud rate
    #[arg(long)]
    pub baud: Option<u32>,

    /// Data bits
    #[arg(long)]
    pub data_bits: Option<u8>,

    /// Stop bits: 1 or 2
    #[arg(long)]
    pub stop_bits: Option<String>,

    /// Parity: none, odd, or even
    #[arg(long)]
    pub parity: Option<String>,

    /// Flow control: none, software, or hardware
    #[arg(long)]
    pub flow_control: Option<String>,

    /// Emit machine-readable JSON on stdout
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct WriteCommand {
    #[command(flatten)]
    pub serial: SerialPortArgs,

    /// Data to write
    #[arg(long)]
    pub data: String,

    /// Data encoding
    #[arg(long, value_enum, default_value_t = CliDataFormat::Utf8)]
    pub format: CliDataFormat,

    /// Read a response after writing
    #[arg(long)]
    pub read: bool,

    /// Read timeout in milliseconds when --read is used
    #[arg(long)]
    pub timeout_ms: Option<u64>,

    /// Maximum bytes to read when --read is used
    #[arg(long, default_value_t = 1024)]
    pub max_bytes: usize,

    #[command(flatten)]
    pub capture: CaptureWindowArgs,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct ReadCommand {
    #[command(flatten)]
    pub serial: SerialPortArgs,

    /// Data encoding for stdout payload
    #[arg(long, value_enum, default_value_t = CliDataFormat::Utf8)]
    pub format: CliDataFormat,

    /// Read timeout in milliseconds
    #[arg(long)]
    pub timeout_ms: Option<u64>,

    /// Maximum bytes to read
    #[arg(long, default_value_t = 1024)]
    pub max_bytes: usize,

    #[command(flatten)]
    pub capture: CaptureWindowArgs,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct CaptureWindowArgs {
    /// Collect data for this many milliseconds instead of returning after one read
    #[arg(long)]
    pub duration_ms: Option<u64>,

    /// When a capture window starts
    #[arg(long, value_enum, default_value_t = CaptureStartTrigger::FirstByte)]
    pub start_trigger: CaptureStartTrigger,

    /// Maximum milliseconds to wait for the first byte in first-byte mode
    #[arg(long)]
    pub initial_timeout_ms: Option<u64>,

    /// Stop capture after this many quiet milliseconds once data has arrived
    #[arg(long)]
    pub idle_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct SetControlLinesCommand {
    #[command(flatten)]
    pub serial: SerialPortArgs,

    /// RTS line level
    #[arg(long, value_enum)]
    pub rts: Option<ControlLineLevel>,

    /// DTR line level
    #[arg(long, value_enum)]
    pub dtr: Option<ControlLineLevel>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum MacroCommand {
    /// Validate a JSON macro pack
    Validate(MacroFileCommand),
    /// List macros and assemblies in a JSON macro pack
    List(MacroFileCommand),
    /// Expand a macro or assembly into a hardware-free plan
    Plan(MacroPlanCommand),
    /// Run a macro or assembly against real serial hardware, dry-run, or simulation
    Run(MacroRunCommand),
}

#[derive(Debug, Clone, ClapArgs)]
pub struct MacroFileCommand {
    /// JSON macro pack file
    #[arg(long)]
    pub file: PathBuf,

    /// Emit machine-readable JSON on stdout
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct MacroPlanCommand {
    /// JSON macro pack file
    #[arg(long)]
    pub file: PathBuf,

    /// Macro name to plan
    #[arg(long = "macro", conflicts_with = "assembly")]
    pub macro_name: Option<String>,

    /// Assembly name to plan
    #[arg(long, conflicts_with = "macro_name")]
    pub assembly: Option<String>,

    /// Emit machine-readable JSON on stdout
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct MacroRunCommand {
    #[command(flatten)]
    pub serial: OptionalSerialPortArgs,

    /// JSON macro pack file
    #[arg(long)]
    pub file: PathBuf,

    /// Macro name to run
    #[arg(long = "macro", conflicts_with = "assembly")]
    pub macro_name: Option<String>,

    /// Assembly name to run
    #[arg(long, conflicts_with = "macro_name")]
    pub assembly: Option<String>,

    /// Return the expanded plan without opening serial hardware
    #[arg(long)]
    pub dry_run: bool,

    /// Simulated UTF-8 read chunk. Repeat for multiple reads.
    #[arg(long = "simulate-read", action = ArgAction::Append)]
    pub simulate_read: Vec<String>,

    /// Emit machine-readable JSON on stdout
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct OptionalSerialPortArgs {
    /// Serial port path or name, such as COM19 or /dev/ttyUSB0
    #[arg(long)]
    pub port: Option<String>,

    /// Baud rate
    #[arg(long)]
    pub baud: Option<u32>,

    /// Data bits
    #[arg(long)]
    pub data_bits: Option<u8>,

    /// Stop bits: 1 or 2
    #[arg(long)]
    pub stop_bits: Option<String>,

    /// Parity: none, odd, or even
    #[arg(long)]
    pub parity: Option<String>,

    /// Flow control: none, software, or hardware
    #[arg(long)]
    pub flow_control: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum CliDataFormat {
    Utf8,
    Hex,
    Base64,
}

#[derive(Debug, Clone, Copy, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ControlLineLevel {
    High,
    Low,
}

impl ControlLineLevel {
    pub fn as_bool(self) -> bool {
        matches!(self, ControlLineLevel::High)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ControlLineLevel::High => "high",
            ControlLineLevel::Low => "low",
        }
    }
}

/// Main configuration structure
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub server: ServerConfig,
    pub serial: SerialConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

impl Config {
    /// Load configuration from file or create default
    pub fn load(config_path: Option<&PathBuf>) -> Result<Self> {
        if let Some(path) = config_path {
            let content = std::fs::read_to_string(path).map_err(|e| {
                SerialError::InvalidConfig(format!("Failed to read config file: {}", e))
            })?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| SerialError::InvalidConfig(format!("Invalid TOML syntax: {}", e)))?;
            config.validate()?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Merge command line arguments into configuration
    pub fn merge_args(&mut self, args: &Args) {
        if let Some(max_connections) = args.max_connections {
            self.server.max_connections = max_connections;
        }
        if let Some(connection_timeout) = args.connection_timeout {
            self.server.connection_timeout_seconds = connection_timeout;
        }
        if let Some(default_baud_rate) = args.default_baud_rate {
            self.serial.default_baud_rate = default_baud_rate;
        }
        if let Some(default_timeout_ms) = args.default_timeout_ms {
            self.serial.default_timeout_ms = default_timeout_ms;
        }
        if let Some(max_buffer_size) = args.max_buffer_size {
            self.serial.max_buffer_size = max_buffer_size;
        }
        if let Some(retry_count) = args.retry_count {
            self.serial.retry_count = retry_count;
        }
        if let Some(auto_discovery) = args.auto_discovery {
            self.serial.auto_discovery = auto_discovery;
        }
        if let Some(allow_port_sharing) = args.allow_port_sharing {
            self.serial.allow_port_sharing = allow_port_sharing;
        }
        if let Some(restrict_ports) = args.restrict_ports {
            self.security.restrict_ports = restrict_ports;
        }
        if let Some(log_level) = &args.log_level {
            self.logging.level = log_level.clone();
        }
        if let Some(log_file) = &args.log_file {
            self.logging.file = Some(log_file.clone());
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Server validation
        if self.server.max_connections == 0 {
            return Err(ConfigError::InvalidValue {
                field: "server.max_connections".to_string(),
                value: "0".to_string(),
            }
            .into());
        }

        if self.server.max_connections > 1000 {
            return Err(ConfigError::ValueOutOfRange {
                field: "server.max_connections".to_string(),
                value: self.server.max_connections.to_string(),
                min: "1".to_string(),
                max: "1000".to_string(),
            }
            .into());
        }

        // Serial validation
        if self.serial.default_baud_rate == 0 {
            return Err(ConfigError::InvalidValue {
                field: "serial.default_baud_rate".to_string(),
                value: "0".to_string(),
            }
            .into());
        }

        let valid_baud_rates = [
            300, 600, 1200, 2400, 4800, 9600, 14400, 19200, 28800, 38400, 57600, 115200, 230400,
            460800, 921600,
        ];
        if !valid_baud_rates.contains(&self.serial.default_baud_rate) {
            return Err(ConfigError::InvalidValue {
                field: "serial.default_baud_rate".to_string(),
                value: self.serial.default_baud_rate.to_string(),
            }
            .into());
        }

        if self.serial.max_buffer_size == 0 {
            return Err(ConfigError::InvalidValue {
                field: "serial.max_buffer_size".to_string(),
                value: "0".to_string(),
            }
            .into());
        }

        if self.serial.max_buffer_size > 1024 * 1024 {
            return Err(ConfigError::ValueOutOfRange {
                field: "serial.max_buffer_size".to_string(),
                value: self.serial.max_buffer_size.to_string(),
                min: "1".to_string(),
                max: "1048576".to_string(),
            }
            .into());
        }

        // Logging validation
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::InvalidValue {
                field: "logging.level".to_string(),
                value: self.logging.level.clone(),
            }
            .into());
        }

        Ok(())
    }

    /// Generate TOML configuration string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| SerialError::InvalidConfig(format!("Failed to serialize config: {}", e)))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub max_connections: usize,
    pub connection_timeout_seconds: u64,
    pub worker_threads: Option<usize>,
    pub enable_metrics: bool,
    pub metrics_interval_seconds: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            connection_timeout_seconds: 30,
            worker_threads: None,
            enable_metrics: false,
            metrics_interval_seconds: 60,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerialConfig {
    pub default_baud_rate: u32,
    pub default_data_bits: u8,
    pub default_stop_bits: String,
    pub default_parity: String,
    pub default_flow_control: String,
    pub default_timeout_ms: u64,
    pub max_buffer_size: usize,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
    pub auto_discovery: bool,
    pub discovery_interval_seconds: u64,
    pub allow_port_sharing: bool,
    pub default_line_ending: String,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            default_baud_rate: 115200,
            default_data_bits: 8,
            default_stop_bits: "One".to_string(),
            default_parity: "None".to_string(),
            default_flow_control: "None".to_string(),
            default_timeout_ms: 1000,
            max_buffer_size: 8192,
            retry_count: 3,
            retry_delay_ms: 1000,
            auto_discovery: false,
            discovery_interval_seconds: 5,
            allow_port_sharing: false,
            default_line_ending: "\n".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub restrict_ports: bool,
    pub allowed_ports: Vec<String>,
    pub blocked_ports: Vec<String>,
    pub max_data_size: usize,
    pub rate_limit_enabled: bool,
    pub rate_limit_requests_per_second: u32,
    pub enable_authentication: bool,
    pub allowed_clients: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            restrict_ports: false,
            allowed_ports: vec![],
            blocked_ports: vec![],
            max_data_size: 65536, // 64KB
            rate_limit_enabled: false,
            rate_limit_requests_per_second: 100,
            enable_authentication: false,
            allowed_clients: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<PathBuf>,
    pub format: String,
    pub timestamp_format: String,
    pub include_location: bool,
    pub include_thread_names: bool,
    pub rotate_logs: bool,
    pub max_log_files: usize,
    pub max_log_size_mb: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            format: "text".to_string(),
            timestamp_format: "rfc3339".to_string(),
            include_location: false,
            include_thread_names: false,
            rotate_logs: false,
            max_log_files: 10,
            max_log_size_mb: 10,
        }
    }
}
