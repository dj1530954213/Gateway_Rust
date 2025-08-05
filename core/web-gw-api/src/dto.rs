//! dto.rs —— 数据传输对象定义
//!
//! - 外部可见的JSON结构体
//! - 统一采用snake_case字段命名
//! - 包含请求、响应、查询参数等DTO
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::{ToSchema, IntoParams};

// ========== 枚举类型 ==========

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum ProtocolKind {
    ModbusTcp,
    OpcUa,
    Mqtt,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum TagDataType {
    Float,
    Int,
    Bool,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CompareOp {
    GT,  // >
    LT,  // <
    GTE, // >=
    LTE, // <=
    EQ,  // ==
    NE,  // !=
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AlertLevel {
    INFO,
    WARN,
    CRIT,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StatsKind {
    Mean,
    Min,
    Max,
    Sum,
}

// ========== 设备相关 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeviceVO {
    pub id: Uuid,
    pub name: String,
    pub protocol: ProtocolKind,
    pub location: Option<String>,
    pub endpoint: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeviceCreateReq {
    pub name: String,
    pub protocol: ProtocolKind,
    pub location: Option<String>,
    pub endpoint: Option<String>,
    pub config: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DevicePatchReq {
    pub name: Option<String>,
    pub location: Option<String>,
    pub endpoint: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct DeviceQuery {
    pub protocol: Option<ProtocolKind>,
    pub enabled: Option<bool>,
    pub page: Option<u64>,
    pub size: Option<u64>,
}

// ========== 点位相关 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TagVO {
    pub id: Uuid,
    pub device_id: Uuid,
    pub name: String,
    pub address: String,
    pub data_type: TagDataType,
    pub scaling: Option<f64>,
    pub offset: Option<f64>,
    pub unit: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TagCreateReq {
    pub device_id: Uuid,
    pub name: String,
    pub address: String,
    pub data_type: TagDataType,
    pub scaling: Option<f64>,
    pub offset: Option<f64>,
    pub unit: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TagPatchReq {
    pub name: Option<String>,
    pub address: Option<String>,
    pub data_type: Option<TagDataType>,
    pub scaling: Option<f64>,
    pub offset: Option<f64>,
    pub unit: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TagQuery {
    pub device_id: Option<Uuid>,
    pub data_type: Option<TagDataType>,
    pub enabled: Option<bool>,
    pub page: Option<u64>,
    pub size: Option<u64>,
}

// ========== 驱动相关 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriverInfo {
    pub protocol: ProtocolKind,
    pub version: String,
    pub file_path: String,
    pub api_version: i16,
    pub status: String,
    pub loaded_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DriverActionResult {
    pub protocol: ProtocolKind,
    pub action: String,
    pub result: String,
    pub message: Option<String>,
}

// ========== 驱动管理新增 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnifiedDriverEntryVO {
    pub driver_id: String,
    pub driver_kind: String,
    pub name: String,
    pub version: String,
    pub protocol: String,
    pub status: String,
    pub description: String,
    pub features: Vec<String>,
    pub loaded_at: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub stats: Option<DriverStatisticsVO>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriverStatisticsVO {
    pub attached_devices: u32,
    pub read_count: u64,
    pub write_count: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriverStatusVO {
    pub loaded_count: u32,
    pub failed_count: u32,
    pub unloaded_count: u32,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegistryOverviewVO {
    pub total_drivers: u32,
    pub static_drivers: u32,
    pub dynamic_drivers: u32,
    pub running_drivers: u32,
    pub error_drivers: u32,
    pub protocol_stats: std::collections::HashMap<String, u32>,
    pub status_stats: std::collections::HashMap<String, u32>,
}

// 驱动上传相关
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriverUploadInfo {
    pub filename: String,
    pub driver_id: String,
    pub file_size: u64,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DriverUploadResponse {
    pub success: bool,
    pub uploaded_files: Vec<DriverUploadInfo>,
    pub message: String,
}

// 驱动列表查询相关
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct DriverListQuery {
    pub driver_kind: Option<String>,
    pub protocol: Option<String>,
    pub status: Option<String>,
    pub name_contains: Option<String>,
    pub active_only: Option<bool>,
    pub error_only: Option<bool>,
    pub sort_by: Option<String>,
    pub descending: Option<bool>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl DriverListQuery {
    pub fn has_filters(&self) -> bool {
        self.driver_kind.is_some() ||
        self.protocol.is_some() ||
        self.status.is_some() ||
        self.name_contains.is_some() ||
        self.active_only.unwrap_or(false) ||
        self.error_only.unwrap_or(false)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DriverListResponse {
    pub drivers: Vec<UnifiedDriverEntryVO>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

// 驱动详情相关
#[derive(Debug, Serialize, ToSchema)]
pub struct DriverDetailResponse {
    pub driver: UnifiedDriverEntryVO,
}

// 驱动搜索相关
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct DriverSearchQuery {
    pub q: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DriverSearchResponse {
    pub query: String,
    pub results: Vec<UnifiedDriverEntryVO>,
    pub total: u32,
}

// 注册表概览相关
#[derive(Debug, Serialize, ToSchema)]
pub struct RegistryOverviewResponse {
    pub overview: RegistryOverviewVO,
}

// 驱动重载相关
#[derive(Debug, Deserialize, ToSchema)]
pub struct DriverReloadRequest {
    pub force: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DriverReloadResponse {
    pub success: bool,
    pub old_driver_id: String,
    pub new_driver_id: Option<String>,
    pub message: String,
}

// 驱动卸载相关
#[derive(Debug, Serialize, ToSchema)]
pub struct DriverUnloadResponse {
    pub success: bool,
    pub driver_id: String,
    pub message: String,
}

// 通用API错误响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

// ========== 历史数据相关 DTO ==========

#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams, Default)]
pub struct HistoryQuery {
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub aggregation_window: Option<String>, // 如 "1s", "1m", "1h"
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryPointVO {
    pub device_id: Uuid,
    pub tag_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub unit: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryStatsVO {
    pub device_id: Uuid,
    pub tag_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub avg_value: Option<f64>,
    pub count: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct HistoryExportRequest {
    #[serde(flatten)]
    pub query: HistoryQuery,
    #[serde(default = "default_true")]
    pub include_headers: bool,
    #[serde(default = "default_iso8601")]
    pub timestamp_format: String, // "ISO8601", "UNIX", "UNIX_MS", "FORMATTED"
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct AggregatedQuery {
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub window: String, // 如 "5m", "1h", "1d"
    pub function: String, // "mean", "min", "max", "sum", "count"
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesPoint {
    pub ts: i64,    // 时间戳（毫秒）
    pub value: f64,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct StatsQuery {
    #[serde(flatten)]
    pub base: HistoryQuery,
    pub agg: StatsKind,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatsResp {
    pub device_id: Uuid,
    pub tag_id: Uuid,
    pub agg: StatsKind,
    pub value: f64,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

// ========== 报警相关 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AlertRuleVO {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub op: CompareOp,
    pub threshold: f64,
    pub level: AlertLevel,
    pub eval_every: String,     // 如 "10s"
    pub eval_for: Option<String>, // 如 "30s"
    pub enabled: bool,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AlertRuleCreateReq {
    pub name: String,
    pub description: Option<String>,
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub op: CompareOp,
    pub threshold: f64,
    pub level: AlertLevel,
    pub eval_every: String,
    pub eval_for: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AlertRulePatchReq {
    pub name: Option<String>,
    pub description: Option<String>,
    pub op: Option<CompareOp>,
    pub threshold: Option<f64>,
    pub level: Option<AlertLevel>,
    pub eval_every: Option<String>,
    pub eval_for: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AlertEventVO {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub fired_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub value: Option<f64>,
    pub threshold: f64,
    pub level: AlertLevel,
    pub message: String,
    pub status: String,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct AlertHistoryQuery {
    pub device_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub level: Option<AlertLevel>,
    pub status: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<u64>,
    pub size: Option<u64>,
}

// ========== WebSocket 消息 ==========

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TelemetryMsg {
    pub device_id: Uuid,
    pub tag_id: Uuid,
    pub ts: i64,        // 时间戳（毫秒）
    pub value: f64,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AlertNotification {
    pub event_id: Uuid,
    pub rule_name: String,
    pub device_name: Option<String>,
    pub tag_name: Option<String>,
    pub level: AlertLevel,
    pub message: String,
    pub fired_at: DateTime<Utc>,
    pub value: Option<f64>,
    pub threshold: f64,
}

// ========== 通用响应 ==========

#[derive(Debug, Serialize, ToSchema)]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
    pub pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub services: std::collections::HashMap<String, String>,
}

// ========== 驱动配置相关 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriverConfigVO {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub connection_type: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    
    // 性能设置
    pub scan_interval: i32,
    pub timeout: i32,
    pub max_concurrent: i32,
    pub batch_size: i32,
    
    // 重连策略
    pub max_retries: i32,
    pub retry_interval: i32,
    pub exponential_backoff: bool,
    pub max_retry_interval: i32,
    
    // 日志设置
    pub log_level: String,
    pub enable_request_log: bool,
    pub enable_response_log: bool,
    pub max_log_size: i32,
    
    // 安全配置
    pub enable_ssl: bool,
    pub verify_certificate: bool,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DriverConfigCreateReq {
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,  // 'modbus_tcp', 'modbus_rtu', 'opcua', 'mqtt', 'ethernet_ip', 'bacnet'
    pub connection_type: String,  // 'ethernet', 'serial'
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub config: serde_json::Value,
    
    // 性能设置
    #[serde(default = "default_scan_interval")]
    pub scan_interval: i32,
    #[serde(default = "default_timeout")]
    pub timeout: i32,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: i32,
    #[serde(default = "default_batch_size")]
    pub batch_size: i32,
    
    // 重连策略
    #[serde(default = "default_max_retries")]
    pub max_retries: i32,
    #[serde(default = "default_retry_interval")]  
    pub retry_interval: i32,
    #[serde(default = "default_true")]
    pub exponential_backoff: bool,
    #[serde(default = "default_max_retry_interval")]
    pub max_retry_interval: i32,
    
    // 日志设置
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub enable_request_log: bool,
    #[serde(default)]
    pub enable_response_log: bool,
    #[serde(default = "default_max_log_size")]
    pub max_log_size: i32,
    
    // 安全配置
    #[serde(default)]
    pub enable_ssl: bool,
    #[serde(default = "default_true")]
    pub verify_certificate: bool,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DriverConfigUpdateReq {
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

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct DriverConfigQuery {
    pub protocol: Option<String>,
    pub enabled: Option<bool>,
    pub name_contains: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DriverConfigResponse {
    pub driver_config: DriverConfigVO,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DriverConfigListResponse {
    pub driver_configs: Vec<DriverConfigVO>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

// ========== 辅助函数 ==========

fn default_true() -> bool {
    true
}

fn default_iso8601() -> String {
    "ISO8601".to_string()
}

// 驱动配置默认值函数
fn default_scan_interval() -> i32 {
    1000
}

fn default_timeout() -> i32 {
    5000
}

fn default_max_concurrent() -> i32 {
    10
}

fn default_batch_size() -> i32 {
    100
}

fn default_max_retries() -> i32 {
    3
}

fn default_retry_interval() -> i32 {
    1000
}

fn default_max_retry_interval() -> i32 {
    10000
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_log_size() -> i32 {
    10
}

impl std::fmt::Display for ProtocolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolKind::ModbusTcp => write!(f, "ModbusTcp"),
            ProtocolKind::OpcUa => write!(f, "OpcUa"),
            ProtocolKind::Mqtt => write!(f, "Mqtt"),
        }
    }
}

impl std::str::FromStr for ProtocolKind {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ModbusTcp" => Ok(ProtocolKind::ModbusTcp),
            "OpcUa" => Ok(ProtocolKind::OpcUa),
            "Mqtt" => Ok(ProtocolKind::Mqtt),
            _ => Err(format!("Unknown protocol: {}", s)),
        }
    }
}

impl std::str::FromStr for StatsKind {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mean" => Ok(StatsKind::Mean),
            "min" => Ok(StatsKind::Min),
            "max" => Ok(StatsKind::Max),
            "sum" => Ok(StatsKind::Sum),
            _ => Err(format!("Unknown stats kind: {}", s)),
        }
    }
}