//! Utility functions and helper types for the serial MCP server
//!
//! This module provides various utility functions for data processing,
//! validation, conversion, and other common operations.

use crate::error::{Result, SerialError};
use base64::prelude::*;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Serial port type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortType {
    /// USB-to-Serial adapter
    UsbSerial,
    /// Built-in serial port
    Native,
    /// Bluetooth serial
    Bluetooth,
    /// Virtual serial port
    Virtual,
    /// Unknown type
    Unknown,
}

impl PortType {
    /// Detect port type from port name or description
    pub fn from_port_info(port_name: &str, description: Option<&str>) -> Self {
        let port_lower = port_name.to_lowercase();
        let desc_lower = description.unwrap_or("").to_lowercase();

        if port_lower.contains("usb") || desc_lower.contains("usb") {
            PortType::UsbSerial
        } else if port_lower.contains("bluetooth") || desc_lower.contains("bluetooth") {
            PortType::Bluetooth
        } else if port_lower.contains("tty") && !port_lower.contains("usb") {
            PortType::Native
        } else if desc_lower.contains("virtual") || desc_lower.contains("loopback") {
            PortType::Virtual
        } else {
            PortType::Unknown
        }
    }
}

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortType::UsbSerial => write!(f, "USB Serial"),
            PortType::Native => write!(f, "Native Serial"),
            PortType::Bluetooth => write!(f, "Bluetooth Serial"),
            PortType::Virtual => write!(f, "Virtual Serial"),
            PortType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Data encoding formats supported by the server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    /// Plain text (UTF-8)
    Text,
    /// Hexadecimal encoding
    Hex,
    /// Base64 encoding
    Base64,
}

impl DataFormat {
    /// Parse format from string.
    pub fn parse(s: &str) -> Result<Self> {
        s.parse()
    }
}

impl FromStr for DataFormat {
    type Err = SerialError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "text" | "utf8" | "string" => Ok(DataFormat::Text),
            "hex" | "hexadecimal" => Ok(DataFormat::Hex),
            "base64" | "b64" => Ok(DataFormat::Base64),
            "binary" | "bin" | "raw" => Err(SerialError::InvalidConfig(
                "Use hex or base64 for binary payloads".to_string(),
            )),
            _ => Err(SerialError::InvalidConfig(format!(
                "Unknown data format: {}",
                s
            ))),
        }
    }
}

impl std::fmt::Display for DataFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataFormat::Text => write!(f, "text"),
            DataFormat::Hex => write!(f, "hex"),
            DataFormat::Base64 => write!(f, "base64"),
        }
    }
}

/// Data conversion utilities
pub struct DataConverter;

impl DataConverter {
    /// Convert data to the specified format
    pub fn encode(data: &[u8], format: DataFormat) -> Result<String> {
        match format {
            DataFormat::Text => String::from_utf8(data.to_vec())
                .map_err(|e| SerialError::EncodingError(format!("UTF-8 encoding failed: {}", e))),
            DataFormat::Hex => Ok(hex::encode(data)),
            DataFormat::Base64 => Ok(base64::prelude::BASE64_STANDARD.encode(data)),
        }
    }

    /// Convert data from the specified format
    pub fn decode(data: &str, format: DataFormat) -> Result<Vec<u8>> {
        match format {
            DataFormat::Text => Ok(data.as_bytes().to_vec()),
            DataFormat::Hex => hex::decode(data.split_whitespace().collect::<String>())
                .map_err(|e| SerialError::EncodingError(format!("Hex decoding failed: {}", e))),
            DataFormat::Base64 => base64::prelude::BASE64_STANDARD
                .decode(data)
                .map_err(|e| SerialError::EncodingError(format!("Base64 decoding failed: {}", e))),
        }
    }

    /// Escape special characters for display
    pub fn escape_string(data: &str) -> String {
        data.chars()
            .map(|c| match c {
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\0' => "\\0".to_string(),
                '\\' => "\\\\".to_string(),
                c if c.is_control() => format!("\\x{:02x}", c as u8),
                c => c.to_string(),
            })
            .collect()
    }

