//! EndpointKit错误类型

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EndpointError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),
    
    #[error("Pool exhausted")]
    PoolExhausted,
    
    #[error("Connection timeout")]
    Timeout,
    
    #[error("TLS error: {0}")]
    Tls(String),
    
    #[error("Address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
    
    #[error("Pool error: {0}")]
    Pool(String),
    
    #[error("Paused by backpressure")]
    Paused,
}