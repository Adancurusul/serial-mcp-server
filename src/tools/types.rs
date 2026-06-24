use crate::serial::{ConnectionConfig, PortInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// 工具请求类型
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListPortsArgs {
    // 无参数
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenArgs {
    pub port: String,
    pub baud_rate: u32,
    #[serde(default = "default_data_bits")]
    pub data_bits: String,
    #[serde(default = "default_stop_bits")]
    pub stop_bits: String,
    #[serde(default = "default_parity")]
    pub parity: String,
    #[serde(default = "default_flow_control")]
    pub flow_control: String,
}

fn default_data_bits() -> String {
    "8".to_string()
}
fn default_stop_bits() -> String {
    "1".to_string()
}
fn default_parity() -> String {
    "none".to_string()
}
fn default_flow_control() -> String {
    "none".to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CloseArgs {
    pub connection_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteArgs {
    pub connection_id: String,
    pub data: String,
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

fn default_encoding() -> String {
    "utf8".to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadArgs {
    pub connection_id: String,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

fn default_max_bytes() -> usize {
    1024
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetControlLinesArgs {
    pub connection_id: String,
    #[serde(default)]
    pub rts: Option<bool>,
    #[serde(default)]
    pub dtr: Option<bool>,
}

impl SetControlLinesArgs {
    pub fn has_line_update(&self) -> bool {
        self.rts.is_some() || self.dtr.is_some()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConfigureArgs {
    pub connection_id: String,
    pub baud_rate: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StatusArgs {
    pub connection_id: String,
}

// 工具响应类型
#[derive(Debug, Serialize)]
pub struct PortsResponse {
    pub ports: Vec<PortInfo>,
}

#[derive(Debug, Serialize)]
pub struct OpenResponse {
    pub connection_id: String,
    pub status: String,
    pub port: String,
    pub baud_rate: u32,
    pub config: String,
}

#[derive(Debug, Serialize)]
pub struct CloseResponse {
    pub connection_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct WriteResponse {
    pub connection_id: String,
    pub bytes_written: usize,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct ReadResponse {
    pub connection_id: String,
    pub bytes_read: usize,
    pub data: String,
    pub encoding: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ConfigureResponse {
    pub connection_id: String,
    pub status: String,
    pub new_baud_rate: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub connection_id: String,
    pub port: String,
    pub baud_rate: u32,
    pub config: String,
    pub status: String,
    pub created_at: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

// 数据编码/解码工具函数
pub fn encode_data(data: &[u8], encoding: &str) -> Result<String, String> {
    match encoding.to_lowercase().as_str() {
        "utf8" | "utf-8" => {
            String::from_utf8(data.to_vec()).map_err(|e| format!("UTF-8 encoding error: {}", e))
        }
        "hex" => {
            let hex_string = hex::encode(data);
            // Add spaces between every two hex characters
            let spaced_hex = hex_string
                .chars()
                .collect::<Vec<char>>()
                .chunks(2)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect::<Vec<String>>()
                .join(" ");
            Ok(spaced_hex)
        }
        "base64" => {
            use base64::{engine::general_purpose, Engine};
            Ok(general_purpose::STANDARD.encode(data))
        }
        _ => Err(format!("Unsupported encoding: {}", encoding)),
    }
}

pub fn decode_data(data: &str, encoding: &str) -> Result<Vec<u8>, String> {
    match encoding.to_lowercase().as_str() {
        "utf8" | "utf-8" => Ok(data.as_bytes().to_vec()),
        "hex" => {
            // Remove spaces from hex string
            let clean_hex = data.replace(" ", "");
            hex::decode(clean_hex).map_err(|e| format!("Hex decoding error: {}", e))
        }
        "base64" => {
            use base64::{engine::general_purpose, Engine};
            // Try with standard padding first, then with URL_SAFE_NO_PAD if that fails
            general_purpose::STANDARD
                .decode(data)
                .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(data))
                .map_err(|e| format!("Base64 decoding error: {}", e))
        }
        _ => Err(format!("Unsupported encoding: {}", encoding)),
    }
}

impl From<OpenArgs> for ConnectionConfig {
    fn from(args: OpenArgs) -> Self {
        use crate::serial::{DataBits, FlowControl, Parity, StopBits};

        let data_bits = match args.data_bits.as_str() {
            "5" => DataBits::Five,
            "6" => DataBits::Six,
            "7" => DataBits::Seven,
            "8" => DataBits::Eight,
            _ => DataBits::Eight,
        };

        let stop_bits = match args.stop_bits.as_str() {
            "1" => StopBits::One,
            "2" => StopBits::Two,
            _ => StopBits::One,
        };

        let parity = match args.parity.to_lowercase().as_str() {
            "none" => Parity::None,
            "odd" => Parity::Odd,
            "even" => Parity::Even,
            _ => Parity::None,
        };

        let flow_control = match args.flow_control.to_lowercase().as_str() {
            "none" => FlowControl::None,
            "software" => FlowControl::Software,
            "hardware" => FlowControl::Hardware,
            _ => FlowControl::None,
        };

        ConnectionConfig {
            port: args.port,
            baud_rate: args.baud_rate,
            data_bits,
            stop_bits,
            parity,
            flow_control,
        }
    }
}
