//! routes/mod.rs —— API路由注册中心
//!
//! 统一注册所有API路由模块：
//! - health: 健康检查
//! - devices: 设备管理
//! - tags: 点位管理  
//! - drivers: 驱动管理
//! - driver_configs: 驱动配置管理
//! - history: 历史数据查询
//! - alerts: 报警管理
//! - telemetry_ws: WebSocket实时数据
//! - system: 系统管理
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod health;
pub mod devices;
pub mod tags;
pub mod drivers;
pub mod driver_configs;
pub mod history;
pub mod websocket;
pub mod alerts;
pub mod system;

use actix_web::web;

/// 配置所有API路由
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // 健康检查（无版本前缀）
        .service(health::scope())
        
        // 系统路由（无版本前缀）
        .service(system::scope())
        
        // API v1 路由组
        .service(
            web::scope("/api/v1")
                .service(devices::scope())
                .service(tags::scope())
                .service(tags::scope_as_datapoints()) // datapoints别名指向tags
                .service(drivers::scope())
                .service(driver_configs::scope())
                .configure(history::configure)
                .service(alerts::scope())
        )
        
        // WebSocket路由
        .service(websocket::scope());
}