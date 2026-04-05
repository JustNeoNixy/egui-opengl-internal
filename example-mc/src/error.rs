//! Centralized error handling for the example-mc project

use jni::errors::Error as JniError;
use std::fmt;

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, ExampleMcError>;

/// Main error type for the example-mc application
#[derive(Debug)]
pub enum ExampleMcError {
    /// JNI-related errors
    Jni(JniError),
    /// Failed to attach to JVM
    JvmAttachFailed,
    /// Failed to find Java class
    ClassNotFound(String),
    /// Failed to find Java method
    MethodNotFound(String),
    /// Failed to find Java field
    FieldNotFound(String),
    /// Invalid method signature
    InvalidSignature(String),
    /// Failed to create global reference
    GlobalRefFailed,
    /// Minecraft not initialized
    MinecraftNotInitialized,
    /// Entity not found
    EntityNotFound,
    /// Invalid entity state
    InvalidEntityState,
    /// ESP system not initialized
    EspNotInitialized,
    /// Feature not initialized
    FeatureNotInitialized(String),
    /// Invalid configuration
    InvalidConfiguration(String),
    /// IO errors
    Io(std::io::Error),
    /// Other errors with custom message
    Other(String),
}

impl fmt::Display for ExampleMcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExampleMcError::Jni(err) => write!(f, "JNI error: {}", err),
            ExampleMcError::JvmAttachFailed => write!(f, "Failed to attach to JVM"),
            ExampleMcError::ClassNotFound(class) => write!(f, "Class not found: {}", class),
            ExampleMcError::MethodNotFound(method) => write!(f, "Method not found: {}", method),
            ExampleMcError::FieldNotFound(field) => write!(f, "Field not found: {}", field),
            ExampleMcError::InvalidSignature(sig) => write!(f, "Invalid signature: {}", sig),
            ExampleMcError::GlobalRefFailed => write!(f, "Failed to create global reference"),
            ExampleMcError::MinecraftNotInitialized => write!(f, "Minecraft not initialized"),
            ExampleMcError::EntityNotFound => write!(f, "Entity not found"),
            ExampleMcError::InvalidEntityState => write!(f, "Invalid entity state"),
            ExampleMcError::EspNotInitialized => write!(f, "ESP system not initialized"),
            ExampleMcError::FeatureNotInitialized(feature) => {
                write!(f, "Feature not initialized: {}", feature)
            }
            ExampleMcError::InvalidConfiguration(config) => {
                write!(f, "Invalid configuration: {}", config)
            }
            ExampleMcError::Io(err) => write!(f, "IO error: {}", err),
            ExampleMcError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ExampleMcError {}

impl From<JniError> for ExampleMcError {
    fn from(err: JniError) -> Self {
        ExampleMcError::Jni(err)
    }
}

impl From<std::io::Error> for ExampleMcError {
    fn from(err: std::io::Error) -> Self {
        ExampleMcError::Io(err)
    }
}

/// Convenience macro for creating Other errors
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::error::ExampleMcError::Other(format!($($arg)*))
    };
}

/// Convenience macro for Result::Err with ExampleMcError
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::error::ExampleMcError::Other(format!($($arg)*)))
    };
}
