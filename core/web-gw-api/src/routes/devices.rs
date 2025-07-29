//! routes/devices.rs —— 设备管理REST API
//!
//! 提供设备的完整CRUD操作：
//! - POST /api/v1/devices: 创建设备并注册驱动
//! - GET /api/v1/devices: 分页列表查询
//! - GET /api/v1/devices/{id}: 获取设备详情
//! - PUT /api/v1/devices/{id}: 更新设备信息
//! - DELETE /api/v1/devices/{id}: 删除设备及关联数据
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::bootstrap::AppState;
use crate::dto::{DeviceCreateReq, DevicePatchReq, DeviceQuery, DeviceVO, PagedResponse, ProtocolKind};
use crate::error::{ApiError, ApiResult};
use actix_web::{web, HttpResponse, Scope};
use pg_repo::{DeviceFilter, DeviceRepoImpl, NewDevice, DeviceUpdate, DbProtocolKind};
use tracing::{info, error, instrument};
use uuid::Uuid;
use utoipa::ToSchema;

/// 设备管理路由作用域
pub fn scope() -> Scope {
    web::scope("/devices")
        .route("", web::post().to(create_device))
        .route("", web::get().to(list_devices))
        .route("/{id}", web::get().to(get_device))
        .route("/{id}", web::put().to(update_device))
        .route("/{id}", web::delete().to(delete_device))
}

/// 创建设备
/// 
/// 创建新设备并自动注册至对应协议驱动
#[utoipa::path(
    post,
    path = "/api/v1/devices",
    request_body = DeviceCreateReq,
    responses(
        (status = 201, description = "Device created successfully", body = DeviceVO),
        (status = 400, description = "Bad request"),
        (status = 409, description = "Device name already exists")
    ),
    tag = "Devices"
)]
#[instrument(skip(state))]
async fn create_device(
    state: web::Data<AppState>,
    req: web::Json<DeviceCreateReq>,
) -> ApiResult<HttpResponse> {
    info!("Creating device: {}", req.name);
    
    let device_repo = DeviceRepoImpl::new(state.pg_pool.clone());
    
    // 检查设备名称是否已存在
    if device_repo.name_exists(&req.name, None).await? {
        return Err(ApiError::conflict("Device name already exists"));
    }
    
    // 创建设备实体
    let device_id = Uuid::new_v4();
    let new_device = NewDevice {
        id: device_id,
        name: req.name.clone(),
        protocol: convert_protocol_kind(&req.protocol)?,
        location: req.location.clone(),
        endpoint: req.endpoint.clone(),
        config: req.config.clone(),
        enabled: req.enabled,
    };
    
    // 保存到数据库
    let device = device_repo.create(new_device).await?;
    
    // 注册到驱动管理器
    if device.enabled {
        if let Err(e) = state.driver_manager.register_device(req.protocol.clone(), device_id).await {
            error!("Failed to register device to driver manager: {}", e);
            // 不阻塞设备创建，只记录错误
        }
    }
    
    let response = DeviceVO {
        id: device.id,
        name: device.name,
        protocol: convert_db_protocol_kind(&device.protocol)?,
        location: device.location,
        endpoint: device.endpoint,
        config: device.config,
        enabled: device.enabled,
        created_at: device.created_at,
        updated_at: device.updated_at,
    };
    
    info!("Device created successfully: {}", device_id);
    Ok(HttpResponse::Created().json(response))
}

