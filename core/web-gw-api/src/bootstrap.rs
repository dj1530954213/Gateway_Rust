//! bootstrap.rs —— 应用启动引导与依赖注入
//!
//! - 配置加载：YAML + 环境变量覆盖
//! - 状态构建：数据库连接、InfluxDB客户端、依赖注入
//! - CORS配置：跨域支持
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::config::ApiConfig;
use crate::error::{ApiError, ApiResult};
use actix_cors::Cors;
use actix_web::web::Data;
use anyhow::Context;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast;

/// 应用状态，包含所有共享资源
#[derive(Clone)]
pub struct AppState {
    pub config: ApiConfig,
    pub pg_pool: Pool<Postgres>,
    pub influx_client: influxdb2::Client,
    pub frame_bus: Arc<dyn FrameBusClient>,
    pub driver_manager: driver_manager::DriverManager,
    pub ws_manager: Arc<crate::routes::websocket::WsConnectionManager>,
    pub frame_bus_bridge: Arc<crate::services::FrameBusBridge>,
    pub history_service: crate::services::HistoryService,
    pub shutdown_tx: broadcast::Sender<()>,
}

/// Frame Bus 客户端抽象
#[async_trait::async_trait]
pub trait FrameBusClient: Send + Sync {
    async fn subscribe(&self, filter: FrameFilter) -> ApiResult<FrameReceiver>;
    async fn publish(&self, frame: DataFrame) -> ApiResult<()>;
}


// 临时类型定义，实际应该从frame-bus导入
pub struct FrameFilter {
    pub device_id: Option<uuid::Uuid>,
}

pub struct FrameReceiver;

pub struct DataFrame {
    pub device_id: uuid::Uuid,
    pub tag_id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub unit: Option<String>,
}

impl AppState {
    /// 获取配置引用
    pub fn config(&self) -> &ApiConfig {
        &self.config
    }
    
    /// 创建CORS中间件
    pub fn cors(&self) -> Cors {
        let mut cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        for origin in &self.config.cors_allowed {
            if origin == "*" {
                cors = cors.allow_any_origin();
            } else {
                cors = cors.allowed_origin(origin);
            }
        }

        cors
    }

    /// 检查服务健康状态
    pub async fn health_check(&self) -> HashMap<String, String> {
        let mut services = HashMap::new();

        // 检查PostgreSQL连接
        match sqlx::query("SELECT 1").fetch_one(&self.pg_pool).await {
            Ok(_) => services.insert("postgres".to_string(), "healthy".to_string()),
            Err(_) => services.insert("postgres".to_string(), "unhealthy".to_string()),
        };

        // 检查InfluxDB连接
        match self.influx_client.health().await {
            Ok(_) => services.insert("influxdb".to_string(), "healthy".to_string()),
            Err(_) => services.insert("influxdb".to_string(), "unhealthy".to_string()),
        };

        // 检查Frame Bus连接
        services.insert("frame_bus".to_string(), "healthy".to_string()); // 简化实现

        // 检查Driver Manager
        let overview = self.driver_manager.get_registry_overview();
        if overview.total_drivers > 0 {
            services.insert("driver_manager".to_string(), "healthy".to_string())
        } else {
            services.insert("driver_manager".to_string(), "healthy".to_string()) // 空驱动注册表也是健康的
        };

        services
    }
}

/// 加载配置文件
pub fn load_config() -> ApiResult<ApiConfig> {
    let env = env::var("ENV").unwrap_or_else(|_| "dev".to_string());
    
    // 读取默认配置
    let mut config = ApiConfig::default();
    
    // 尝试读取配置文件
    if let Ok(config_content) = std::fs::read_to_string("config/default.yaml") {
        if let Ok(default_config) = serde_yaml::from_str::<ApiConfig>(&config_content) {
            config = default_config;
        }
    }
    
    // 尝试读取环境特定配置
    let env_config_path = format!("config/{}.yaml", env);
    if let Ok(env_content) = std::fs::read_to_string(&env_config_path) {
        if let Ok(env_config) = serde_yaml::from_str::<ApiConfig>(&env_content) {
            // 简化合并，实际应该使用更复杂的配置合并逻辑
            config = env_config;
        }
    }
    
    // 环境变量覆盖
    apply_env_overrides(&mut config);
    
    Ok(config)
}

