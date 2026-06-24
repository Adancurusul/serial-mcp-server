//! MCP tool implementations
//!
//! Serial communication MCP tools using rust-sdk standard patterns

// Legacy implementations (keep for reference)
// pub mod serial_tools_simple;
// pub mod serial_tools_fixed;
// pub mod serial_tools_minimal;
// pub mod serial_tools_rmcp;
// pub mod serial_tools_rmcp_fixed;
// pub mod serial_tools_latest;
// pub mod serial_tools_working;

// Current implementation using rust-sdk standards
pub mod serial_handler;
pub mod types;

#[cfg(test)]
mod tests;

// Export the main handler and types
pub use serial_handler::*;
pub use types::*;
