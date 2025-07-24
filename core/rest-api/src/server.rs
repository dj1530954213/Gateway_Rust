/*! 
# REST API Server

HTTP server implementation using Warp framework for command API.
*/

use std::net::SocketAddr;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use anyhow::Result;
use tracing::{info, error};

use crate::auth::AuthManager;
use crate::handlers::{CommandHandler, HealthHandler};
use crate::middleware::auth_filter;
use crate::error::ApiError;

/// REST API server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub max_connections: usize,
    pub enable_cors: bool,
    pub enable_rate_limit: bool,
    pub rate_limit_per_minute: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8080".parse().unwrap(),
            max_connections: 1000,
            enable_cors: true,
            enable_rate_limit: true,
            rate_limit_per_minute: 60,
        }
    }
}

/// REST API server
pub struct ApiServer {
    config: ServerConfig,
    auth_manager: Arc<AuthManager>,
    command_handler: Arc<CommandHandler>,
    health_handler: Arc<HealthHandler>,
}

impl ApiServer {
    /// Create new API server
    pub fn new(
        config: ServerConfig,
        auth_manager: Arc<AuthManager>,
        command_handler: Arc<CommandHandler>,
        health_handler: Arc<HealthHandler>,
    ) -> Self {
        Self {
            config,
            auth_manager,
            command_handler,
            health_handler,
        }
    }

    /// Start the API server
    pub async fn start(&self) -> Result<()> {
        info!("Starting REST API server on {}", self.config.bind_addr);

        // Build route filters
        let routes = self.build_routes();

        // Start server
        warp::serve(routes)
            .run(self.config.bind_addr)
            .await;

        Ok(())
    }

    /// Build all route filters
    fn build_routes(&self) -> impl Filter<Extract = impl Reply, Error = std::convert::Infallible> + Clone {
        // Health routes (no auth required)
        let health_routes = warp::path("health")
            .and(warp::get())
            .and(with_handler(self.health_handler.clone()))
            .and_then(health_check);

        // Command routes (auth required)
        let command_routes = warp::path("commands")
            .and(warp::post())
            .and(warp::body::json())
            .and(auth_filter(self.auth_manager.clone()))
            .and(with_handler(self.command_handler.clone()))
            .and_then(execute_command);

        // Combine routes with error recovery
        health_routes
            .or(command_routes)
            .recover(crate::error::handle_rejection)
    }
}

// Helper functions
fn with_handler<T>(handler: Arc<T>) -> impl Filter<Extract = (Arc<T>,), Error = std::convert::Infallible> + Clone
where
    T: Send + Sync,
{
    warp::any().map(move || handler.clone())
}

async fn health_check(
    handler: Arc<HealthHandler>,
) -> Result<impl Reply, Rejection> {
    match handler.check().await {
        Ok(status) => Ok(warp::reply::json(&status)),
        Err(e) => {
            error!("Health check failed: {}", e);
            Err(warp::reject::custom(ApiError::Internal(e.to_string())))
        }
    }
}

async fn execute_command(
    request: serde_json::Value,
    user_id: String,
    handler: Arc<CommandHandler>,
) -> Result<impl Reply, Rejection> {
    match handler.execute(request, &user_id).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => {
            error!("Command execution failed: {}", e);
            Err(warp::reject::custom(ApiError::BadRequest(e.to_string())))
        }
    }
}

