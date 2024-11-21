/// Error Handling Module
///
/// This module provides a comprehensive error handling system for the entire
/// application. It defines custom error types and implements conversions from
/// various error sources (IO, S3, etc.).
///
/// Error Types:
/// - Config: Configuration-related errors
/// - Io: File system operation errors
/// - Storage: Cloud storage errors (S3)
/// - NotImplemented: Features not yet available
/// - InvalidOperation: User input validation errors
///
/// The module provides:
/// 1. Custom Result type alias for consistent error handling
/// 2. Error type conversions (From implementations)
/// 3. Error formatting for user-friendly messages
/// 4. Error source tracking for debugging
///
/// Usage:
/// All public functions in the application should use the Result<T>
/// type alias defined in this module for consistent error handling.

use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, ToolError>;

#[derive(Debug)]
pub enum ToolError {
    Config(String),
    Io(std::io::Error),
    Storage(object_store::Error),
    NotImplemented(String),
    InvalidOperation(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ToolError::Io(err) => write!(f, "I/O error: {}", err),
            ToolError::Storage(err) => write!(f, "Storage error: {}", err),
            ToolError::NotImplemented(feature) => write!(f, "Feature not implemented: {}", feature),
            ToolError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl Error for ToolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ToolError::Io(err) => Some(err),
            ToolError::Storage(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::Io(err)
    }
}

impl From<object_store::Error> for ToolError {
    fn from(err: object_store::Error) -> Self {
        ToolError::Storage(err)
    }
}
