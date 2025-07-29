//! routes/drivers.rs —— 驱动管理REST API
//!
//! - scope(): `/api/v1/drivers`
//! - 依赖注入：AppState<DriverManager>
//! - 包含驱动上传、列表、热重载、卸载等功能
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::{dto::*, error::ApiError};
use actix_multipart::Multipart;
use actix_web::{
    web::{self, Data, Path, Query},
    HttpResponse, Responder, Result,
};
use driver_manager::{
    DriverQueryRequest, DriverQueryFilter, DriverSortBy, DriverKind,
};
use futures_util::TryStreamExt;
use std::io::Write;
use tokio::fs;
use tracing::{error, info, warn};
use utoipa::OpenApi;

/// 驱动管理OpenAPI文档
#[derive(OpenApi)]
#[openapi(
    paths(
        upload_driver,
        list_drivers,
        get_driver_details,
        search_drivers,
        get_registry_overview,
        reload_driver,
        unload_driver,
    ),
    components(schemas(
        DriverUploadResponse,
        DriverListQuery,
        DriverListResponse,
        DriverDetailResponse,
        DriverSearchQuery,
        DriverSearchResponse,
        RegistryOverviewResponse,
        DriverReloadRequest,
        DriverReloadResponse,
        DriverUnloadResponse,
        UnifiedDriverEntryVO,
        RegistryOverviewVO,
        DriverStatisticsVO,
    ))
)]
pub struct DriversApiDoc;

/// 配置驱动管理路由
pub fn scope() -> actix_web::Scope {
    web::scope("/drivers")
        .route("", web::post().to(upload_driver))
        .route("", web::get().to(list_drivers))
        .route("/search", web::get().to(search_drivers))
        .route("/overview", web::get().to(get_registry_overview))
        .route("/{driver_id}", web::get().to(get_driver_details))
        .route("/{driver_id}/reload", web::post().to(reload_driver))
        .route("/{driver_id}", web::delete().to(unload_driver))
}

