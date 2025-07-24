/*!
# Dynamic Driver Error Types

Defines error types for the dynamic driver system.
*/

use std::path::PathBuf;
use thiserror::Error;

/// Dynamic driver errors
#[derive(Error, Debug)]
pub enum DynamicDriverError {
    #[error("Library not found: {0:?}")]
    LibraryNotFound(PathBuf),
    
    #[error("Driver already registered: {0}")]
    DriverAlreadyRegistered(String),
    
    #[error("Driver not found: {0}")]
    DriverNotFound(String),
    
    #[error("Driver is disabled: {0}")]
    DriverDisabled(String),
    
    #[error("Directory not found: {0:?}")]
    DirectoryNotFound(PathBuf),
    
    #[error("Invalid ABI: {0}")]
    InvalidAbi(String),
    
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),
    
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Signature not found: {0:?}")]
    SignatureNotFound(PathBuf),
    
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    
    #[error("No trusted keys configured")]
    NoTrustedKeys,
    
    #[error("Insecure file permissions: {0:?}")]
    InsecurePermissions(PathBuf),
    
    #[error("Suspicious file: {0}")]
    SuspiciousFile(String),
    
    #[error("Hot swap operation failed: {0}")]
    HotSwapFailed(String),
    
    #[error("Driver initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Driver operation failed: {0}")]
    OperationFailed(String),
}