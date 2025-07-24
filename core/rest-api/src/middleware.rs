/*!
# Middleware

HTTP middleware for CORS, rate limiting, and authentication.
*/

use std::sync::Arc;
use std::time::Duration;

use warp::{Filter, Rejection};
use warp::http::Method;
use anyhow::Result;

use crate::auth::AuthManager;
use crate::error::ApiError;

/// CORS middleware filter
pub fn cors_filter() -> warp::filters::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["authorization", "content-type", "x-requested-with"])
        .allow_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .max_age(Duration::from_secs(3600))
}

/// Rate limiting middleware (simplified - no-op)
pub fn rate_limit_filter(
    _requests_per_minute: u32,
) -> impl Filter<Extract = (), Error = std::convert::Infallible> + Clone {
    // Simplified rate limiting - just pass through for now
    warp::any().map(|| ()).untuple_one()
}

/// Authentication middleware filter
pub fn auth_filter(
    auth_manager: Arc<AuthManager>,
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and_then(move |auth_header: Option<String>| {
            let auth_manager = auth_manager.clone();
            async move {
                let token = extract_bearer_token(auth_header)?;
                
                match auth_manager.validate_token(&token) {
                    Ok(claims) => Ok(claims.sub),
                    Err(_) => Err(warp::reject::custom(ApiError::Unauthorized("Invalid token".to_string()))),
                }
            }
        })
}

/// Extract bearer token from authorization header
fn extract_bearer_token(auth_header: Option<String>) -> Result<String, Rejection> {
    let header = auth_header
        .ok_or_else(|| warp::reject::custom(ApiError::Unauthorized("Missing authorization header".to_string())))?;

    if header.starts_with("Bearer ") {
        Ok(header[7..].to_string())
    } else {
        Err(warp::reject::custom(ApiError::Unauthorized("Invalid authorization format".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use frame_bus::permissions::PermissionManager;

    #[tokio::test]
    async fn test_extract_bearer_token() {
        // Valid bearer token
        let header = Some("Bearer abc123".to_string());
        let token = extract_bearer_token(header).unwrap();
        assert_eq!(token, "abc123");

        // Missing header
        let result = extract_bearer_token(None);
        assert!(result.is_err());

        // Invalid format
        let header = Some("Basic abc123".to_string());
        let result = extract_bearer_token(header);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_filter() {
        let permission_manager = Arc::new(PermissionManager::new());
        let auth_manager = Arc::new(AuthManager::new("test-secret", permission_manager));

        // Create test user
        let user = crate::auth::User::new(
            "test".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password"
        );
        auth_manager.add_user(user);

        // Get valid token
        let _token = auth_manager.authenticate("Test User", "password").unwrap();

        // Test auth filter (this would normally be used in warp filter chain)
        let _auth_filter = auth_filter(auth_manager);
        
        // Note: Full integration test would require setting up warp request context
        // This is a simplified test to verify the filter compiles correctly
    }
}