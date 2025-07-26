/*!
# REST API Module

Provides RESTful API interface for command and control operations.
MVP-3: Includes RBAC and CP-0/CP-1 endpoints
*/

pub mod server;
pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod error;
pub mod rbac;
pub mod server_v2;

#[cfg(test)]
mod simple_test;

pub use server::*;
pub use auth::*;
pub use handlers::*;
pub use middleware::*;
pub use error::*;
pub use rbac::*;
pub use server_v2::*;

/// API Result type
pub type Result<T> = std::result::Result<T, ApiError>;

/// Common API Error types for RBAC and v2 server
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad Request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not Found: {0}")]
    NotFound(String),
    
    #[error("Internal Server Error: {0}")]
    Internal(String),
}