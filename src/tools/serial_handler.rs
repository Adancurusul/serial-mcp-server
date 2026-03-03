//! Serial MCP Handler using rust-sdk standard approach
//! 
//! This implementation follows the official rust-sdk patterns for proper tool registration

use std::sync::Arc;
use std::future::Future;
use rmcp::{
    tool, tool_handler, tool_router, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    ErrorData as McpError,
    service::RequestContext,
    RoleServer,
};
use tracing::{debug, error, info};

use crate::serial::{PortInfo, ConnectionManager};
use crate::config::Config;
use super::types::*;

/// Serial tool handler using rust-sdk standard patterns
#[derive(Clone)]
pub struct SerialHandler {
    connection_manager: Arc<ConnectionManager>,
    #[allow(dead_code)]
    config: Config,
    tool_router: ToolRouter<SerialHandler>,
}

#[tool_router]
impl SerialHandler {
    pub fn new(config: Config) -> Self {
        Self {
            connection_manager: Arc::new(ConnectionManager::new()),
            config,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "List all available serial ports on the system")]
    async fn list_ports(&self) -> Result<CallToolResult, McpError> {
        debug!("Listing available serial ports");
        
        match PortInfo::list_ports() {
            Ok(ports) => {
                info!("Found {} serial ports", ports.len());
                
                let message = if ports.is_empty() {
                    "No serial ports found on the system".to_string()
                } else {
                    let port_list = ports
                        .iter()
                        .map(|p| {
                            if let Some(ref hw_id) = p.hardware_id {
                                format!("- {}: {} ({})", p.name, p.description, hw_id)
                            } else {
                                format!("- {}: {}", p.name, p.description)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    
                    format!("Found {} serial ports:\n{}", ports.len(), port_list)
                };
                
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            Err(e) => {
                error!("Failed to list serial ports: {}", e);
                Err(McpError::internal_error(format!("Failed to list ports: {}", e), None))
            }
        }
    }

    #[tool(description = "Open a serial port connection with specified configuration")]
    async fn open(&self, Parameters(args): Parameters<OpenArgs>) -> Result<CallToolResult, McpError> {
        debug!("Opening serial connection to {}", args.port);
        
        let config: crate::serial::ConnectionConfig = args.into();
        
        match self.connection_manager.open(config.clone()).await {
            Ok(connection_id) => {
                info!("Opened serial connection {} to {}", connection_id, config.port);
                
                let message = format!(
                    "Serial connection opened\nConnection ID: {}\nPort: {}\nBaud rate: {}",
                    connection_id, config.port, config.baud_rate
                );
                
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            Err(e) => {
                error!("Failed to open serial connection to {}: {}", config.port, e);
                let error_msg = format!("Error: Failed to open port {} - {}", config.port, e);
                Err(McpError::internal_error(error_msg, None))
            }
        }
    }

    #[tool(description = "Close an open serial port connection")]
    async fn close(&self, Parameters(args): Parameters<CloseArgs>) -> Result<CallToolResult, McpError> {
        debug!("Closing serial connection {}", args.connection_id);
        
        match self.connection_manager.close(&args.connection_id).await {
            Ok(()) => {
                info!("Closed serial connection {}", args.connection_id);
                let message = format!("Serial connection closed\nConnection ID: {}", args.connection_id);
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            Err(e) => {
                error!("Failed to close connection {}: {}", args.connection_id, e);
                let error_msg = format!("Error: Failed to close connection {} - {}", args.connection_id, e);
                Err(McpError::internal_error(error_msg, None))
            }
        }
    }

    #[tool(description = "Write data to a serial port connection")]
    async fn write(&self, Parameters(args): Parameters<WriteArgs>) -> Result<CallToolResult, McpError> {
        debug!("Writing to connection {} with encoding {}", args.connection_id, args.encoding);
        
        // Get connection
        let connection = match self.connection_manager.get(&args.connection_id).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Invalid connection ID {}: {}", args.connection_id, e);
                let error_msg = format!("Error: Connection ID {} not found", args.connection_id);
                return Err(McpError::internal_error(error_msg, None));
            }
        };
        
        // Decode data
        let data = match decode_data(&args.data, &args.encoding) {
            Ok(data) => data,
            Err(e) => {  
                error!("Failed to decode data with encoding {}: {}", args.encoding, e);
                let error_msg = format!("Error: Data decoding failed - {}", e);
                return Err(McpError::internal_error(error_msg, None));
            }
        };
        
        // Send data
        match connection.write(&data).await {
            Ok(bytes_written) => {
                debug!("Wrote {} bytes to connection {}", bytes_written, args.connection_id);
                let message = format!(
                    "Data sent successfully\nConnection ID: {}\nBytes written: {}\nData: {:?}",
                    args.connection_id, bytes_written, args.data
                );
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            Err(e) => {
                error!("Failed to write to connection {}: {}", args.connection_id, e);
                let error_msg = format!("Error: Data sending failed - {}", e);
                Err(McpError::internal_error(error_msg, None))
            }
        }
    }

    #[tool(description = "Set RTS and/or DTR control line levels on a serial port connection")]
    async fn set_control_lines(&self, Parameters(args): Parameters<SetControlLinesArgs>) -> Result<CallToolResult, McpError> {
        debug!("Setting control lines on connection {}: rts={:?} dtr={:?}", args.connection_id, args.rts, args.dtr);

        if args.rts.is_none() && args.dtr.is_none() {
            return Err(McpError::internal_error("At least one of 'rts' or 'dtr' must be specified".to_string(), None));
        }

        let connection = match self.connection_manager.get(&args.connection_id).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Invalid connection ID {}: {}", args.connection_id, e);
                return Err(McpError::internal_error(format!("Error: Connection ID {} not found", args.connection_id), None));
            }
        };

        if let Some(rts) = args.rts {
            if let Err(e) = connection.set_rts(rts).await {
                error!("Failed to set RTS on connection {}: {}", args.connection_id, e);
                return Err(McpError::internal_error(format!("Error: Failed to set RTS - {}", e), None));
            }
        }

        if let Some(dtr) = args.dtr {
            if let Err(e) = connection.set_dtr(dtr).await {
                error!("Failed to set DTR on connection {}: {}", args.connection_id, e);
                return Err(McpError::internal_error(format!("Error: Failed to set DTR - {}", e), None));
            }
        }

        let mut parts = vec![format!("Control lines updated\nConnection ID: {}", args.connection_id)];
        if let Some(rts) = args.rts {
            parts.push(format!("RTS: {}", if rts { "high" } else { "low" }));
        }
        if let Some(dtr) = args.dtr {
            parts.push(format!("DTR: {}", if dtr { "high" } else { "low" }));
        }

        Ok(CallToolResult::success(vec![Content::text(parts.join("\n"))]))
    }

    #[tool(description = "Read data from a serial port connection")]
    async fn read(&self, Parameters(args): Parameters<ReadArgs>) -> Result<CallToolResult, McpError> {
        debug!("Reading from connection {} with timeout {:?}", args.connection_id, args.timeout_ms);
        
        // Get connection
        let connection = match self.connection_manager.get(&args.connection_id).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Invalid connection ID {}: {}", args.connection_id, e);
                let error_msg = format!("Error: Connection ID {} not found", args.connection_id);
                return Err(McpError::internal_error(error_msg, None));
            }
        };
        
        // Prepare buffer
        let mut buffer = vec![0u8; args.max_bytes];
        
        // Read data
        match connection.read(&mut buffer, args.timeout_ms).await {
            Ok(bytes_read) => {
                buffer.truncate(bytes_read);
                
                // Encode data
                match encode_data(&buffer, &args.encoding) {
                    Ok(encoded) => {
                        debug!("Read {} bytes from connection {}", bytes_read, args.connection_id);
                        
                        let message = if bytes_read > 0 {
                            format!(
                                "Data read successfully\nConnection ID: {}\nBytes read: {}\nData: {:?}",
                                args.connection_id, bytes_read, encoded
                            )
                        } else {
                            format!(
                                "Read timeout\nConnection ID: {}\nTimeout: {}ms\nBytes read: 0",
                                args.connection_id, args.timeout_ms.unwrap_or(1000)
                            )
                        };
                        
                        Ok(CallToolResult::success(vec![Content::text(message)]))
                    }
                    Err(e) => {
                        error!("Failed to encode read data: {}", e);
                        let error_msg = format!("Error: Data encoding failed - {}", e);
                        Err(McpError::internal_error(error_msg, None))
                    }
                }
            }
            Err(e) => {
                match e {
                    crate::serial::LocalSerialError::ReadTimeout => {
                        debug!("Read timeout on connection {}", args.connection_id);
                        let message = format!(
                            "Read timeout\nConnection ID: {}\nTimeout: {}ms\nBytes read: 0",
                            args.connection_id, args.timeout_ms.unwrap_or(1000)
                        );
                        Ok(CallToolResult::success(vec![Content::text(message)]))
                    }
                    _ => {
                        error!("Failed to read from connection {}: {}", args.connection_id, e);
                        let error_msg = format!("Error: Data reading failed - {}", e);
                        Err(McpError::internal_error(error_msg, None))
                    }
                }
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for SerialHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("A serial port communication MCP server. Use list_ports to discover available serial ports, then open connections to communicate with serial devices.".to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("Serial MCP server initialized");
        Ok(self.get_info())
    }
}

/// Decode data to bytes array
fn decode_data(data: &str, encoding: &str) -> Result<Vec<u8>, String> {
    match encoding {
        "utf8" | "utf-8" => Ok(data.as_bytes().to_vec()),
        "hex" => {
            let data = data.trim().replace(' ', "");
            if data.len() % 2 != 0 {
                return Err("Hex string must have even length".to_string());
            }
            
            (0..data.len())
                .step_by(2)
                .map(|i| {
                    u8::from_str_radix(&data[i..i+2], 16)
                        .map_err(|_| format!("Invalid hex character at position {}", i))
                })
                .collect()
        }
        "base64" => {
            use base64::{Engine as _, engine::general_purpose};
            general_purpose::STANDARD
                .decode(data.trim())
                .map_err(|e| format!("Invalid base64: {}", e))
        }
        _ => Err(format!("Unsupported encoding: {}", encoding)),
    }
}

/// Encode bytes array to string
fn encode_data(data: &[u8], encoding: &str) -> Result<String, String> {
    match encoding {
        "utf8" | "utf-8" => {
            String::from_utf8(data.to_vec())
                .map_err(|e| format!("Invalid UTF-8: {}", e))
        }
        "hex" => {
            Ok(data.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" "))
        }
        "base64" => {
            use base64::{Engine as _, engine::general_purpose};
            Ok(general_purpose::STANDARD.encode(data))
        }
        _ => Err(format!("Unsupported encoding: {}", encoding)),
    }
}