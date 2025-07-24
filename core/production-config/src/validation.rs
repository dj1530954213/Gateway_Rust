/*!
# Configuration Validation

Custom validation functions for production configuration.
*/

use std::net::SocketAddr;
use validator::ValidationError;

/// Validate socket address format
pub fn validate_socket_addr(addr: &str) -> Result<(), ValidationError> {
    match addr.parse::<SocketAddr>() {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("invalid_socket_address")),
    }
}

/// Validate log level
pub fn validate_log_level(level: &str) -> Result<(), ValidationError> {
    match level.to_lowercase().as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => Ok(()),
        _ => Err(ValidationError::new("invalid_log_level")),
    }
}

/// Validate log format
pub fn validate_log_format(format: &str) -> Result<(), ValidationError> {
    match format.to_lowercase().as_str() {
        "json" | "text" | "compact" => Ok(()),
        _ => Err(ValidationError::new("invalid_log_format")),
    }
}

/// Validate environment name
pub fn validate_environment(env: &str) -> Result<(), ValidationError> {
    match env.to_lowercase().as_str() {
        "development" | "dev" | "staging" | "stage" | "production" | "prod" => Ok(()),
        _ => Err(ValidationError::new("invalid_environment")),
    }
}

/// Validate JWT secret strength
pub fn validate_jwt_secret(secret: &str) -> Result<(), ValidationError> {
    if secret.len() < 32 {
        return Err(ValidationError::new("jwt_secret_too_short"));
    }
    
    // Check for base64 encoding
    use base64::{Engine, engine::general_purpose::STANDARD};
    if STANDARD.decode(secret).is_err() {
        return Err(ValidationError::new("jwt_secret_invalid_base64"));
    }
    
    Ok(())
}

/// Validate driver type
pub fn validate_driver_type(driver_type: &str) -> Result<(), ValidationError> {
    match driver_type.to_lowercase().as_str() {
        "modbus" | "opcua" | "mqtt" | "http" | "serial" | "tcp" | "custom" => Ok(()),
        _ => Err(ValidationError::new("unsupported_driver_type")),
    }
}

/// Validate connector type
pub fn validate_connector_type(connector_type: &str) -> Result<(), ValidationError> {
    match connector_type.to_lowercase().as_str() {
        "mqtt5" | "mqtt" | "http" | "websocket" | "tcp" | "udp" | "custom" => Ok(()),
        _ => Err(ValidationError::new("unsupported_connector_type")),
    }
}

/// Validate percentage value
pub fn validate_percentage(value: f32) -> Result<(), ValidationError> {
    if value < 0.0 || value > 100.0 {
        return Err(ValidationError::new("invalid_percentage"));
    }
    Ok(())
}

/// Validate port number
pub fn validate_port(port: u16) -> Result<(), ValidationError> {
    if port == 0 {
        return Err(ValidationError::new("invalid_port_zero"));
    }
    
    // Check for well-known system ports
    if port < 1024 {
        return Err(ValidationError::new("system_port_warning"));
    }
    
    Ok(())
}

/// Validate timeout value in seconds
pub fn validate_timeout_seconds(timeout: u32) -> Result<(), ValidationError> {
    if timeout == 0 {
        return Err(ValidationError::new("timeout_cannot_be_zero"));
    }
    
    if timeout > 3600 {
        return Err(ValidationError::new("timeout_too_large"));
    }
    
    Ok(())
}

/// Validate interval in seconds
pub fn validate_interval_seconds(interval: u32) -> Result<(), ValidationError> {
    if interval == 0 {
        return Err(ValidationError::new("interval_cannot_be_zero"));
    }
    
    if interval > 86400 {
        return Err(ValidationError::new("interval_too_large"));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_socket_addr() {
        assert!(validate_socket_addr("127.0.0.1:8080").is_ok());
        assert!(validate_socket_addr("0.0.0.0:9090").is_ok());
        assert!(validate_socket_addr("invalid").is_err());
        assert!(validate_socket_addr("127.0.0.1:99999").is_err());
    }

    #[test]
    fn test_validate_log_level() {
        assert!(validate_log_level("debug").is_ok());
        assert!(validate_log_level("INFO").is_ok());
        assert!(validate_log_level("invalid").is_err());
    }

    #[test]
    fn test_validate_log_format() {
        assert!(validate_log_format("json").is_ok());
        assert!(validate_log_format("TEXT").is_ok());
        assert!(validate_log_format("invalid").is_err());
    }

    #[test]
    fn test_validate_environment() {
        assert!(validate_environment("production").is_ok());
        assert!(validate_environment("DEV").is_ok());
        assert!(validate_environment("invalid").is_err());
    }

    #[test]
    fn test_validate_jwt_secret() {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let valid_secret = STANDARD.encode(&[0u8; 32]);
        assert!(validate_jwt_secret(&valid_secret).is_ok());
        assert!(validate_jwt_secret("short").is_err());
        assert!(validate_jwt_secret("not_base64_but_long_enough_string_here").is_err());
    }

    #[test]
    fn test_validate_driver_type() {
        assert!(validate_driver_type("modbus").is_ok());
        assert!(validate_driver_type("OPCUA").is_ok());
        assert!(validate_driver_type("invalid").is_err());
    }

    #[test]
    fn test_validate_percentage() {
        assert!(validate_percentage(50.0).is_ok());
        assert!(validate_percentage(0.0).is_ok());
        assert!(validate_percentage(100.0).is_ok());
        assert!(validate_percentage(-1.0).is_err());
        assert!(validate_percentage(101.0).is_err());
    }

    #[test]
    fn test_validate_port() {
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(0).is_err());
        assert!(validate_port(80).is_err()); // System port
    }

    #[test]
    fn test_validate_timeout_seconds() {
        assert!(validate_timeout_seconds(30).is_ok());
        assert!(validate_timeout_seconds(0).is_err());
        assert!(validate_timeout_seconds(4000).is_err());
    }
}