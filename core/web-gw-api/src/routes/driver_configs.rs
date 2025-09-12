//! routes/driver_configs.rs —— 驱动配置管理REST API  
//!
//! - scope(): `/api/v1/driver-configs`
//! - 依赖注入：AppState<DriverConfigRepo>
//! - 包含驱动配置CRUD操作
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::{dto::*, error::ApiError};
use actix_web::{
    web::{self, Data, Path, Query, Json},
    HttpResponse, Responder, Result,
};
use uuid::Uuid;
use tracing::{info, error, warn};
use utoipa::OpenApi;

/// 驱动配置OpenAPI文档
#[derive(OpenApi)]
#[openapi(
    paths(
        create_driver_config,
        list_driver_configs,
        get_driver_config,
        update_driver_config,
        delete_driver_config,
        start_driver_config,
        stop_driver_config,
        restart_driver_config,
        get_driver_config_status,
    ),
    components(schemas(
        DriverConfigVO,
        DriverConfigCreateReq,
        DriverConfigUpdateReq,
        DriverConfigQuery,
        DriverConfigResponse,
        DriverConfigListResponse,
        DriverLifecycleResponse,
        DriverConfigStatusResponse,
        DriverConfigStatusVO,
    ))
)]
pub struct DriverConfigsApiDoc;

/// 配置驱动配置管理路由
pub fn scope() -> actix_web::Scope {
    web::scope("/driver-configs")
        .route("", web::post().to(create_driver_config))
        .route("", web::get().to(list_driver_configs))
        .route("/{id}", web::get().to(get_driver_config))
        .route("/{id}", web::put().to(update_driver_config))
        .route("/{id}", web::delete().to(delete_driver_config))
        // 生命周期管理API
        .route("/{id}/start", web::post().to(start_driver_config))
        .route("/{id}/stop", web::post().to(stop_driver_config))
        .route("/{id}/restart", web::post().to(restart_driver_config))
        .route("/{id}/status", web::get().to(get_driver_config_status))
}