/// 上传驱动文件
///
/// 上传动态驱动文件（.so/.dll/.dylib）到服务器并自动加载
#[utoipa::path(
    post,
    path = "/api/v1/drivers",
    request_body(content = String, description = "驱动文件（multipart/form-data）", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "上传成功", body = DriverUploadResponse),
        (status = 400, description = "请求参数错误", body = ApiErrorResponse),
        (status = 413, description = "文件过大", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "drivers"
)]
async fn upload_driver(
    mut payload: Multipart,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    info!("Uploading driver file");

    // 获取上传目录
    let upload_dir = std::env::var("DRIVERS_DIR").unwrap_or_else(|_| "./drivers".to_string());
    
    // 确保上传目录存在
    if let Err(e) = fs::create_dir_all(&upload_dir).await {
        error!("Failed to create upload directory: {}", e);
        return Err(ApiError::InternalServerError(
            "Failed to create upload directory".to_string()
        ));
    }

    let mut uploaded_files = Vec::new();
    let max_file_size = 50 * 1024 * 1024; // 50MB限制

    // 处理multipart数据
    while let Some(mut field) = payload.try_next().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        ApiError::BadRequest("Invalid multipart data".to_string())
    })? {
        let content_disposition = field.content_disposition();
        
        if let Some(filename) = content_disposition.get_filename() {
            // 验证文件扩展名
            if !is_valid_driver_file(filename) {
                return Err(ApiError::BadRequest(
                    format!("Invalid driver file extension. Expected .so, .dll, or .dylib, got: {}", filename)
                ));
            }

            let filepath = std::path::PathBuf::from(&upload_dir).join(filename);
            
            info!("Uploading driver file: {}", filepath.display());

            // 创建文件
            let mut file = web::block(move || std::fs::File::create(filepath.clone()))
                .await
                .map_err(|e| {
                    error!("Failed to create file: {}", e);
                    ApiError::InternalServerError("Failed to create file".to_string())
                })?
                .map_err(|e| {
                    error!("Failed to create file: {}", e);
                    ApiError::InternalServerError("Failed to create file".to_string())
                })?;

            let mut file_size = 0;

            // 写入文件数据
            while let Some(chunk) = field.try_next().await.map_err(|e| {
                error!("Failed to read file chunk: {}", e);
                ApiError::BadRequest("Failed to read file data".to_string())
            })? {
                file_size += chunk.len();
                
                // 检查文件大小限制
                if file_size > max_file_size {
                    // 删除部分上传的文件
                    let _ = std::fs::remove_file(&filepath);
                    return Err(ApiError::BadRequest(
                        format!("File too large. Maximum size is {}MB", max_file_size / 1024 / 1024)
                    ));
                }

                file = web::block(move || file.write_all(&chunk).map(|_| file))
                    .await
                    .map_err(|e| {
                        error!("Failed to write file chunk: {}", e);
                        ApiError::InternalServerError("Failed to write file".to_string())
                    })?
                    .map_err(|e| {
                        error!("Failed to write file chunk: {}", e);
                        ApiError::InternalServerError("Failed to write file".to_string())
                    })?;
            }

            // 同步文件到磁盘
            web::block(move || file.sync_all())
                .await
                .map_err(|e| {
                    error!("Failed to sync file: {}", e);
                    ApiError::InternalServerError("Failed to sync file".to_string())
                })?
                .map_err(|e| {
                    error!("Failed to sync file: {}", e);
                    ApiError::InternalServerError("Failed to sync file".to_string())
                })?;

            // 尝试加载驱动
            match app_state.driver_manager.dynamic_loader().load_driver(&filepath).await {
                Ok(driver_id) => {
                    info!("Successfully loaded driver: {} from {}", driver_id, filename);
                    uploaded_files.push(DriverUploadInfo {
                        filename: filename.to_string(),
                        driver_id,
                        file_size: file_size as u64,
                        status: "loaded".to_string(),
                        message: None,
                    });
                }
                Err(e) => {
                    error!("Failed to load driver from {}: {}", filename, e);
                    // 保留文件但标记为加载失败
                    uploaded_files.push(DriverUploadInfo {
                        filename: filename.to_string(),
                        driver_id: format!("failed_{}", filename),
                        file_size: file_size as u64,
                        status: "failed".to_string(),
                        message: Some(e.to_string()),
                    });
                }
            }
        }
    }

    if uploaded_files.is_empty() {
        return Err(ApiError::BadRequest(
            "No valid driver files found in upload".to_string()
        ));
    }

    let response = DriverUploadResponse {
        success: uploaded_files.iter().any(|f| f.status == "loaded"),
        uploaded_files,
        message: "Driver upload completed".to_string(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 查询驱动列表
///
/// 分页查询已注册的驱动列表，支持过滤和排序
#[utoipa::path(
    get,
    path = "/api/v1/drivers",
    params(DriverListQuery),
    responses(
        (status = 200, description = "查询成功", body = DriverListResponse),
        (status = 400, description = "请求参数错误", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "drivers"
)]
async fn list_drivers(
    query: Query<DriverListQuery>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let query = query.into_inner();
    
    // 构建查询请求
    let filter = if query.has_filters() {
        Some(DriverQueryFilter {
            driver_kind: query.driver_kind.map(|k| match k.as_str() {
                "static" => DriverKind::Static,
                "dyn" => DriverKind::Dyn,
                "wasm" => DriverKind::Wasm,
                _ => DriverKind::Dyn,
            }),
            protocol: query.protocol,
            status: query.status,
            name_contains: query.name_contains,
            active_only: query.active_only.unwrap_or(false),
            error_only: query.error_only.unwrap_or(false),
        })
    } else {
        None
    };

    let sort_by = query.sort_by.map(|s| match s.as_str() {
        "name" => DriverSortBy::Name,
        "type" => DriverSortBy::Type,
        "status" => DriverSortBy::Status,
        "loaded_at" => DriverSortBy::LoadedAt,
        "success_rate" => DriverSortBy::SuccessRate,
        _ => DriverSortBy::Name,
    });

    let request = DriverQueryRequest {
        filter,
        sort_by,
        descending: query.descending.unwrap_or(false),
        page: query.page.unwrap_or(1).max(1),
        page_size: query.page_size.unwrap_or(20).min(100).max(1),
    };

    // 执行查询
    let query_result = app_state.driver_manager.query_registry(&request);

    // 转换为VO
    let drivers: Vec<UnifiedDriverEntryVO> = query_result.drivers
        .into_iter()
        .map(|entry| UnifiedDriverEntryVO {
            driver_id: entry.driver_id,
            driver_kind: entry.driver_kind,
            name: entry.name,
            version: entry.version,
            protocol: entry.protocol,
            status: entry.status,
            description: entry.description,
            features: entry.features,
            loaded_at: entry.loaded_at,
            file_path: entry.file_path.map(|p| p.to_string_lossy().to_string()),
            stats: entry.stats.map(|s| DriverStatisticsVO {
                attached_devices: s.attached_devices,
                read_count: s.read_count,
                write_count: s.write_count,
                error_count: s.error_count,
                avg_response_time_ms: s.avg_response_time_ms,
                success_rate: s.success_rate,
            }),
        })
        .collect();

    let response = DriverListResponse {
        drivers,
        total: query_result.total,
        page: query_result.page,
        page_size: query_result.page_size,
        total_pages: query_result.total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 获取驱动详情
///
/// 根据驱动ID获取详细信息
#[utoipa::path(
    get,
    path = "/api/v1/drivers/{driver_id}",
    params(
        ("driver_id" = String, Path, description = "驱动ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = DriverDetailResponse),
        (status = 404, description = "驱动不存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "drivers"
)]
async fn get_driver_details(
    path: Path<String>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let driver_id = path.into_inner();
    
    match app_state.driver_manager.get_driver_details(&driver_id) {
        Some(entry) => {
            let driver_vo = UnifiedDriverEntryVO {
                driver_id: entry.driver_id,
                driver_kind: entry.driver_kind,
                name: entry.name,
                version: entry.version,
                protocol: entry.protocol,
                status: entry.status,
                description: entry.description,
                features: entry.features,
                loaded_at: entry.loaded_at,
                file_path: entry.file_path.map(|p| p.to_string_lossy().to_string()),
                stats: entry.stats.map(|s| DriverStatisticsVO {
                    attached_devices: s.attached_devices,
                    read_count: s.read_count,
                    write_count: s.write_count,
                    error_count: s.error_count,
                    avg_response_time_ms: s.avg_response_time_ms,
                    success_rate: s.success_rate,
                }),
            };

            let response = DriverDetailResponse {
                driver: driver_vo,
            };

            Ok(HttpResponse::Ok().json(response))
        }
        None => Err(ApiError::NotFound(format!("Driver not found: {}", driver_id))),
    }
}

/// 搜索驱动
///
/// 根据关键词搜索驱动
#[utoipa::path(
    get,
    path = "/api/v1/drivers/search",
    params(DriverSearchQuery),
    responses(
        (status = 200, description = "搜索成功", body = DriverSearchResponse),
        (status = 400, description = "请求参数错误", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "drivers"
)]
async fn search_drivers(
    query: Query<DriverSearchQuery>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let query = query.into_inner();
    
    if query.q.is_empty() {
        return Err(ApiError::BadRequest("Search query cannot be empty".to_string()));
    }

    let results = app_state.driver_manager.search_drivers(&query.q);
    
    let drivers: Vec<UnifiedDriverEntryVO> = results
        .into_iter()
        .map(|entry| UnifiedDriverEntryVO {
            driver_id: entry.driver_id,
            driver_kind: entry.driver_kind,
            name: entry.name,
            version: entry.version,
            protocol: entry.protocol,
            status: entry.status,
            description: entry.description,
            features: entry.features,
            loaded_at: entry.loaded_at,
            file_path: entry.file_path.map(|p| p.to_string_lossy().to_string()),
            stats: entry.stats.map(|s| DriverStatisticsVO {
                attached_devices: s.attached_devices,
                read_count: s.read_count,
                write_count: s.write_count,
                error_count: s.error_count,
                avg_response_time_ms: s.avg_response_time_ms,
                success_rate: s.success_rate,
            }),
        })
        .collect();

    let response = DriverSearchResponse {
        query: query.q,
        results: drivers,
        total: drivers.len() as u32,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 获取注册表概览
///
/// 获取驱动注册表的统计概览信息
#[utoipa::path(
    get,
    path = "/api/v1/drivers/overview",
    responses(
        (status = 200, description = "获取成功", body = RegistryOverviewResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "drivers"
)]
async fn get_registry_overview(
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let overview = app_state.driver_manager.get_registry_overview();
    
    let overview_vo = RegistryOverviewVO {
        total_drivers: overview.total_drivers,
        static_drivers: overview.static_drivers,
        dynamic_drivers: overview.dynamic_drivers,
        running_drivers: overview.running_drivers,
        error_drivers: overview.error_drivers,
        protocol_stats: overview.protocol_stats,
        status_stats: overview.status_stats,
    };

    let response = RegistryOverviewResponse {
        overview: overview_vo,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 热重载驱动
///
/// 热重载指定的驱动，保持服务不中断
#[utoipa::path(
    post,
    path = "/api/v1/drivers/{driver_id}/reload",
    params(
        ("driver_id" = String, Path, description = "驱动ID")
    ),
    request_body = DriverReloadRequest,
    responses(
        (status = 200, description = "重载成功", body = DriverReloadResponse),
        (status = 404, description = "驱动不存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "drivers"
)]
async fn reload_driver(
    path: Path<String>,
    _request: web::Json<DriverReloadRequest>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let driver_id = path.into_inner();
    
    info!("Hot reloading driver: {}", driver_id);

    match app_state.driver_manager.reload_dynamic_driver(&driver_id).await {
        Ok(new_driver_id) => {
            info!("Successfully reloaded driver: {} -> {}", driver_id, new_driver_id);
            
            let response = DriverReloadResponse {
                success: true,
                old_driver_id: driver_id,
                new_driver_id: Some(new_driver_id),
                message: "Driver reloaded successfully".to_string(),
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("Failed to reload driver {}: {}", driver_id, e);
            
            let response = DriverReloadResponse {
                success: false,
                old_driver_id: driver_id,
                new_driver_id: None,
                message: format!("Failed to reload driver: {}", e),
            };

            Ok(HttpResponse::Ok().json(response))
        }
    }
}

/// 卸载驱动
///
/// 卸载指定的驱动，释放相关资源
#[utoipa::path(
    delete,
    path = "/api/v1/drivers/{driver_id}",
    params(
        ("driver_id" = String, Path, description = "驱动ID")
    ),
    responses(
        (status = 200, description = "卸载成功", body = DriverUnloadResponse),
        (status = 404, description = "驱动不存在", body = ApiErrorResponse),
        (status = 500, description = "服务器内部错误", body = ApiErrorResponse)
    ),
    tag = "drivers"
)]
async fn unload_driver(
    path: Path<String>,
    app_state: Data<crate::bootstrap::AppState>,
) -> Result<impl Responder, ApiError> {
    let driver_id = path.into_inner();
    
    info!("Unloading driver: {}", driver_id);

    match app_state.driver_manager.unload_dynamic_driver(&driver_id).await {
        Ok(()) => {
            info!("Successfully unloaded driver: {}", driver_id);
            
            let response = DriverUnloadResponse {
                success: true,
                driver_id: driver_id.clone(),
                message: "Driver unloaded successfully".to_string(),
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("Failed to unload driver {}: {}", driver_id, e);
            
            let response = DriverUnloadResponse {
                success: false,
                driver_id: driver_id.clone(),
                message: format!("Failed to unload driver: {}", e),
            };

            Ok(HttpResponse::Ok().json(response))
        }
    }
}

/// 验证是否为有效的驱动文件
fn is_valid_driver_file(filename: &str) -> bool {
    let filename = filename.to_lowercase();
    filename.ends_with(".so") || filename.ends_with(".dll") || filename.ends_with(".dylib")
}