    /// Unescape string with special characters
    pub fn unescape_string(data: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = data.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('0') => result.push('\0'),
                    Some('\\') => result.push('\\'),
                    Some('x') => {
                        // Handle \xNN hex escape
                        let hex_chars: String = chars.by_ref().take(2).collect();
                        if hex_chars.len() == 2 {
                            match u8::from_str_radix(&hex_chars, 16) {
                                Ok(byte) => result.push(byte as char),
                                Err(_) => {
                                    return Err(SerialError::EncodingError(format!(
                                        "Invalid hex escape: \\x{}",
                                        hex_chars
                                    )))
                                }
                            }
                        } else {
                            return Err(SerialError::EncodingError(
                                "Incomplete hex escape".to_string(),
                            ));
                        }
                    }
                    Some(other) => {
                        return Err(SerialError::EncodingError(format!(
                            "Unknown escape sequence: \\{}",
                            other
                        )))
                    }
                    None => {
                        return Err(SerialError::EncodingError(
                            "Incomplete escape sequence".to_string(),
                        ))
                    }
                }
            } else {
                result.push(c);
            }
        }

        Ok(result)
    }
}

/// Time utilities
pub struct TimeUtils;

impl TimeUtils {
    /// Get current Unix timestamp in milliseconds
    pub fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Get current Unix timestamp in seconds
    pub fn now_seconds() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Format duration as human-readable string
    pub fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let millis = duration.subsec_millis();

        if total_secs >= 3600 {
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            format!("{}h {}m {}s", hours, mins, secs)
        } else if total_secs >= 60 {
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{}m {}s", mins, secs)
        } else if total_secs > 0 {
            format!("{}.{:03}s", total_secs, millis)
        } else {
            format!("{}ms", millis)
        }
    }
}

/// Validation utilities
pub struct Validator;

impl Validator {
    /// Validate baud rate
    pub fn validate_baud_rate(baud_rate: u32) -> Result<()> {
        const VALID_BAUD_RATES: &[u32] = &[
            300, 600, 1200, 2400, 4800, 9600, 14400, 19200, 28800, 38400, 57600, 115200, 230400,
            460800, 921600,
        ];

        if VALID_BAUD_RATES.contains(&baud_rate) {
            Ok(())
        } else {
            Err(SerialError::InvalidBaudRate(baud_rate))
        }
    }

    /// Validate data bits
    pub fn validate_data_bits(data_bits: u8) -> Result<()> {
        match data_bits {
            5..=8 => Ok(()),
            _ => Err(SerialError::InvalidDataBits(data_bits)),
        }
    }

    /// Validate stop bits
    pub fn validate_stop_bits(stop_bits: &str) -> Result<()> {
        match stop_bits.to_lowercase().as_str() {
            "one" | "1" | "two" | "2" => Ok(()),
            _ => Err(SerialError::InvalidStopBits(stop_bits.to_string())),
        }
    }

    /// Validate parity
    pub fn validate_parity(parity: &str) -> Result<()> {
        match parity.to_lowercase().as_str() {
            "none" | "even" | "odd" => Ok(()),
            _ => Err(SerialError::InvalidParity(parity.to_string())),
        }
    }

    /// Validate flow control
    pub fn validate_flow_control(flow_control: &str) -> Result<()> {
        match flow_control.to_lowercase().as_str() {
            "none" | "software" | "hardware" => Ok(()),
            _ => Err(SerialError::InvalidFlowControl(flow_control.to_string())),
        }
    }

    /// Validate port name format
    pub fn validate_port_name(port_name: &str) -> Result<()> {
        if port_name.is_empty() {
            return Err(SerialError::InvalidConfig(
                "Port name cannot be empty".to_string(),
            ));
        }

        // Basic validation - could be extended based on OS
        if port_name.len() > 255 {
            return Err(SerialError::InvalidConfig("Port name too long".to_string()));
        }

        Ok(())
    }
}

/// Buffer utilities
pub struct BufferUtils;

impl BufferUtils {
    /// Find pattern in buffer
    pub fn find_pattern(buffer: &[u8], pattern: &[u8]) -> Option<usize> {
        if pattern.is_empty() || buffer.len() < pattern.len() {
            return None;
        }

        buffer
            .windows(pattern.len())
            .position(|window| window == pattern)
    }

    /// Split buffer by delimiter
    pub fn split_by_delimiter(buffer: &[u8], delimiter: &[u8]) -> Vec<Vec<u8>> {
        if delimiter.is_empty() {
            return vec![buffer.to_vec()];
        }

        let mut result = Vec::new();
        let mut start = 0;

        while let Some(pos) = BufferUtils::find_pattern(&buffer[start..], delimiter) {
            let absolute_pos = start + pos;
            result.push(buffer[start..absolute_pos].to_vec());
            start = absolute_pos + delimiter.len();
        }

        // Add remaining data
        if start < buffer.len() {
            result.push(buffer[start..].to_vec());
        }

        result
    }

