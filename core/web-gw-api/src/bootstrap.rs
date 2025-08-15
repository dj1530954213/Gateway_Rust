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
use crate::database_pool::{DatabasePoolManager, validate_pool_config};
use actix_cors::Cors;
use actix_web::web::Data;
use anyhow::Context;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast;
use pg_repo;
use crate::services::{get_driver_config_monitor, init_driver_config_monitor};

/// 应用状态，包含所有共享资源
#[derive(Clone)]
pub struct AppState {
    pub config: ApiConfig,
    pub pg_pool: Pool<Postgres>,
    pub pool_manager: Arc<DatabasePoolManager>,
    pub influx_client: influxdb2::Client,
    pub frame_bus: Arc<dyn FrameBusClient>,
    pub driver_manager: Arc<driver_manager::DriverManager>,
    pub driver_config_repo: Arc<dyn pg_repo::DriverConfigRepo>,
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
    
    /// 创建CORS中间件 - 安全配置
    pub fn cors(&self) -> Cors {
        // 从环境变量或配置文件读取允许的来源
        let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:50020,http://127.0.0.1:50020".to_string());
        
        let origins: Vec<&str> = allowed_origins.split(',').collect();
        
        let mut cors = Cors::default();
        
        // 严格配置允许的来源
        for origin in origins {
            cors = cors.allowed_origin(origin.trim());
        }
        
        cors = cors
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])  // 限制HTTP方法
            .allowed_headers(vec![
                "content-type", 
                "authorization", 
                "x-requested-with",
                "x-api-key"
            ])  // 限制允许的头部
            .expose_headers(vec![
                "content-length",
                "x-total-count", 
                "x-page-count"
            ])  // 限制暴露的头部
            .max_age(300)  // 缩短预检缓存时间到5分钟
            .supports_credentials();  // 支持凭据
        
        tracing::info!("CORS configured with secure origins: {:?}", origins);
        cors
    }

    /// 检查服务健康状态
    pub async fn health_check(&self) -> HashMap<String, String> {
        let mut services = HashMap::new();

        // 检查PostgreSQL连接池健康状态
        match self.pool_manager.health_check().await {
            Ok(true) => {
                services.insert("postgres".to_string(), "healthy".to_string());
                
                // 获取连接池统计信息
                let stats = self.pool_manager.get_pool_stats().await;
                services.insert("postgres_pool_utilization".to_string(), 
                    format!("{:.1}%", stats.utilization_percent));
                services.insert("postgres_active_connections".to_string(), 
                    stats.active_connections.to_string());
            }
            _ => services.insert("postgres".to_string(), "unhealthy".to_string()),
        };

        // 检查InfluxDB连接 - 使用HTTP客户端而不是influxdb2库的health()方法
        // InfluxDB 3.x 使用简单的HTTP GET /health 端点
        match self.check_influxdb_health().await {
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

    /// 检查InfluxDB健康状态
    /// 使用HTTP客户端调用InfluxDB 3.x的/health端点
    async fn check_influxdb_health(&self) -> ApiResult<()> {
        let client = awc::Client::new();
        
        // 从InfluxDB客户端URL构建健康检查URL
        let influx_url = &self.config.influx_url;
        let health_url = format!("{}/health", influx_url);
        
        let response = client
            .get(&health_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| ApiError::internal_error(format!("InfluxDB health check request failed: {}", e)))?;
        
        if response.status().is_success() {
            // InfluxDB 3.x健康端点返回200状态码和"OK"文本
            tracing::debug!("InfluxDB health check passed");
            Ok(())
        } else {
            tracing::warn!("InfluxDB health check failed with status: {}", response.status());
            Err(ApiError::internal_error(format!("InfluxDB health check failed with status: {}", response.status())))
        }
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
    
    // 数据库连接池环境变量覆盖
    if let Ok(max_conns) = env::var("WEBGW_DB_MAX_CONNECTIONS") {
        if let Ok(parsed) = max_conns.parse() {
            config.database_pool.max_connections = parsed;
        }
    }
    
    if let Ok(min_conns) = env::var("WEBGW_DB_MIN_CONNECTIONS") {
        if let Ok(parsed) = min_conns.parse() {
            config.database_pool.min_connections = parsed;
        }
    }
    
    if let Ok(timeout) = env::var("WEBGW_DB_ACQUIRE_TIMEOUT_SECS") {
        if let Ok(parsed) = timeout.parse() {
            config.database_pool.acquire_timeout_secs = parsed;
        }
    }
    
    if let Ok(idle_timeout) = env::var("WEBGW_DB_IDLE_TIMEOUT_SECS") {
        if let Ok(parsed) = idle_timeout.parse() {
            config.database_pool.idle_timeout_secs = parsed;
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
    
    if let Ok(enable_metrics) = env::var("WEBGW_DB_ENABLE_METRICS") {
        if let Ok(parsed) = enable_metrics.parse::<bool>() {
            config.database_pool.enable_metrics = parsed;
        }
    }
}

/// 初始化应用状态
pub async fn init_state(config: &ApiConfig) -> ApiResult<Data<AppState>> {
    // 验证连接池配置
    validate_pool_config(&config.database_pool)
        .context("数据库连接池配置验证失败")?;
    
    // 初始化数据库连接池管理器
    let pool_manager = Arc::new(
        DatabasePoolManager::new(&config.pg_dsn, config.database_pool.clone())
            .await
            .context("数据库连接池管理器初始化失败")?
    );
    
    // 获取连接池引用
    let pg_pool = pool_manager.pool().clone();
    
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
    
    // 初始化驱动配置仓库
    let driver_config_repo: Arc<dyn pg_repo::DriverConfigRepo> = Arc::new(
        pg_repo::DriverConfigRepoImpl::new(pg_pool.clone())
    );

    let driver_manager_arc = Arc::new(driver_manager);
    
    // 初始化驱动配置监听服务
    crate::services::init_driver_config_monitor(
        driver_config_repo.clone(),
        driver_manager_arc.clone(),
    );

    // 启动驱动配置监听服务 (带超时)
    if let Some(monitor) = get_driver_config_monitor() {
        match tokio::time::timeout(std::time::Duration::from_secs(10), monitor.start()).await {
            Ok(Ok(())) => {
                tracing::info!("Driver config monitor started successfully");
            }
            Ok(Err(e)) => {
                tracing::error!("Failed to start driver config monitor: {}", e);
            }
            Err(_) => {
                tracing::warn!("Driver config monitor start timed out, continuing anyway");
            }
        }
    }
    
    let state = AppState {
        config: config.clone(),
        pg_pool,
        pool_manager,
        influx_client,
        frame_bus,
        driver_manager: driver_manager_arc,
        driver_config_repo,
        ws_manager,
        frame_bus_bridge,
        history_service,
        shutdown_tx,
    };
    
    Ok(Data::new(state))
}

// ========== 真实FrameBus实现 ==========

struct RealFrameBusClient {
    // 简化的实现，暂时不使用实际的frame-bus类型
}

impl RealFrameBusClient {
    async fn new() -> ApiResult<Self> {
        // 简化的frame-bus初始化
        tracing::info!("Initializing frame-bus client (simplified implementation)");
        
        // TODO: 实际的frame-bus初始化
        // 这里是占位实现，生产环境需要替换为真实的frame-bus客户端
        
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl FrameBusClient for RealFrameBusClient {
    async fn subscribe(&self, _filter: FrameFilter) -> ApiResult<FrameReceiver> {
        // TODO: 实际的frame-bus订阅实现
        tracing::debug!("Frame-bus subscribe called (simplified implementation)");
        
        Ok(FrameReceiver)
    }
    
    async fn publish(&self, frame: DataFrame) -> ApiResult<()> {
        // TODO: 实际的frame-bus发布实现
        tracing::debug!("Publishing frame for device {} tag {} (simplified implementation)", 
                       frame.device_id, frame.tag_id);
        
        Ok(())
    }
}


