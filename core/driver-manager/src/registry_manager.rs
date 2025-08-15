//! registry_manager.rs —— 统一驱动注册表管理器
//!
//! 提供统一的驱动注册表管理，整合静态驱动和动态驱动
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::{
    driver::{DriverKind},
    dynamic::{DynamicDriverInfo, DynamicDriverLoader},
    registry::StaticDriverRegistry,
};
use driver_sdk::abi::DriverStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// 统一驱动注册表条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDriverEntry {
    /// 驱动ID
    pub driver_id: String,
    /// 驱动类型
    pub driver_kind: DriverKind,
    /// 驱动名称
    pub name: String,
    /// 驱动版本
    pub version: String,
    /// 协议类型
    pub protocol: String,
    /// 当前状态
    pub status: String,
    /// 描述
    pub description: String,
    /// 特性列表
    pub features: Vec<String>,
    /// 加载时间
    pub loaded_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 文件路径（动态驱动）
    pub file_path: Option<std::path::PathBuf>,
    /// 统计信息
    pub stats: Option<DriverStatistics>,
}

/// 驱动统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverStatistics {
    /// 关联设备数
    pub attached_devices: u32,
    /// 读取次数
    pub read_count: u64,
    /// 写入次数
    pub write_count: u64,
    /// 错误次数
    pub error_count: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 成功率
    pub success_rate: f64,
}

/// 驱动查询过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverQueryFilter {
    /// 按驱动类型过滤
    pub driver_kind: Option<DriverKind>,
    /// 按协议过滤
    pub protocol: Option<String>,
    /// 按状态过滤
    pub status: Option<String>,
    /// 按名称模糊匹配
    pub name_contains: Option<String>,
    /// 仅活跃驱动
    pub active_only: bool,
    /// 仅有错误的驱动
    pub error_only: bool,
}

/// 驱动排序选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverSortBy {
    /// 按名称排序
    Name,
    /// 按类型排序
    Type,
    /// 按状态排序
    Status,
    /// 按加载时间排序
    LoadedAt,
    /// 按成功率排序
    SuccessRate,
}

/// 驱动分页查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverQueryRequest {
    /// 过滤条件
    pub filter: Option<DriverQueryFilter>,
    /// 排序字段
    pub sort_by: Option<DriverSortBy>,
    /// 是否降序
    pub descending: bool,
    /// 页码（从1开始）
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
}

/// 驱动分页查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverQueryResponse {
    /// 驱动列表
    pub drivers: Vec<UnifiedDriverEntry>,
    /// 总数
    pub total: u32,
    /// 当前页
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
    /// 总页数
    pub total_pages: u32,
}

/// 注册表概览统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryOverview {
    /// 总驱动数
    pub total_drivers: u32,
    /// 静态驱动数
    pub static_drivers: u32,
    /// 动态驱动数
    pub dynamic_drivers: u32,
    /// 运行中驱动数
    pub running_drivers: u32,
    /// 错误状态驱动数
    pub error_drivers: u32,
    /// 按协议统计
    pub protocol_stats: HashMap<String, u32>,
    /// 按状态统计
    pub status_stats: HashMap<String, u32>,
}

/// 统一驱动注册表管理器
pub struct RegistryManager {
    /// 静态驱动注册表
    static_registry: StaticDriverRegistry,
    /// 动态驱动加载器
    dynamic_loader: Option<DynamicDriverLoader>,
}

impl RegistryManager {
    /// 创建新的注册表管理器
    pub fn new() -> Self {
        Self {
            static_registry: StaticDriverRegistry::new(),
            dynamic_loader: None,
        }
    }

    /// 设置动态驱动加载器
    pub fn with_dynamic_loader(mut self, dynamic_loader: DynamicDriverLoader) -> Self {
        self.dynamic_loader = Some(dynamic_loader);
        self
    }