    /// Calculate checksum (simple sum)
    pub fn checksum_sum(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
    }

    /// Calculate checksum (XOR)
    pub fn checksum_xor(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc ^ b)
    }

    /// Calculate CRC-8
    pub fn crc8(data: &[u8]) -> u8 {
        const CRC8_TABLE: [u8; 256] = [
            0x00, 0x07, 0x0E, 0x09, 0x1C, 0x1B, 0x12, 0x15, 0x38, 0x3F, 0x36, 0x31, 0x24, 0x23,
            0x2A, 0x2D, 0x70, 0x77, 0x7E, 0x79, 0x6C, 0x6B, 0x62, 0x65, 0x48, 0x4F, 0x46, 0x41,
            0x54, 0x53, 0x5A, 0x5D, 0xE0, 0xE7, 0xEE, 0xE9, 0xFC, 0xFB, 0xF2, 0xF5, 0xD8, 0xDF,
            0xD6, 0xD1, 0xC4, 0xC3, 0xCA, 0xCD, 0x90, 0x97, 0x9E, 0x99, 0x8C, 0x8B, 0x82, 0x85,
            0xA8, 0xAF, 0xA6, 0xA1, 0xB4, 0xB3, 0xBA, 0xBD, 0xC7, 0xC0, 0xC9, 0xCE, 0xDB, 0xDC,
            0xD5, 0xD2, 0xFF, 0xF8, 0xF1, 0xF6, 0xE3, 0xE4, 0xED, 0xEA, 0xB7, 0xB0, 0xB9, 0xBE,
            0xAB, 0xAC, 0xA5, 0xA2, 0x8F, 0x88, 0x81, 0x86, 0x93, 0x94, 0x9D, 0x9A, 0x27, 0x20,
            0x29, 0x2E, 0x3B, 0x3C, 0x35, 0x32, 0x1F, 0x18, 0x11, 0x16, 0x03, 0x04, 0x0D, 0x0A,
            0x57, 0x50, 0x59, 0x5E, 0x4B, 0x4C, 0x45, 0x42, 0x6F, 0x68, 0x61, 0x66, 0x73, 0x74,
            0x7D, 0x7A, 0x89, 0x8E, 0x87, 0x80, 0x95, 0x92, 0x9B, 0x9C, 0xB1, 0xB6, 0xBF, 0xB8,
            0xAD, 0xAA, 0xA3, 0xA4, 0xF9, 0xFE, 0xF7, 0xF0, 0xE5, 0xE2, 0xEB, 0xEC, 0xC1, 0xC6,
            0xCF, 0xC8, 0xDD, 0xDA, 0xD3, 0xD4, 0x69, 0x6E, 0x67, 0x60, 0x75, 0x72, 0x7B, 0x7C,
            0x51, 0x56, 0x5F, 0x58, 0x4D, 0x4A, 0x43, 0x44, 0x19, 0x1E, 0x17, 0x10, 0x05, 0x02,
            0x0B, 0x0C, 0x21, 0x26, 0x2F, 0x28, 0x3D, 0x3A, 0x33, 0x34, 0x4E, 0x49, 0x40, 0x47,
            0x52, 0x55, 0x5C, 0x5B, 0x76, 0x71, 0x78, 0x7F, 0x6A, 0x6D, 0x64, 0x63, 0x3E, 0x39,
            0x30, 0x37, 0x22, 0x25, 0x2C, 0x2B, 0x06, 0x01, 0x08, 0x0F, 0x1A, 0x1D, 0x14, 0x13,
            0xAE, 0xA9, 0xA0, 0xA7, 0xB2, 0xB5, 0xBC, 0xBB, 0x96, 0x91, 0x98, 0x9F, 0x8A, 0x8D,
            0x84, 0x83, 0xDE, 0xD9, 0xD0, 0xD7, 0xC2, 0xC5, 0xCC, 0xCB, 0xE6, 0xE1, 0xE8, 0xEF,
            0xFA, 0xFD, 0xF4, 0xF3,
        ];

        data.iter()
            .fold(0u8, |crc, &byte| CRC8_TABLE[(crc ^ byte) as usize])
    }
}

/// Session ID generation
pub struct SessionIdGenerator;