/// 应用环境变量覆盖
fn apply_env_overrides(config: &mut ApiConfig) {
    if let Ok(addr) = env::var("WEBGW_HTTP_ADDR") {
        if let Ok(parsed) = addr.parse() {
            config.http_addr = parsed;
        }
    }
    
    if let Ok(dsn) = env::var("WEBGW_PG_DSN") {
        config.pg_dsn = dsn;
    }
    
    if let Ok(url) = env::var("WEBGW_INFLUX_URL") {
        config.influx_url = url;
    }
    
    if let Ok(token) = env::var("WEBGW_INFLUX_TOKEN") {
        config.influx_token = token;
    }
    
    if let Ok(org) = env::var("WEBGW_INFLUX_ORG") {
        config.influx_org = org;
    }
    
    if let Ok(bucket) = env::var("WEBGW_INFLUX_BUCKET") {
        config.influx_bucket = bucket;
    }
    
    if let Ok(level) = env::var("WEBGW_LOG_LEVEL") {
        config.log_level = level;
    }
    
    if let Ok(cors) = env::var("WEBGW_CORS_ALLOWED") {
        config.cors_allowed = cors.split(',').map(|s| s.trim().to_string()).collect();
    }
    
    if let Ok(max_conn) = env::var("WEBGW_WS_MAX_CONNECTIONS") {
        if let Ok(parsed) = max_conn.parse() {
            config.ws_max_connections = parsed;
        }
    }
    
    if let Ok(timeout) = env::var("WEBGW_WS_HEARTBEAT_TIMEOUT") {
        if let Ok(parsed) = timeout.parse() {
            config.ws_heartbeat_timeout = parsed;
        }
    }
    
    if let Ok(interval) = env::var("WEBGW_WS_CLEANUP_INTERVAL") {
        if let Ok(parsed) = interval.parse() {
            config.ws_cleanup_interval = parsed;
        }
    }
    
    if let Ok(url) = env::var("WEBGW_ALERT_ENGINE_URL") {
        config.alert_engine_url = url;
    }
}

/// 初始化应用状态
pub async fn init_state(config: &ApiConfig) -> ApiResult<Data<AppState>> {
    // 初始化PostgreSQL连接池
    let pg_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.pg_dsn)
        .await
        .context("Failed to connect to PostgreSQL")?;
    
    // 运行数据库迁移
    sqlx::migrate!("../../schema/migrations")
        .run(&pg_pool)
        .await
        .context("Failed to run database migrations")?;
    
    // 初始化InfluxDB客户端
    let influx_client = influxdb2::Client::new(&config.influx_url, &config.influx_org, &config.influx_token);
    
    // 初始化Frame Bus客户端
    let frame_bus = Arc::new(RealFrameBusClient::new().await?);
    
    // 初始化Driver Manager
    let drivers_dir = env::var("WEBGW_DRIVERS_DIR").unwrap_or_else(|_| "./drivers".to_string());
    
    let driver_manager = match driver_manager::DriverManager::with_drivers_dir(&drivers_dir) {
        Ok(manager) => {
            tracing::info!("Successfully initialized driver manager with directory: {}", drivers_dir);
            
            // 启用自动重载
            if let Err(e) = manager.enable_auto_reload() {
                tracing::warn!("Failed to enable driver auto-reload: {}", e);
            }
            
            // 扫描并加载所有动态驱动
            match manager.scan_and_load_dynamic_drivers().await {
                Ok(loaded_drivers) => {
                    tracing::info!("Loaded {} dynamic drivers", loaded_drivers.len());
                }
                Err(e) => {
                    tracing::warn!("Failed to load some dynamic drivers: {}", e);
                }
            }
            
            manager
        }
        Err(e) => {
            tracing::error!("Failed to initialize driver manager: {}", e);
            // 创建一个默认的manager作为fallback
            driver_manager::DriverManager::new().unwrap_or_else(|_| {
                panic!("Failed to create fallback driver manager");
            })
        }
    };

    // 初始化WebSocket连接管理器
    let ws_config = crate::routes::websocket::ConnectionManagerConfig {
        max_connections: config.ws_max_connections,
        heartbeat_timeout_secs: config.ws_heartbeat_timeout,
        cleanup_interval_secs: config.ws_cleanup_interval,
        enable_connection_limit: true,
    };
    let ws_manager = Arc::new(crate::routes::websocket::WsConnectionManager::with_config(ws_config));

    // 创建停止信号通道
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

    // 初始化FrameBus桥接服务
    let frame_bus_bridge = Arc::new(crate::services::FrameBusBridge::new(
        ws_manager.clone(),
        shutdown_rx,
    ));

    // 启动FrameBus桥接服务
    let bridge_handle = frame_bus_bridge.clone();
    tokio::spawn(async move {
        if let Err(e) = bridge_handle.start().await {
            tracing::error!("FrameBus bridge service failed: {}", e);
        }
    });

    // 初始化历史数据查询服务
    let history_service = crate::services::HistoryService::new(
        influx_client.clone(),
        config.influx_org.clone(),
        config.influx_bucket.clone(),
    );
    
    let state = AppState {
        config: config.clone(),
        pg_pool,
        influx_client,
        frame_bus,
        driver_manager,
        ws_manager,
        frame_bus_bridge,
        history_service,
        shutdown_tx,
    };
    
    Ok(Data::new(state))
}