    /// 查询驱动列表
    /// 
    /// # Parameters
    /// * `request` – 查询请求
    /// 
    /// # Returns
    /// 分页查询结果
    pub fn query_drivers(&self, request: &DriverQueryRequest) -> DriverQueryResponse {
        debug!("Querying drivers with request: {:?}", request);

        // 收集所有驱动
        let mut all_drivers = Vec::new();

        // 添加静态驱动
        for name in self.static_registry.list() {
            let entry = UnifiedDriverEntry {
                driver_id: format!("static_{}", name),
                driver_kind: DriverKind::Static,
                name: name.clone(),
                version: "1.0.0".to_string(), // 静态驱动版本固定
                protocol: "unknown".to_string(), // 静态驱动协议需要从元数据获取
                status: "available".to_string(),
                description: format!("Static driver: {}", name),
                features: vec![],
                loaded_at: None,
                file_path: None,
                stats: None,
            };
            all_drivers.push(entry);
        }

        // 添加动态驱动
        if let Some(dynamic_loader) = &self.dynamic_loader {
            for driver_info in dynamic_loader.list_drivers() {
                let entry = self.convert_dynamic_to_unified(driver_info);
                all_drivers.push(entry);
            }
        }

        // 应用过滤器
        if let Some(filter) = &request.filter {
            all_drivers = self.apply_filter(all_drivers, filter);
        }

        // 排序
        if let Some(sort_by) = &request.sort_by {
            self.sort_drivers(&mut all_drivers, sort_by, request.descending);
        }

        // 分页
        let total = all_drivers.len() as u32;
        let total_pages = (total + request.page_size - 1) / request.page_size;
        
        let start = ((request.page - 1) * request.page_size) as usize;
        let end = (start + request.page_size as usize).min(all_drivers.len());
        
        let drivers = if start < all_drivers.len() {
            all_drivers[start..end].to_vec()
        } else {
            Vec::new()
        };

        DriverQueryResponse {
            drivers,
            total,
            page: request.page,
            page_size: request.page_size,
            total_pages,
        }
    }

    /// 获取注册表概览
    pub fn get_overview(&self) -> RegistryOverview {
        let mut overview = RegistryOverview {
            total_drivers: 0,
            static_drivers: 0,
            dynamic_drivers: 0,
            running_drivers: 0,
            error_drivers: 0,
            protocol_stats: HashMap::new(),
            status_stats: HashMap::new(),
        };

        // 统计静态驱动
        let static_count = self.static_registry.list().len() as u32;
        overview.static_drivers = static_count;
        overview.total_drivers += static_count;

        // 统计动态驱动
        if let Some(dynamic_loader) = &self.dynamic_loader {
            for driver_info in dynamic_loader.list_drivers() {
                overview.dynamic_drivers += 1;
                overview.total_drivers += 1;

                // 统计状态
                let status = self.convert_status(&driver_info.status);
                *overview.status_stats.entry(status.clone()).or_insert(0) += 1;

                if status == "running" {
                    overview.running_drivers += 1;
                } else if status == "error" {
                    overview.error_drivers += 1;
                }

                // 统计协议
                let protocol = driver_info.meta.protocol_name().to_string();
                *overview.protocol_stats.entry(protocol).or_insert(0) += 1;
            }
        }

        // 为静态驱动添加默认状态
        *overview.status_stats.entry("available".to_string()).or_insert(0) += static_count;

        info!("Registry overview: {} total drivers ({} static, {} dynamic)", 
              overview.total_drivers, overview.static_drivers, overview.dynamic_drivers);

        overview
    }

    /// 获取特定驱动详情
    pub fn get_driver_details(&self, driver_id: &str) -> Option<UnifiedDriverEntry> {
        // 检查是否为静态驱动
        if driver_id.starts_with("static_") {
            let name = driver_id.strip_prefix("static_").unwrap();
            if self.static_registry.list().iter().any(|n| n == name) {
                return Some(UnifiedDriverEntry {
                    driver_id: driver_id.to_string(),
                    driver_kind: DriverKind::Static,
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    protocol: "unknown".to_string(),
                    status: "available".to_string(),
                    description: format!("Static driver: {}", name),
                    features: vec![],
                    loaded_at: None,
                    file_path: None,
                    stats: None,
                });
            }
        }

        // 检查动态驱动
        if let Some(dynamic_loader) = &self.dynamic_loader {
            if let Some(driver_info) = dynamic_loader.get_driver_info(driver_id) {
                return Some(self.convert_dynamic_to_unified(driver_info));
            }
        }

        None
    }

    /// 搜索驱动
    pub fn search_drivers(&self, query: &str) -> Vec<UnifiedDriverEntry> {
        let filter = DriverQueryFilter {
            driver_kind: None,
            protocol: None,
            status: None,
            name_contains: Some(query.to_string()),
            active_only: false,
            error_only: false,
        };

        let request = DriverQueryRequest {
            filter: Some(filter),
            sort_by: Some(DriverSortBy::Name),
            descending: false,
            page: 1,
            page_size: 100, // 搜索返回前100个结果
        };

        self.query_drivers(&request).drivers
    }

