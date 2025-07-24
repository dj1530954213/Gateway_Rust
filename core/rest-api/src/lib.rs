/*!
# REST API Module

Provides RESTful API interface for command and control operations.
*/

pub mod server;
pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod error;

#[cfg(test)]
mod simple_test;

pub use server::*;
pub use auth::*;
pub use handlers::*;
pub use middleware::*;
pub use error::*;