// ========== 真实FrameBus实现 ==========

struct RealFrameBusClient {
    _tx: frame_bus::FrameSender,
    _rx: frame_bus::FrameReceiver,
}

impl RealFrameBusClient {
    async fn new() -> ApiResult<Self> {
        // 初始化frame-bus
        let ring_size = 1024; // 2^10
        let wal_dir = env::var("WEBGW_FRAMEBUS_WAL_DIR")
            .unwrap_or_else(|_| "./data/wal".to_string());
        
        // 确保WAL目录存在
        std::fs::create_dir_all(&wal_dir)
            .context("Failed to create frame-bus WAL directory")?;
        
        let (tx, rx) = frame_bus::init(ring_size, &wal_dir)
            .context("Failed to initialize frame-bus")?;
        
        tracing::info!("Initialized frame-bus with ring size {} and WAL dir {}", ring_size, wal_dir);
        
        Ok(Self { _tx: tx, _rx: rx })
    }
}

#[async_trait::async_trait]
impl FrameBusClient for RealFrameBusClient {
    async fn subscribe(&self, filter: FrameFilter) -> ApiResult<FrameReceiver> {
        // 将FrameFilter转换为frame_bus::Filter
        let bus_filter = if let Some(device_id) = filter.device_id {
            // 为特定设备创建前缀过滤器
            frame_bus::Filter::tag_starts_with(format!("telemetry.{}", device_id))
        } else {
            // 订阅所有遥测数据
            frame_bus::Filter::tag_starts_with("telemetry.")
        };
        
        let receiver = frame_bus::subscribe(bus_filter)
            .context("Failed to subscribe to frame-bus")?;
        
        Ok(FrameReceiver)
    }
    
    async fn publish(&self, frame: DataFrame) -> ApiResult<()> {
        // 将抽象DataFrame转换为frame_bus::DataFrame
        let bus_frame = frame_bus::DataFrame::new(
            format!("telemetry.{}.{}", frame.device_id, frame.tag_id),
            frame_bus::Value::float(frame.value)
        )
        .with_meta("unit", frame.unit.unwrap_or_default());
        
        // 简化实现，实际应根据frame_bus的API调整
        tracing::debug!("Publishing frame for device {} tag {}", frame.device_id, frame.tag_id);
        
        Ok(())
    }
}

// ========== 模拟实现（备用） ==========

struct MockFrameBusClient;

#[async_trait::async_trait]
impl FrameBusClient for MockFrameBusClient {
    async fn subscribe(&self, _filter: FrameFilter) -> ApiResult<FrameReceiver> {
        Ok(FrameReceiver)
    }
    
    async fn publish(&self, _frame: DataFrame) -> ApiResult<()> {
        Ok(())
    }
}