    /// 将动态驱动信息转换为统一格式
    fn convert_dynamic_to_unified(&self, driver_info: DynamicDriverInfo) -> UnifiedDriverEntry {
        let stats = if driver_info.stats.read_count > 0 || driver_info.stats.write_count > 0 {
            Some(DriverStatistics {
                attached_devices: driver_info.stats.attached_devices,
                read_count: driver_info.stats.read_count,
                write_count: driver_info.stats.write_count,
                error_count: driver_info.stats.error_count,
                avg_response_time_ms: driver_info.stats.avg_response_time_ms,
                success_rate: driver_info.stats.success_rate(),
            })
        } else {
            None
        };

        let protocol_name = driver_info.meta.protocol_name().to_string();

        UnifiedDriverEntry {
            driver_id: driver_info.driver_id,
            driver_kind: DriverKind::Dyn,
            name: driver_info.meta.name,
            version: driver_info.meta.version,
            protocol: protocol_name.clone(),
            status: self.convert_status(&driver_info.status),
            description: driver_info.meta.description,
            features: vec![protocol_name],
            loaded_at: Some(driver_info.loaded_at),
            file_path: Some(driver_info.file_path),
            stats,
        }
    }

    /// 转换驱动状态
    fn convert_status(&self, status: &DriverStatus) -> String {
        match status {
            DriverStatus::Unloaded => "unloaded".to_string(),
            DriverStatus::Loading => "loading".to_string(),
            DriverStatus::Loaded => "loaded".to_string(),
            DriverStatus::Initializing => "initializing".to_string(),
            DriverStatus::Running => "running".to_string(),
            DriverStatus::Error(_) => "error".to_string(),
            DriverStatus::Unloading => "unloading".to_string(),
        }
    }

    /// 应用过滤器
    fn apply_filter(&self, mut drivers: Vec<UnifiedDriverEntry>, filter: &DriverQueryFilter) -> Vec<UnifiedDriverEntry> {
        drivers.retain(|driver| {
            // 按驱动类型过滤
            if let Some(kind) = &filter.driver_kind {
                if driver.driver_kind != *kind {
                    return false;
                }
            }

            // 按协议过滤
            if let Some(protocol) = &filter.protocol {
                if driver.protocol != *protocol {
                    return false;
                }
            }

            // 按状态过滤
            if let Some(status) = &filter.status {
                if driver.status != *status {
                    return false;
                }
            }

            // 按名称模糊匹配
            if let Some(name_pattern) = &filter.name_contains {
                if !driver.name.to_lowercase().contains(&name_pattern.to_lowercase()) {
                    return false;
                }
            }

            // 仅活跃驱动
            if filter.active_only && driver.status != "running" {
                return false;
            }

            // 仅错误驱动
            if filter.error_only && driver.status != "error" {
                return false;
            }

            true
        });

        drivers
    }

    /// 排序驱动列表
    fn sort_drivers(&self, drivers: &mut Vec<UnifiedDriverEntry>, sort_by: &DriverSortBy, descending: bool) {
        match sort_by {
            DriverSortBy::Name => {
                drivers.sort_by(|a, b| {
                    let cmp = a.name.cmp(&b.name);
                    if descending { cmp.reverse() } else { cmp }
                });
            }
            DriverSortBy::Type => {
                drivers.sort_by(|a, b| {
                    let cmp = format!("{:?}", a.driver_kind).cmp(&format!("{:?}", b.driver_kind));
                    if descending { cmp.reverse() } else { cmp }
                });
            }
            DriverSortBy::Status => {
                drivers.sort_by(|a, b| {
                    let cmp = a.status.cmp(&b.status);
                    if descending { cmp.reverse() } else { cmp }
                });
            }
            DriverSortBy::LoadedAt => {
                drivers.sort_by(|a, b| {
                    let cmp = a.loaded_at.cmp(&b.loaded_at);
                    if descending { cmp.reverse() } else { cmp }
                });
            }
            DriverSortBy::SuccessRate => {
                drivers.sort_by(|a, b| {
                    let a_rate = a.stats.as_ref().map(|s| s.success_rate).unwrap_or(0.0);
                    let b_rate = b.stats.as_ref().map(|s| s.success_rate).unwrap_or(0.0);
                    let cmp = a_rate.partial_cmp(&b_rate).unwrap_or(std::cmp::Ordering::Equal);
                    if descending { cmp.reverse() } else { cmp }
                });
            }
        }
    }
}

impl Default for RegistryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DriverQueryRequest {
    fn default() -> Self {
        Self {
            filter: None,
            sort_by: Some(DriverSortBy::Name),
            descending: false,
            page: 1,
            page_size: 20,
        }
    }
}

impl Default for DriverQueryFilter {
    fn default() -> Self {
        Self {
            driver_kind: None,
            protocol: None,
            status: None,
            name_contains: None,
            active_only: false,
            error_only: false,
        }
    }
}