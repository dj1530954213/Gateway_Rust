//! models.rs —— 数据库模型定义
//!
//! 定义与数据库表对应的Rust结构体：
//! - Device: 设备模型
//! - Tag: 点位模型  
//! - AlertRule: 报警规则模型
//! - AlertHistory: 报警历史模型
//! - DriverRegistry: 驱动注册模型
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ========== 枚举类型映射 ==========

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "protocol_kind", rename_all = "PascalCase")]
pub enum DbProtocolKind {
    ModbusTcp,
    OpcUa,
    Mqtt,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "tag_data_type", rename_all = "PascalCase")]
pub enum DbTagDataType {
    Float,
    Int,
    Bool,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "compare_op", rename_all = "UPPERCASE")]
pub enum DbCompareOp {
    GT,
    LT,
    GTE,
    LTE,
    EQ,
    NE,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "alert_level", rename_all = "UPPERCASE")]
pub enum DbAlertLevel {
    INFO,
    WARN,
    CRIT,
}

// ========== 设备模型 ==========

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub name: String,
    pub protocol: DbProtocolKind,
    pub location: Option<String>,
    pub endpoint: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewDevice {
    pub id: Uuid,
    pub name: String,
    pub protocol: DbProtocolKind,
    pub location: Option<String>,
    pub endpoint: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceUpdate {
    pub name: Option<String>,
    pub location: Option<String>,
    pub endpoint: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

// ========== 点位模型 ==========

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub device_id: Uuid,
    pub name: String,
    pub address: String,
    pub data_type: DbTagDataType,
    pub scaling: Option<f64>,
    pub offset: Option<f64>,
    pub unit: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTag {
    pub id: Uuid,
    pub device_id: Uuid,
    pub name: String,
    pub address: String,
    pub data_type: DbTagDataType,
    pub scaling: Option<f64>,
    pub offset: Option<f64>,
    pub unit: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagUpdate {
    pub name: Option<String>,
    pub address: Option<String>,
    pub data_type: Option<DbTagDataType>,
    pub scaling: Option<f64>,
    pub offset: Option<f64>,
    pub unit: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

// ========== 报警规则模型 ==========

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub op: DbCompareOp,
    pub threshold: f64,
    pub level: DbAlertLevel,
    #[serde(skip)]
    pub eval_every: sqlx::postgres::types::PgInterval,
    #[serde(skip)]
    pub eval_for: Option<sqlx::postgres::types::PgInterval>,
    pub enabled: bool,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub op: DbCompareOp,
    pub threshold: f64,
    pub level: DbAlertLevel,
    #[serde(skip)]
    pub eval_every: sqlx::postgres::types::PgInterval,
    #[serde(skip)]
    pub eval_for: Option<sqlx::postgres::types::PgInterval>,
    pub enabled: bool,
    pub created_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertRuleUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub op: Option<DbCompareOp>,
    pub threshold: Option<f64>,
    pub level: Option<DbAlertLevel>,
    #[serde(skip)]
    pub eval_every: Option<sqlx::postgres::types::PgInterval>,
    #[serde(skip)]
    pub eval_for: Option<sqlx::postgres::types::PgInterval>,
    pub enabled: Option<bool>,
}

// ========== 报警历史模型 ==========

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AlertHistory {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub fired_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub value: Option<f64>,
    pub threshold: f64,
    pub level: DbAlertLevel,
    pub message: String,
    pub status: String,
    #[serde(skip)]
    pub duration: Option<sqlx::postgres::types::PgInterval>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAlertHistory {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub value: Option<f64>,
    pub threshold: f64,
    pub level: DbAlertLevel,
    pub message: String,
    pub status: String,
}

// ========== 驱动注册模型 ==========

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DriverRegistry {
    pub protocol: DbProtocolKind,
    pub version: String,
    pub file_path: String,
    pub file_hash: String,
    pub api_version: i16,
    pub metadata: Option<serde_json::Value>,
    pub loaded_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewDriverRegistry {
    pub protocol: DbProtocolKind,
    pub version: String,
    pub file_path: String,
    pub file_hash: String,
    pub api_version: i16,
    pub metadata: Option<serde_json::Value>,
    pub status: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DriverLoadHistory {
    pub id: Uuid,
    pub protocol: DbProtocolKind,
    pub version: String,
    pub action: String,
    pub file_path: Option<String>,
    pub result: String,
    pub error_msg: Option<String>,
    pub loaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewDriverLoadHistory {
    pub id: Uuid,
    pub protocol: DbProtocolKind,
    pub version: String,
    pub action: String,
    pub file_path: Option<String>,
    pub result: String,
    pub error_msg: Option<String>,
}

// ========== 查询参数 ==========

#[derive(Debug, Default)]
pub struct DeviceFilter {
    pub protocol: Option<DbProtocolKind>,
    pub enabled: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Default)]
pub struct TagFilter {
    pub device_id: Option<Uuid>,
    pub data_type: Option<DbTagDataType>,
    pub enabled: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Default)]
pub struct AlertHistoryFilter {
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub level: Option<DbAlertLevel>,
    pub status: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ========== 驱动配置模型 ==========

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DriverConfig {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub connection_type: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub scan_interval: i32,
    pub timeout: i32,
    pub max_concurrent: i32,
    pub batch_size: i32,
    pub max_retries: i32,
    pub retry_interval: i32,
    pub exponential_backoff: bool,
    pub max_retry_interval: i32,
    pub log_level: String,
    pub enable_request_log: bool,
    pub enable_response_log: bool,
    pub max_log_size: i32,
    pub enable_ssl: bool,
    pub verify_certificate: bool,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewDriverConfig {
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub connection_type: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub scan_interval: i32,
    pub timeout: i32,
    pub max_concurrent: i32,
    pub batch_size: i32,
    pub max_retries: i32,
    pub retry_interval: i32,
    pub exponential_backoff: bool,
    pub max_retry_interval: i32,
    pub log_level: String,
    pub enable_request_log: bool,
    pub enable_response_log: bool,
    pub max_log_size: i32,
    pub enable_ssl: bool,
    pub verify_certificate: bool,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverConfigUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub protocol: Option<String>,
    pub connection_type: Option<String>,
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
    pub scan_interval: Option<i32>,
    pub timeout: Option<i32>,
    pub max_concurrent: Option<i32>,
    pub batch_size: Option<i32>,
    pub max_retries: Option<i32>,
    pub retry_interval: Option<i32>,
    pub exponential_backoff: Option<bool>,
    pub max_retry_interval: Option<i32>,
    pub log_level: Option<String>,
    pub enable_request_log: Option<bool>,
    pub enable_response_log: Option<bool>,
    pub max_log_size: Option<i32>,
    pub enable_ssl: Option<bool>,
    pub verify_certificate: Option<bool>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct DriverConfigFilter {
    pub protocol: Option<String>,
    pub enabled: Option<bool>,
    pub name_contains: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}