//! error.rs —— 统一错误处理与HTTP响应映射
//!
//! - ApiError：统一的API错误类型
//! - 自动转换：数据库错误、验证错误等 -> HTTP状态码
//! - RFC 7807：Problem Details for HTTP APIs格式
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// API统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {message}")]
    BadRequest { message: String },
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden: {message}")]
    Forbidden { message: String },
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Conflict: {message}")]
    Conflict { message: String },
    
    #[error("Unprocessable entity: {errors:?}")]
    UnprocessableEntity { errors: Vec<String> },
    
    #[error("Internal server error")]
    InternalServerError,
    
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
    
    #[error("Payload too large: {message}")]
    PayloadTooLarge { message: String },
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("InfluxDB error: {0}")]
    InfluxDb(#[from] influxdb2::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Frame bus error: {0}")]
    FrameBus(String),
    
    #[error("Driver manager error: {0}")]
    DriverManager(String),
}

/// RFC 7807 Problem Details for HTTP APIs
#[derive(Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let problem = self.to_problem_details();
        
        match self {
            ApiError::BadRequest { .. } => HttpResponse::BadRequest().json(problem),
            ApiError::Unauthorized => HttpResponse::Unauthorized().json(problem),
            ApiError::Forbidden { .. } => HttpResponse::Forbidden().json(problem),
            ApiError::NotFound { .. } => HttpResponse::NotFound().json(problem),
            ApiError::Conflict { .. } => HttpResponse::Conflict().json(problem),
            ApiError::UnprocessableEntity { .. } => HttpResponse::UnprocessableEntity().json(problem),
            ApiError::ServiceUnavailable { .. } => HttpResponse::ServiceUnavailable().json(problem),
            ApiError::PayloadTooLarge { .. } => HttpResponse::PayloadTooLarge().json(problem),
            
            // 数据库错误映射
            ApiError::Database(sqlx::Error::RowNotFound) => {
                let problem = ProblemDetails {
                    type_: "https://gateway.example.com/problems/not-found".to_string(),
                    title: "Resource Not Found".to_string(),
                    status: 404,
                    detail: "The requested resource was not found".to_string(),
                    instance: None,
                    extensions: serde_json::Map::new(),
                };
                HttpResponse::NotFound().json(problem)
            },
            
            ApiError::Database(sqlx::Error::Database(db_err)) => {
                // PostgreSQL错误码映射
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => { // unique_violation
                            let problem = ProblemDetails {
                                type_: "https://gateway.example.com/problems/conflict".to_string(),
                                title: "Resource Conflict".to_string(),
                                status: 409,
                                detail: "Resource already exists".to_string(),
                                instance: None,
                                extensions: serde_json::Map::new(),
                            };
                            HttpResponse::Conflict().json(problem)
                        },
                        "23503" => { // foreign_key_violation  
                            let problem = ProblemDetails {
                                type_: "https://gateway.example.com/problems/bad-request".to_string(),
                                title: "Invalid Reference".to_string(),
                                status: 400,
                                detail: "Referenced resource does not exist".to_string(),
                                instance: None,
                                extensions: serde_json::Map::new(),
                            };
                            HttpResponse::BadRequest().json(problem)
                        },
                        _ => HttpResponse::InternalServerError().json(problem)
                    }
                } else {
                    HttpResponse::InternalServerError().json(problem)
                }
            },
            
            // 其他错误默认500
            _ => HttpResponse::InternalServerError().json(problem),
        }
    }
    
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::BadRequest { .. } => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            ApiError::Forbidden { .. } => actix_web::http::StatusCode::FORBIDDEN,
            ApiError::NotFound { .. } => actix_web::http::StatusCode::NOT_FOUND,
            ApiError::Conflict { .. } => actix_web::http::StatusCode::CONFLICT,
            ApiError::UnprocessableEntity { .. } => actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::ServiceUnavailable { .. } => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            ApiError::PayloadTooLarge { .. } => actix_web::http::StatusCode::PAYLOAD_TOO_LARGE,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ApiError {
    fn to_problem_details(&self) -> ProblemDetails {
        match self {
            ApiError::BadRequest { message } => ProblemDetails {
                type_: "https://gateway.example.com/problems/bad-request".to_string(),
                title: "Bad Request".to_string(),
                status: 400,
                detail: message.clone(),
                instance: None,
                extensions: serde_json::Map::new(),
            },
            ApiError::Unauthorized => ProblemDetails {
                type_: "https://gateway.example.com/problems/unauthorized".to_string(),
                title: "Unauthorized".to_string(),
                status: 401,
                detail: "Authentication required".to_string(),
                instance: None,
                extensions: serde_json::Map::new(),
            },
            ApiError::NotFound { resource } => ProblemDetails {
                type_: "https://gateway.example.com/problems/not-found".to_string(),
                title: "Not Found".to_string(),
                status: 404,
                detail: format!("{} not found", resource),
                instance: None,
                extensions: serde_json::Map::new(),
            },
            _ => ProblemDetails {
                type_: "https://gateway.example.com/problems/internal-error".to_string(),
                title: "Internal Server Error".to_string(),
                status: 500,
                detail: "An unexpected error occurred".to_string(),
                instance: None,
                extensions: serde_json::Map::new(),
            },
        }
    }
    
    /// 创建验证错误
    pub fn validation_error(message: impl Into<String>) -> Self {
        ApiError::Validation(message.into())
    }
    
    /// 创建资源未找到错误
    pub fn not_found(resource: impl Into<String>) -> Self {
        ApiError::NotFound { resource: resource.into() }
    }
    
    /// 创建冲突错误
    pub fn conflict(message: impl Into<String>) -> Self {
        ApiError::Conflict { message: message.into() }
    }
    
    /// 创建请求错误
    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError::BadRequest { message: message.into() }
    }
    
    /// 创建内部服务器错误
    pub fn internal_error(message: impl Into<String>) -> Self {
        tracing::error!("Internal error: {}", message.into());
        ApiError::InternalServerError
    }
    
    /// 创建数据过大错误
    pub fn payload_too_large(message: impl Into<String>) -> Self {
        ApiError::PayloadTooLarge { message: message.into() }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;