/// 查询设备列表
/// 
/// 支持按协议、状态过滤和分页查询
#[utoipa::path(
    get,
    path = "/api/v1/devices",
    params(DeviceQuery),
    responses(
        (status = 200, description = "Device list retrieved successfully", body = PagedResponse<DeviceVO>)
    ),
    tag = "Devices"
)]
#[instrument(skip(state))]
async fn list_devices(
    state: web::Data<AppState>,
    query: web::Query<DeviceQuery>,
) -> ApiResult<HttpResponse> {
    let device_repo = DeviceRepoImpl::new(state.pg_pool.clone());
    
    let page = query.page.unwrap_or(1);
    let size = query.size.unwrap_or(20).min(100); // 限制最大分页大小
    let offset = (page - 1) * size;
    
    let filter = DeviceFilter {
        protocol: query.protocol.as_ref().map(|p| convert_protocol_kind(p)).transpose()?,
        enabled: query.enabled,
        limit: Some(size as i64),
        offset: Some(offset as i64),
    };
    
    // 并行查询列表和总数
    let (devices, total) = tokio::try_join!(
        device_repo.list(filter.clone()),
        device_repo.count(filter)
    )?;
    
    let items: Vec<DeviceVO> = devices.into_iter()
        .map(|device| Ok(DeviceVO {
            id: device.id,
            name: device.name,
            protocol: convert_db_protocol_kind(&device.protocol)?,
            location: device.location,
            endpoint: device.endpoint,
            config: device.config,
            enabled: device.enabled,
            created_at: device.created_at,
            updated_at: device.updated_at,
        }))
        .collect::<Result<Vec<_>, ApiError>>()?;
    
    let pages = (total as f64 / size as f64).ceil() as u64;
    
    let response = PagedResponse {
        items,
        total: total as u64,
        page,
        size,
        pages,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// 获取设备详情
/// 
/// 根据ID获取设备的详细信息
#[utoipa::path(
    get,
    path = "/api/v1/devices/{id}",
    params(
        ("id" = Uuid, Path, description = "Device ID")
    ),
    responses(
        (status = 200, description = "Device retrieved successfully", body = DeviceVO),
        (status = 404, description = "Device not found")
    ),
    tag = "Devices"
)]
#[instrument(skip(state))]
async fn get_device(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let device_id = path.into_inner();
    let device_repo = DeviceRepoImpl::new(state.pg_pool.clone());
    
    let device = device_repo.get_by_id(device_id).await?
        .ok_or_else(|| ApiError::not_found("Device"))?;
    
    let response = DeviceVO {
        id: device.id,
        name: device.name,
        protocol: convert_db_protocol_kind(&device.protocol)?,
        location: device.location,
        endpoint: device.endpoint,
        config: device.config,
        enabled: device.enabled,
        created_at: device.created_at,
        updated_at: device.updated_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// 更新设备
/// 
/// 更新设备信息，如果协议发生变化会重新注册驱动
#[utoipa::path(
    put,
    path = "/api/v1/devices/{id}",
    params(
        ("id" = Uuid, Path, description = "Device ID")
    ),
    request_body = DevicePatchReq,
    responses(
        (status = 200, description = "Device updated successfully", body = DeviceVO),
        (status = 404, description = "Device not found"),
        (status = 409, description = "Device name already exists")
    ),
    tag = "Devices"
)]
#[instrument(skip(state))]
async fn update_device(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: web::Json<DevicePatchReq>,
) -> ApiResult<HttpResponse> {
    let device_id = path.into_inner();
    let device_repo = DeviceRepoImpl::new(state.pg_pool.clone());
    
    // 获取现有设备信息
    let existing_device = device_repo.get_by_id(device_id).await?
        .ok_or_else(|| ApiError::not_found("Device"))?;
    
    // 检查名称冲突
    if let Some(name) = &req.name {
        if device_repo.name_exists(name, Some(device_id)).await? {
            return Err(ApiError::conflict("Device name already exists"));
        }
    }
    
    // 构建更新对象
    let update = DeviceUpdate {
        name: req.name.clone(),
        location: req.location.clone(),
        endpoint: req.endpoint.clone(),
        config: req.config.clone(),
        enabled: req.enabled,
    };
    
    // 更新数据库
    let updated_device = device_repo.update(device_id, update).await?
        .ok_or_else(|| ApiError::not_found("Device"))?;
    
    // 处理启用状态变化
    match (existing_device.enabled, updated_device.enabled) {
        (false, true) => {
            // 启用设备：注册到驱动
            if let Err(e) = state.driver_manager.register_device(
                convert_db_protocol_kind(&updated_device.protocol)?, 
                device_id
            ).await {
                error!("Failed to register device to driver manager: {}", e);
            }
        },
        (true, false) => {
            // 禁用设备：从驱动解除注册
            if let Err(e) = state.driver_manager.detach_device(device_id).await {
                error!("Failed to detach device from driver manager: {}", e);
            }
        },
        _ => {} // 状态无变化
    }
    
    let response = DeviceVO {
        id: updated_device.id,
        name: updated_device.name,
        protocol: convert_db_protocol_kind(&updated_device.protocol)?,
        location: updated_device.location,
        endpoint: updated_device.endpoint,
        config: updated_device.config,
        enabled: updated_device.enabled,
        created_at: updated_device.created_at,
        updated_at: updated_device.updated_at,
    };
    
    info!("Device updated successfully: {}", device_id);
    Ok(HttpResponse::Ok().json(response))
}

/// 删除设备
/// 
/// 删除设备及其所有关联的点位数据，并从驱动中解除注册
#[utoipa::path(
    delete,
    path = "/api/v1/devices/{id}",
    params(
        ("id" = Uuid, Path, description = "Device ID")
    ),
    responses(
        (status = 204, description = "Device deleted successfully"),
        (status = 404, description = "Device not found")
    ),
    tag = "Devices"
)]
#[instrument(skip(state))]
async fn delete_device(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let device_id = path.into_inner();
    let device_repo = DeviceRepoImpl::new(state.pg_pool.clone());
    
    // 检查设备是否存在
    if !device_repo.exists(device_id).await? {
        return Err(ApiError::not_found("Device"));
    }
    
    // 从驱动管理器中解除注册
    if let Err(e) = state.driver_manager.detach_device(device_id).await {
        error!("Failed to detach device from driver manager: {}", e);
        // 继续删除，不阻塞操作
    }
    
    // 删除设备（级联删除关联的点位）
    let deleted = device_repo.delete(device_id).await?;
    
    if !deleted {
        return Err(ApiError::not_found("Device"));
    }
    
    info!("Device deleted successfully: {}", device_id);
    Ok(HttpResponse::NoContent().finish())
}

// ========== 辅助函数 ==========

fn convert_protocol_kind(protocol: &ProtocolKind) -> ApiResult<DbProtocolKind> {
    match protocol {
        ProtocolKind::ModbusTcp => Ok(DbProtocolKind::ModbusTcp),
        ProtocolKind::OpcUa => Ok(DbProtocolKind::OpcUa),
        ProtocolKind::Mqtt => Ok(DbProtocolKind::Mqtt),
    }
}

fn convert_db_protocol_kind(protocol: &DbProtocolKind) -> ApiResult<ProtocolKind> {
    match protocol {
        DbProtocolKind::ModbusTcp => Ok(ProtocolKind::ModbusTcp),
        DbProtocolKind::OpcUa => Ok(ProtocolKind::OpcUa),
        DbProtocolKind::Mqtt => Ok(ProtocolKind::Mqtt),
    }
}