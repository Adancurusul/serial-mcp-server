use base64::{engine::general_purpose, Engine as _};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{AutomationError, AutomationResult};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MacroPack {
    pub schema_version: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub macros: Vec<MacroDefinition>,
    #[serde(default)]
    pub assemblies: Vec<AssemblyDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MacroDefinition {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub steps: Vec<MacroStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssemblyDefinition {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub steps: Vec<AssemblyStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MacroStep {
    Send(SendStep),
    Delay(DelayStep),
    Expect(ExpectStep),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AssemblyStep {
    Macro(MacroCallStep),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MacroCallStep {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SendStep {
    pub data: String,
    #[serde(default)]
    pub encoding: Encoding,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DelayStep {
    pub ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectStep {
    pub op: ExpectOperation,
    pub data: String,
    #[serde(default)]
    pub encoding: Encoding,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_idle_ms")]
    pub idle_ms: u64,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub trim: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExpectOperation {
    Contains,
    Equals,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    #[serde(rename = "utf8", alias = "utf-8")]
    #[default]
    Utf8,
    Hex,
    Base64,
}

impl Encoding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Utf8 => "utf8",
            Self::Hex => "hex",
            Self::Base64 => "base64",
        }
    }

    pub fn decode(self, data: &str, field: &str) -> AutomationResult<Vec<u8>> {
        match self {
            Self::Utf8 => Ok(data.as_bytes().to_vec()),
            Self::Hex => {
                let clean_hex = data.replace(' ', "");
                hex::decode(clean_hex)
                    .map_err(|error| AutomationError::encoding(field, error.to_string()))
            }
            Self::Base64 => general_purpose::STANDARD
                .decode(data)
                .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(data))
                .map_err(|error| AutomationError::encoding(field, error.to_string())),
        }
    }

    pub fn encode(self, data: &[u8], field: &str) -> AutomationResult<String> {
        match self {
            Self::Utf8 => String::from_utf8(data.to_vec())
                .map_err(|error| AutomationError::encoding(field, error.to_string())),
            Self::Hex => Ok(hex::encode(data)),
            Self::Base64 => Ok(general_purpose::STANDARD.encode(data)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DataRecord {
    pub data: String,
    pub encoding: Encoding,
}

impl DataRecord {
    pub fn from_bytes(data: &[u8]) -> Self {
        match String::from_utf8(data.to_vec()) {
            Ok(text) => Self {
                data: text,
                encoding: Encoding::Utf8,
            },
            Err(_) => Self {
                data: hex::encode(data),
                encoding: Encoding::Hex,
            },
        }
    }
}

fn default_timeout_ms() -> u64 {
    1_000
}

fn default_idle_ms() -> u64 {
    50
}

fn default_max_bytes() -> usize {
    1_024
}
