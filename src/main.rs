//! Serial MCP Server - Main Entry Point
//!
//! A Model Context Protocol server for serial port communication.

use clap::Parser;
use tracing::{info, error, debug};
use tracing_subscriber::{EnvFilter, fmt};
use rmcp::{ServiceExt, transport::stdio};

use serial_mcp_server::{
    Config,
    config::Args,
    tools::SerialHandler,
    Result, SerialError,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Handle special flags first
    if args.generate_config {
        let config = Config::default();
        println!("{}", config.to_toml()?);
        return Ok(());
    }

    // Initialize logging
    init_logging(&args)?;

    info!("Starting Serial MCP Server v{}", env!("CARGO_PKG_VERSION"));
    debug!("Command line args: {:?}", args);

    // Load configuration
    let mut config = Config::load(args.config.as_ref())
        .map_err(|e| {
            error!("Failed to load configuration: {}", e);
            e
        })?;

    // Merge command line arguments into configuration
    config.merge_args(&args);

    if args.validate_config {
        config.validate()?;
        println!("Configuration is valid");
        return Ok(());
    }

    if args.show_config {
        println!("{}", config.to_toml()?);
        return Ok(());
    }

    // Validate final configuration
    config.validate()
        .map_err(|e| {
            error!("Configuration validation failed: {}", e);
            e
        })?;

    info!("Configuration loaded and validated successfully");
    info!("Server settings: max_connections={}, timeout={}s", 
          config.server.max_connections, 
          config.server.connection_timeout_seconds);
    info!("Serial settings: default_baud={}, buffer_size={}", 
          config.serial.default_baud_rate, 
          config.serial.max_buffer_size);

    // Create handler and get reference to connection manager for cleanup
    let handler = SerialHandler::new(config.clone());
    let connection_manager = handler.connection_manager();

    // Create and serve the handler using rust-sdk standard pattern
    let service = handler
        .serve(stdio()).await.map_err(|e| {
            error!("Serving error: {:?}", e);
            SerialError::InternalError(format!("Failed to start server: {}", e))
        })?;
    
    info!("Serial MCP Server started successfully");
    
    // Wait for the service to complete or for shutdown signal
    tokio::select! {
        result = service.waiting() => {
            if let Err(e) = result {
                // Log but don't treat as fatal - this often happens on clean shutdown
                debug!("Service ended: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
    }

    // Cleanup: close all open connections
    info!("Cleaning up resources...");
    let closed = connection_manager.close_all().await;
    if closed > 0 {
        info!("Closed {} open connection(s)", closed);
    }

    info!("Serial MCP Server stopped");
    Ok(())
}

/// Initialize logging system
fn init_logging(args: &Args) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(false)
        .with_line_number(false);

    // Configure output destination
    if let Some(log_file) = &args.log_file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
        
        subscriber
            .with_writer(file)
            .init();
        
        println!("Logging to file: {}", log_file.display());
    } else {
        subscriber
            .with_writer(std::io::stderr)
            .init();
    }

    debug!("Logging initialized with level: {}", args.log_level);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(&[
            "serial-mcp-rs",
            "--log-level", "debug",
            "--max-connections", "20",
            "--default-baud-rate", "9600",
        ]);
        
        assert_eq!(args.log_level, "debug");
        assert_eq!(args.max_connections, 20);
        assert_eq!(args.default_baud_rate, 9600);
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.server.max_connections, 10);
        assert_eq!(config.serial.default_baud_rate, 115200);
    }
}
