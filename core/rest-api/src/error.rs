/*!
# Error Handling

Custom error types and rejection handling for the REST API.
*/

use serde::Serialize;
use std::fmt;
use warp::{Rejection, Reply};

/// API error types
#[derive(Debug, Clone)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Internal(String),
    RateLimited,
    PayloadTooLarge,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::RateLimited => write!(f, "Rate Limited"),
            ApiError::PayloadTooLarge => write!(f, "Payload Too Large"),
        }
    }
}

impl std::error::Error for ApiError {}

impl warp::reject::Reject for ApiError {}

/// Error response structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    pub message: String,
    pub timestamp: u64,
}

impl ErrorResponse {
    pub fn new(code: u16, error: String, message: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            error,
            code,
            message,
            timestamp,
        }
    }

    pub fn bad_request(message: String) -> Self {
        Self::new(400, "Bad Request".to_string(), message)
    }

    pub fn unauthorized(message: String) -> Self {
        Self::new(401, "Unauthorized".to_string(), message)
    }

    pub fn forbidden(message: String) -> Self {
        Self::new(403, "Forbidden".to_string(), message)
    }

    pub fn not_found(message: String) -> Self {
        Self::new(404, "Not Found".to_string(), message)
    }

    pub fn internal_error(message: String) -> Self {
        Self::new(500, "Internal Server Error".to_string(), message)
    }

    pub fn rate_limited() -> Self {
        Self::new(429, "Too Many Requests".to_string(), "Rate limit exceeded".to_string())
    }

    pub fn payload_too_large() -> Self {
        Self::new(413, "Payload Too Large".to_string(), "Request body too large".to_string())
    }
}

/// Convert API error to HTTP response
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let (response, status_code) = if let Some(api_error) = err.find::<ApiError>() {
        match api_error {
            ApiError::BadRequest(msg) => {
                let response = ErrorResponse::bad_request(msg.clone());
                (response, warp::http::StatusCode::BAD_REQUEST)
            }
            ApiError::Unauthorized(msg) => {
                let response = ErrorResponse::unauthorized(msg.clone());
                (response, warp::http::StatusCode::UNAUTHORIZED)
            }
            ApiError::Forbidden(msg) => {
                let response = ErrorResponse::forbidden(msg.clone());
                (response, warp::http::StatusCode::FORBIDDEN)
            }
            ApiError::NotFound(msg) => {
                let response = ErrorResponse::not_found(msg.clone());
                (response, warp::http::StatusCode::NOT_FOUND)
            }
            ApiError::Internal(msg) => {
                let response = ErrorResponse::internal_error(msg.clone());
                (response, warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
            ApiError::RateLimited => {
                let response = ErrorResponse::rate_limited();
                (response, warp::http::StatusCode::TOO_MANY_REQUESTS)
            }
            ApiError::PayloadTooLarge => {
                let response = ErrorResponse::payload_too_large();
                (response, warp::http::StatusCode::PAYLOAD_TOO_LARGE)
            }
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        let response = ErrorResponse::new(
            405,
            "Method Not Allowed".to_string(),
            "HTTP method not allowed for this endpoint".to_string(),
        );
        (response, warp::http::StatusCode::METHOD_NOT_ALLOWED)
    } else if err.find::<warp::reject::InvalidQuery>().is_some() {
        let response = ErrorResponse::bad_request("Invalid query parameters".to_string());
        (response, warp::http::StatusCode::BAD_REQUEST)
    } else if err.find::<warp::reject::InvalidHeader>().is_some() {
        let response = ErrorResponse::bad_request("Invalid headers".to_string());
        (response, warp::http::StatusCode::BAD_REQUEST)
    } else if err.find::<warp::reject::MissingHeader>().is_some() {
        let response = ErrorResponse::bad_request("Missing required headers".to_string());
        (response, warp::http::StatusCode::BAD_REQUEST)
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        let response = ErrorResponse::payload_too_large();
        (response, warp::http::StatusCode::PAYLOAD_TOO_LARGE)
    } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
        let response = ErrorResponse::new(
            415,
            "Unsupported Media Type".to_string(),
            "Content-Type not supported".to_string(),
        );
        (response, warp::http::StatusCode::UNSUPPORTED_MEDIA_TYPE)
    } else {
        // Fallback for unknown rejections
        tracing::error!("Unhandled rejection: {:?}", err);
        let response = ErrorResponse::internal_error("Internal server error".to_string());
        (response, warp::http::StatusCode::INTERNAL_SERVER_ERROR)
    };

    let json = warp::reply::json(&response);
    Ok(warp::reply::with_status(json, status_code))
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Helper macro for creating API errors
#[macro_export]
macro_rules! api_error {
    (bad_request, $msg:expr) => {
        ApiError::BadRequest($msg.to_string())
    };
    (unauthorized, $msg:expr) => {
        ApiError::Unauthorized($msg.to_string())
    };
    (forbidden, $msg:expr) => {
        ApiError::Forbidden($msg.to_string())
    };
    (not_found, $msg:expr) => {
        ApiError::NotFound($msg.to_string())
    };
    (internal, $msg:expr) => {
        ApiError::Internal($msg.to_string())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::bad_request("Invalid input".to_string());
        assert_eq!(error.code, 400);
        assert_eq!(error.error, "Bad Request");
        assert_eq!(error.message, "Invalid input");

        let error = ErrorResponse::rate_limited();
        assert_eq!(error.code, 429);
        assert_eq!(error.error, "Too Many Requests");
    }

    #[test]
    fn test_api_error_display() {
        let error = ApiError::BadRequest("Test error".to_string());
        assert_eq!(format!("{}", error), "Bad Request: Test error");

        let error = ApiError::RateLimited;
        assert_eq!(format!("{}", error), "Rate Limited");
    }

    #[test]
    fn test_api_error_macro() {
        let error = api_error!(bad_request, "Test message");
        match error {
            ApiError::BadRequest(msg) => assert_eq!(msg, "Test message"),
            _ => panic!("Wrong error type"),
        }

        let error = api_error!(unauthorized, "Access denied");
        match error {
            ApiError::Unauthorized(msg) => assert_eq!(msg, "Access denied"),
            _ => panic!("Wrong error type"),
        }
    }
}