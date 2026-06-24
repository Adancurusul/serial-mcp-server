use thiserror::Error;

pub type AutomationResult<T> = Result<T, AutomationError>;

#[derive(Debug, Error)]
pub enum AutomationError {
    #[error("invalid macro pack at {field}: {message}")]
    Validation { field: String, message: String },

    #[error("unknown macro target: {0}")]
    UnknownMacro(String),

    #[error("unknown assembly target: {0}")]
    UnknownAssembly(String),

    #[error("macro pack JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("macro pack I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("data encoding error at {field}: {message}")]
    Encoding { field: String, message: String },

    #[error("serial transport error: {0}")]
    Transport(String),

    #[error("macro execution failed at step {step_index}: {message}")]
    Execution { step_index: usize, message: String },
}

impl AutomationError {
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn encoding(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Encoding {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn execution(step_index: usize, message: impl Into<String>) -> Self {
        Self::Execution {
            step_index,
            message: message.into(),
        }
    }
}