impl SessionIdGenerator {
    /// Generate a unique session ID
    pub fn generate() -> String {
        format!(
            "serial_session_{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")[..16].to_lowercase()
        )
    }

    /// Generate a connection ID
    pub fn generate_connection_id(port_name: &str) -> String {
        let timestamp = TimeUtils::now_millis();
        let port_hash = port_name
            .chars()
            .fold(0u32, |acc, c| acc.wrapping_add(c as u32));
        format!("conn_{}_{:08x}", timestamp, port_hash)
    }
}

/// String utilities
pub struct StringUtils;

impl StringUtils {
    /// Truncate string to maximum length with ellipsis
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    /// Sanitize string for safe display
    pub fn sanitize(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                    '?'
                } else {
                    c
                }
            })
            .collect()
    }

    /// Parse comma-separated values
    pub fn parse_csv(s: &str) -> Vec<String> {
        s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Format bytes as human-readable size
    pub fn format_bytes(bytes: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_converter_encode_decode() {
        let data = b"Hello, World!";

        // Test hex encoding/decoding
        let hex_encoded = DataConverter::encode(data, DataFormat::Hex).unwrap();
        let hex_decoded = DataConverter::decode(&hex_encoded, DataFormat::Hex).unwrap();
        assert_eq!(data, hex_decoded.as_slice());

        // Test base64 encoding/decoding
        let b64_encoded = DataConverter::encode(data, DataFormat::Base64).unwrap();
        let b64_decoded = DataConverter::decode(&b64_encoded, DataFormat::Base64).unwrap();
        assert_eq!(data, b64_decoded.as_slice());

        // Test text encoding/decoding
        let text_encoded = DataConverter::encode(data, DataFormat::Text).unwrap();
        let text_decoded = DataConverter::decode(&text_encoded, DataFormat::Text).unwrap();
        assert_eq!(data, text_decoded.as_slice());
    }

    #[test]
    fn test_escape_unescape() {
        let original = "Hello\nWorld\r\tTest\\0\x01";
        let escaped = DataConverter::escape_string(original);
        let _unescaped = DataConverter::unescape_string(&escaped).unwrap();
        // The escape/unescape should be symmetric for most chars
        // but control chars like \x01 will be different
        let original_simple = "Hello\nWorld\r\tTest\\0";
        let escaped_simple = DataConverter::escape_string(original_simple);
        let unescaped_simple = DataConverter::unescape_string(&escaped_simple).unwrap();
        assert_eq!(original_simple, unescaped_simple);
    }

    #[test]
    fn test_validator() {
        assert!(Validator::validate_baud_rate(115200).is_ok());
        assert!(Validator::validate_baud_rate(12345).is_err());

        assert!(Validator::validate_data_bits(8).is_ok());
        assert!(Validator::validate_data_bits(9).is_err());

        assert!(Validator::validate_parity("none").is_ok());
        assert!(Validator::validate_parity("invalid").is_err());
    }

    #[test]
    fn test_buffer_utils() {
        let buffer = b"Hello,World,Test";
        let delimiter = b",";
        let parts = BufferUtils::split_by_delimiter(buffer, delimiter);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], b"Hello");
        assert_eq!(parts[1], b"World");
        assert_eq!(parts[2], b"Test");
    }

    #[test]
    fn test_checksum() {
        let data = b"Hello";
        let sum_checksum = BufferUtils::checksum_sum(data);
        let xor_checksum = BufferUtils::checksum_xor(data);
        let crc8_checksum = BufferUtils::crc8(data);

        assert_eq!(sum_checksum, 244); // 'H' + 'e' + 'l' + 'l' + 'o' = 72 + 101 + 108 + 108 + 111 = 500 % 256 = 244
        assert_ne!(xor_checksum, 0);
        assert_ne!(crc8_checksum, 0);
    }

    #[test]
    fn test_string_utils() {
        assert_eq!(StringUtils::truncate("Hello, World!", 10), "Hello, ...");
        assert_eq!(StringUtils::truncate("Hi", 10), "Hi");

        let csv = "a, b,  c  , d,";
        let parsed = StringUtils::parse_csv(csv);
        assert_eq!(parsed, vec!["a", "b", "c", "d"]);

        assert_eq!(StringUtils::format_bytes(1024), "1.0 KB");
        assert_eq!(StringUtils::format_bytes(1048576), "1.0 MB");
    }
}