/// 创建驱动配置
#[utoipa::path(
    post,
    path = "/api/v1/driver-configs",
    request_body = DriverConfigCreateReq,
    responses(
        (status = 201, description = "创建成功", body = DriverConfigResponse),
        (status = 400, description = "请求参数错误", body = ApiErrorResponse),
        (status = 409, description = "驱动名称已存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn create_driver_config(
    json: Json<DriverConfigCreateReq>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let req = json.into_inner();
    
    info!("Creating driver config: {}", req.name);

    // 检查驱动名称是否已存在
    if app_state.driver_config_repo.is_driver_name_exists(&req.name, None).await
        .map_err(|e| {
            error!("Failed to check driver name existence: {}", e);
            ApiError::internal_error("Database query failed")
        })? 
    {
        return Err(ApiError::conflict(format!("Driver name '{}' already exists", req.name)));
    }

    // 转换为数据库模型
    let new_config = pg_repo::NewDriverConfig {
        name: req.name,
        description: req.description,
        protocol: req.protocol,
        connection_type: req.connection_type,
        enabled: req.enabled,
        config: req.config,
        scan_interval: req.scan_interval,
        timeout: req.timeout,
        max_concurrent: req.max_concurrent,
        batch_size: req.batch_size,
        max_retries: req.max_retries,
        retry_interval: req.retry_interval,
        exponential_backoff: req.exponential_backoff,
        max_retry_interval: req.max_retry_interval,
        log_level: req.log_level,
        enable_request_log: req.enable_request_log,
        enable_response_log: req.enable_response_log,
        max_log_size: req.max_log_size,
        enable_ssl: req.enable_ssl,
        verify_certificate: req.verify_certificate,
        client_cert_path: req.client_cert_path,
        client_key_path: req.client_key_path,
    };

    // 创建驱动配置
    let created_config = app_state.driver_config_repo.create_driver_config(new_config).await
        .map_err(|e| {
            error!("Failed to create driver config: {}", e);
            ApiError::internal_error("Failed to create driver config")
        })?;

    // 转换为VO
    let config_vo = DriverConfigVO {
        id: created_config.id,
        name: created_config.name,
        description: created_config.description,
        protocol: created_config.protocol,
        connection_type: created_config.connection_type,
        enabled: created_config.enabled,
        config: created_config.config,
        scan_interval: created_config.scan_interval,
        timeout: created_config.timeout,
        max_concurrent: created_config.max_concurrent,
        batch_size: created_config.batch_size,
        max_retries: created_config.max_retries,
        retry_interval: created_config.retry_interval,
        exponential_backoff: created_config.exponential_backoff,
        max_retry_interval: created_config.max_retry_interval,
        log_level: created_config.log_level,
        enable_request_log: created_config.enable_request_log,
        enable_response_log: created_config.enable_response_log,
        max_log_size: created_config.max_log_size,
        enable_ssl: created_config.enable_ssl,
        verify_certificate: created_config.verify_certificate,
        client_cert_path: created_config.client_cert_path,
        client_key_path: created_config.client_key_path,
        created_at: created_config.created_at,
        updated_at: created_config.updated_at,
    };

    let response = DriverConfigResponse {
        driver_config: config_vo,
    };

    info!("Successfully created driver config: {}", response.driver_config.id);
    Ok(HttpResponse::Created().json(response))
}

/// 查询驱动配置列表
#[utoipa::path(
    get,
    path = "/api/v1/driver-configs",
    params(DriverConfigQuery),
    responses(
        (status = 200, description = "查询成功", body = DriverConfigListResponse),
        (status = 400, description = "请求参数错误", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn list_driver_configs(
    query: Query<DriverConfigQuery>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let query = query.into_inner();
    
    // 构建过滤器
    let filter = pg_repo::DriverConfigFilter {
        protocol: query.protocol,
        enabled: query.enabled,
        name_contains: query.name_contains,
        limit: query.page_size.map(|s| s as i64),
        offset: query.page.and_then(|p| query.page_size.map(|s| ((p - 1) * s) as i64)),
    };

    // 查询驱动配置列表
    let configs = app_state.driver_config_repo.list_driver_configs(filter.clone()).await
        .map_err(|e| {
            error!("Failed to list driver configs: {}", e);
            ApiError::internal_error("Failed to query driver configs")
        })?;

    // 查询总数
    let total = app_state.driver_config_repo.count_driver_configs(filter).await
        .map_err(|e| {
            error!("Failed to count driver configs: {}", e);
            ApiError::internal_error("Failed to count driver configs")
        })?;

    // 转换为VO
    let config_vos: Vec<DriverConfigVO> = configs.into_iter().map(|config| {
        DriverConfigVO {
            id: config.id,
            name: config.name,
            description: config.description,
            protocol: config.protocol,
            connection_type: config.connection_type,
            enabled: config.enabled,
            config: config.config,
            scan_interval: config.scan_interval,
            timeout: config.timeout,
            max_concurrent: config.max_concurrent,
            batch_size: config.batch_size,
            max_retries: config.max_retries,
            retry_interval: config.retry_interval,
            exponential_backoff: config.exponential_backoff,
            max_retry_interval: config.max_retry_interval,
            log_level: config.log_level,
            enable_request_log: config.enable_request_log,
            enable_response_log: config.enable_response_log,
            max_log_size: config.max_log_size,
            enable_ssl: config.enable_ssl,
            verify_certificate: config.verify_certificate,
            client_cert_path: config.client_cert_path,
            client_key_path: config.client_key_path,
            created_at: config.created_at,
            updated_at: config.updated_at,
        }
    }).collect();

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

    let response = DriverConfigListResponse {
        driver_configs: config_vos,
        total,
        page,
        page_size,
        total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 获取驱动配置详情
#[utoipa::path(
    get,
    path = "/api/v1/driver-configs/{id}",
    params(
        ("id" = Uuid, Path, description = "驱动配置ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = DriverConfigResponse),
        (status = 404, description = "驱动配置不存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn get_driver_config(
    path: Path<Uuid>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let config_id = path.into_inner();
    
    let config = app_state.driver_config_repo.get_driver_config(config_id).await
        .map_err(|e| {
            error!("Failed to get driver config {}: {}", config_id, e);
            ApiError::internal_error("Database query failed")
        })?
        .ok_or_else(|| ApiError::not_found(format!("Driver config not found: {}", config_id)))?;

    // 转换为VO
    let config_vo = DriverConfigVO {
        id: config.id,
        name: config.name,
        description: config.description,
        protocol: config.protocol,
        connection_type: config.connection_type,
        enabled: config.enabled,
        config: config.config,
        scan_interval: config.scan_interval,
        timeout: config.timeout,
        max_concurrent: config.max_concurrent,
        batch_size: config.batch_size,
        max_retries: config.max_retries,
        retry_interval: config.retry_interval,
        exponential_backoff: config.exponential_backoff,
        max_retry_interval: config.max_retry_interval,
        log_level: config.log_level,
        enable_request_log: config.enable_request_log,
        enable_response_log: config.enable_response_log,
        max_log_size: config.max_log_size,
        enable_ssl: config.enable_ssl,
        verify_certificate: config.verify_certificate,
        client_cert_path: config.client_cert_path,
        client_key_path: config.client_key_path,
        created_at: config.created_at,
        updated_at: config.updated_at,
    };

    let response = DriverConfigResponse {
        driver_config: config_vo,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 更新驱动配置
#[utoipa::path(
    put,
    path = "/api/v1/driver-configs/{id}",
    params(
        ("id" = Uuid, Path, description = "驱动配置ID")
    ),
    request_body = DriverConfigUpdateReq,
    responses(
        (status = 200, description = "更新成功", body = DriverConfigResponse),
        (status = 404, description = "驱动配置不存在", body = ApiErrorResponse),
        (status = 400, description = "请求参数错误", body = ApiErrorResponse),
        (status = 409, description = "驱动名称已存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn update_driver_config(
    path: Path<Uuid>,
    json: Json<DriverConfigUpdateReq>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let config_id = path.into_inner();
    let req = json.into_inner();
    
    info!("Updating driver config: {}", config_id);

    // 如果更新名称，检查是否与其他驱动冲突
    if let Some(ref name) = req.name {
        if app_state.driver_config_repo.is_driver_name_exists(name, Some(config_id)).await
            .map_err(|e| {
                error!("Failed to check driver name existence: {}", e);
                ApiError::internal_error("Database query failed")
            })? 
        {
            return Err(ApiError::conflict(format!("Driver name '{}' already exists", name)));
        }
    }

    // 转换为数据库更新模型
    let update = pg_repo::DriverConfigUpdate {
        name: req.name,
        description: req.description,
        protocol: req.protocol,
        connection_type: req.connection_type,
        enabled: req.enabled,
        config: req.config,
        scan_interval: req.scan_interval,
        timeout: req.timeout,
        max_concurrent: req.max_concurrent,
        batch_size: req.batch_size,
        max_retries: req.max_retries,
        retry_interval: req.retry_interval,
        exponential_backoff: req.exponential_backoff,
        max_retry_interval: req.max_retry_interval,
        log_level: req.log_level,
        enable_request_log: req.enable_request_log,
        enable_response_log: req.enable_response_log,
        max_log_size: req.max_log_size,
        enable_ssl: req.enable_ssl,
        verify_certificate: req.verify_certificate,
        client_cert_path: req.client_cert_path,
        client_key_path: req.client_key_path,
    };

    // 更新驱动配置
    let updated_config = app_state.driver_config_repo.update_driver_config(config_id, update).await
        .map_err(|e| {
            error!("Failed to update driver config {}: {}", config_id, e);
            ApiError::internal_error("Failed to update driver config")
        })?
        .ok_or_else(|| ApiError::not_found(format!("Driver config not found: {}", config_id)))?;

    // 转换为VO
    let config_vo = DriverConfigVO {
        id: updated_config.id,
        name: updated_config.name,
        description: updated_config.description,
        protocol: updated_config.protocol,
        connection_type: updated_config.connection_type,
        enabled: updated_config.enabled,
        config: updated_config.config,
        scan_interval: updated_config.scan_interval,
        timeout: updated_config.timeout,
        max_concurrent: updated_config.max_concurrent,
        batch_size: updated_config.batch_size,
        max_retries: updated_config.max_retries,
        retry_interval: updated_config.retry_interval,
        exponential_backoff: updated_config.exponential_backoff,
        max_retry_interval: updated_config.max_retry_interval,
        log_level: updated_config.log_level,
        enable_request_log: updated_config.enable_request_log,
        enable_response_log: updated_config.enable_response_log,
        max_log_size: updated_config.max_log_size,
        enable_ssl: updated_config.enable_ssl,
        verify_certificate: updated_config.verify_certificate,
        client_cert_path: updated_config.client_cert_path,
        client_key_path: updated_config.client_key_path,
        created_at: updated_config.created_at,
        updated_at: updated_config.updated_at,
    };

    let response = DriverConfigResponse {
        driver_config: config_vo,
    };

    info!("Successfully updated driver config: {}", config_id);
    Ok(HttpResponse::Ok().json(response))
}

/// 删除驱动配置
#[utoipa::path(
    delete,
    path = "/api/v1/driver-configs/{id}",
    params(
        ("id" = Uuid, Path, description = "驱动配置ID")
    ),
    responses(
        (status = 204, description = "删除成功"),
        (status = 404, description = "驱动配置不存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn delete_driver_config(
    path: Path<Uuid>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let config_id = path.into_inner();
    
    info!("Deleting driver config: {}", config_id);

    let deleted = app_state.driver_config_repo.delete_driver_config(config_id).await
        .map_err(|e| {
            error!("Failed to delete driver config {}: {}", config_id, e);
            ApiError::internal_error("Failed to delete driver config")
        })?;

    if !deleted {
        return Err(ApiError::not_found(format!("Driver config not found: {}", config_id)));
    }

    info!("Successfully deleted driver config: {}", config_id);
    Ok(HttpResponse::NoContent().finish())
}

/// 启动驱动配置实例
#[utoipa::path(
    post,
    path = "/api/v1/driver-configs/{id}/start",
    params(
        ("id" = Uuid, Path, description = "驱动配置ID")
    ),
    responses(
        (status = 200, description = "启动成功", body = DriverLifecycleResponse),
        (status = 404, description = "驱动配置不存在", body = ApiErrorResponse),
        (status = 409, description = "驱动已在运行中", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn start_driver_config(
    path: Path<Uuid>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let config_id = path.into_inner();
    
    info!("Starting driver config: {}", config_id);

    // 获取驱动配置
    let config = app_state.driver_config_repo.get_driver_config(config_id).await
        .map_err(|e| {
            error!("Failed to get driver config {}: {}", config_id, e);
            ApiError::internal_error("Database query failed")
        })?
        .ok_or_else(|| ApiError::not_found(format!("Driver config not found: {}", config_id)))?;

    if !config.enabled {
        return Err(ApiError::bad_request("Cannot start a disabled driver config"));
    }

    // 生成管理器中的驱动ID
    let manager_driver_id = manager_driver_id(&config);

    // 如果未加载，尝试按协议加载静态/动态驱动（最小实现：Modbus）
    let state_opt = app_state.driver_manager.get_driver_state(&manager_driver_id).await;
    if state_opt.is_none() {
        if let Err(e) = try_load_driver_for_config(&app_state, &config, &manager_driver_id).await {
            warn!("Failed to pre-load driver for config {}: {}", config_id, e);
        }
    }

    // 启动驱动
    if let Err(e) = app_state.driver_manager.start_driver(&manager_driver_id).await {
        warn!("start_driver failed for {}: {}", manager_driver_id, e);
    }

    // 检查驱动是否成功启动
    let status = get_config_driver_status(&config, &app_state).await?;

    let response = DriverLifecycleResponse {
        config_id,
        config_name: config.name,
        operation: "start".to_string(),
        success: status.running,
        message: if status.running {
            "Driver config started successfully".to_string()
        } else {
            "Driver config start initiated, but not yet running".to_string()
        },
        current_status: Some(status),
    };

    info!("Driver config {} start operation completed", config_id);
    Ok(HttpResponse::Ok().json(response))
}

/// 停止驱动配置实例
#[utoipa::path(
    post,
    path = "/api/v1/driver-configs/{id}/stop",
    params(
        ("id" = Uuid, Path, description = "驱动配置ID")
    ),
    responses(
        (status = 200, description = "停止成功", body = DriverLifecycleResponse),
        (status = 404, description = "驱动配置不存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn stop_driver_config(
    path: Path<Uuid>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let config_id = path.into_inner();
    
    info!("Stopping driver config: {}", config_id);

    // 获取驱动配置
    let config = app_state.driver_config_repo.get_driver_config(config_id).await
        .map_err(|e| {
            error!("Failed to get driver config {}: {}", config_id, e);
            ApiError::internal_error("Database query failed")
        })?
        .ok_or_else(|| ApiError::not_found(format!("Driver config not found: {}", config_id)))?;

    // 生成管理器中的驱动ID
    let manager_driver_id = manager_driver_id(&config);
    // 停止驱动（若不存在则忽略）
    if let Err(e) = app_state.driver_manager.stop_driver(&manager_driver_id).await {
        warn!("stop_driver failed for {}: {}", manager_driver_id, e);
    }

    // 检查驱动是否成功停止
    let status = get_config_driver_status(&config, &app_state).await?;

    let response = DriverLifecycleResponse {
        config_id,
        config_name: config.name,
        operation: "stop".to_string(),
        success: !status.running,
        message: if !status.running {
            "Driver config stopped successfully".to_string()
        } else {
            "Driver config stop initiated, but still running".to_string()
        },
        current_status: Some(status),
    };

    info!("Driver config {} stop operation completed", config_id);
    Ok(HttpResponse::Ok().json(response))
}

/// 重启驱动配置实例
#[utoipa::path(
    post,
    path = "/api/v1/driver-configs/{id}/restart",
    params(
        ("id" = Uuid, Path, description = "驱动配置ID")
    ),
    responses(
        (status = 200, description = "重启成功", body = DriverLifecycleResponse),
        (status = 404, description = "驱动配置不存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn restart_driver_config(
    path: Path<Uuid>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let config_id = path.into_inner();
    
    info!("Restarting driver config: {}", config_id);

    // 获取驱动配置
    let config = app_state.driver_config_repo.get_driver_config(config_id).await
        .map_err(|e| {
            error!("Failed to get driver config {}: {}", config_id, e);
            ApiError::internal_error("Database query failed")
        })?
        .ok_or_else(|| ApiError::not_found(format!("Driver config not found: {}", config_id)))?;

    if !config.enabled {
        return Err(ApiError::bad_request("Cannot restart a disabled driver config"));
    }

    // 生成管理器中的驱动ID
    let manager_driver_id = manager_driver_id(&config);
    // 停止
    if let Err(e) = app_state.driver_manager.stop_driver(&manager_driver_id).await {
        warn!("stop_driver (restart) failed for {}: {}", manager_driver_id, e);
    }
    // 短暂等待
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    // 若未加载则尝试加载
    if app_state.driver_manager.get_driver_state(&manager_driver_id).await.is_none() {
        if let Err(e) = try_load_driver_for_config(&app_state, &config, &manager_driver_id).await {
            warn!("Failed to pre-load driver for restart {}: {}", config_id, e);
        }
    }
    // 启动
    if let Err(e) = app_state.driver_manager.start_driver(&manager_driver_id).await {
        warn!("start_driver (restart) failed for {}: {}", manager_driver_id, e);
    }

    // 检查驱动状态
    let status = get_config_driver_status(&config, &app_state).await?;

    let response = DriverLifecycleResponse {
        config_id,
        config_name: config.name,
        operation: "restart".to_string(),
        success: status.running,
        message: if status.running {
            "Driver config restarted successfully".to_string()
        } else {
            "Driver config restart initiated, but not yet running".to_string()
        },
        current_status: Some(status),
    };

    info!("Driver config {} restart operation completed", config_id);
    Ok(HttpResponse::Ok().json(response))
}

/// 获取驱动配置状态
#[utoipa::path(
    get,
    path = "/api/v1/driver-configs/{id}/status",
    params(
        ("id" = Uuid, Path, description = "驱动配置ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = DriverConfigStatusResponse),
        (status = 404, description = "驱动配置不存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "driver-configs"
)]
async fn get_driver_config_status(
    path: Path<Uuid>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let config_id = path.into_inner();
    
    // 获取驱动配置
    let config = app_state.driver_config_repo.get_driver_config(config_id).await
        .map_err(|e| {
            error!("Failed to get driver config {}: {}", config_id, e);
            ApiError::internal_error("Database query failed")
        })?
        .ok_or_else(|| ApiError::not_found(format!("Driver config not found: {}", config_id)))?;

    let status = get_config_driver_status(&config, &app_state).await?;

    let response = DriverConfigStatusResponse {
        config_id,
        config_name: config.name,
        status,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 获取驱动配置的运行状态
async fn get_config_driver_status(
    config: &pg_repo::DriverConfig,
    app_state: &Data<crate::bootstrap::AppState>,
) -> Result<DriverConfigStatusVO, ApiError> {
    // 生成管理器驱动ID（与之前约定一致）
    let expected_driver_id = manager_driver_id(config);

    // 从驱动管理器获取状态
    let driver_state = app_state.driver_manager.get_driver_state(&expected_driver_id).await;

    let running = matches!(driver_state, Some(driver_manager::DriverState::Active));

    let status_message = if !config.enabled {
        "Disabled".to_string()
    } else if running {
        "Running".to_string()
    } else {
        format!("Not running (state: {:?})", driver_state)
    };

    Ok(DriverConfigStatusVO {
        running,
        enabled: config.enabled,
        managed_driver_id: Some(expected_driver_id),
        driver_state: driver_state.map(|s| format!("{:?}", s)),
        status_message,
        last_checked: chrono::Utc::now(),
    })
}

/// 由配置生成管理器中的驱动ID（命名约定）
fn manager_driver_id(config: &pg_repo::DriverConfig) -> String {
    format!("config_{}_{}", config.name, config.id)
}

/// 按配置尝试加载驱动（最小实现：支持 Modbus 静态驱动）
async fn try_load_driver_for_config(
    app_state: &Data<crate::bootstrap::AppState>,
    config: &pg_repo::DriverConfig,
    manager_driver_id: &str,
) -> anyhow::Result<()> {
    // 仅在启用时加载
    if !config.enabled { return Ok(()); }

    // 选择驱动名称（静态）
    let protocol = config.protocol.to_lowercase();
    let driver_name = if protocol.contains("modbus") {
        "modbus-tcp"
    } else {
        return Err(anyhow::anyhow!("Protocol '{}' not supported for auto-load", config.protocol));
    };

    // 传递原始配置JSON给驱动
    let cfg = config.config.clone();
    app_state.driver_manager.load_static_driver(manager_driver_id.to_string(), driver_name, cfg).await
}