//! routes/tags.rs —— 点位管理REST API
//!
//! 提供点位的完整CRUD操作：
//! - POST /api/v1/tags: 创建点位
//! - GET /api/v1/tags: 分页列表查询  
//! - GET /api/v1/tags/{id}: 获取点位详情
//! - PUT /api/v1/tags/{id}: 更新点位信息
//! - DELETE /api/v1/tags/{id}: 删除点位
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::bootstrap::AppState;
use crate::dto::{TagCreateReq, TagPatchReq, TagQuery, TagVO, PagedResponse, TagDataType};
use crate::error::{ApiError, ApiResult};
use actix_web::{web, HttpResponse, Scope};
use pg_repo::{TagFilter, TagRepoImpl, NewTag, TagUpdate, DbTagDataType, TagRepo};
use tracing::{info, error, instrument};
use uuid::Uuid;
use utoipa::ToSchema;

/// 点位管理路由作用域
pub fn scope() -> Scope {
    web::scope("/tags")
        .route("", web::post().to(create_tag))
        .route("", web::get().to(list_tags))
        .route("/{id}", web::get().to(get_tag))
        .route("/{id}", web::put().to(update_tag))
        .route("/{id}", web::delete().to(delete_tag))
}

/// datapoints别名路由作用域 (指向tags)
pub fn scope_as_datapoints() -> Scope {
    web::scope("/datapoints")
        .route("", web::post().to(create_tag))
        .route("", web::get().to(list_tags))
        .route("/{id}", web::get().to(get_tag))
        .route("/{id}", web::put().to(update_tag))
        .route("/{id}", web::delete().to(delete_tag))
}

/// 创建点位
/// 
/// 创建新点位，关联到指定设备
#[utoipa::path(
    post,
    path = "/api/v1/tags",
    request_body = TagCreateReq,
    responses(
        (status = 201, description = "Tag created successfully", body = TagVO),
        (status = 400, description = "Bad request"),
        (status = 409, description = "Tag address already exists for this device")
    ),
    tag = "Tags"
)]
#[instrument(skip(state))]
async fn create_tag(
    state: web::Data<AppState>,
    req: web::Json<TagCreateReq>,
) -> ApiResult<HttpResponse> {
    info!("Creating tag: {} for device {}", req.name, req.device_id);
    
    let tag_repo = TagRepoImpl::new(state.pg_pool.clone());
    
    // 检查设备中地址是否已存在
    if tag_repo.address_exists(req.device_id, &req.address, None).await? {
        return Err(ApiError::conflict("Tag address already exists for this device"));
    }
    
    // 创建点位实体
    let tag_id = Uuid::new_v4();
    let new_tag = NewTag {
        id: tag_id,
        device_id: req.device_id,
        name: req.name.clone(),
        address: req.address.clone(),
        data_type: convert_tag_data_type(&req.data_type)?,
        scaling: req.scaling,
        offset: req.offset,
        unit: req.unit.clone(),
        description: req.description.clone(),
        enabled: req.enabled,
    };
    
    // 保存到数据库
    let tag = tag_repo.create(new_tag).await?;
    
    let response = TagVO {
        id: tag.id,
        device_id: tag.device_id,
        name: tag.name,
        address: tag.address,
        data_type: convert_db_tag_data_type(&tag.data_type)?,
        scaling: tag.scaling,
        offset: tag.offset,
        unit: tag.unit,
        description: tag.description,
        enabled: tag.enabled,
        created_at: tag.created_at,
    };
    
    info!("Tag created successfully: {}", tag_id);
    Ok(HttpResponse::Created().json(response))
}

/// 查询点位列表
/// 
/// 支持按设备、数据类型过滤和分页查询
#[utoipa::path(
    get,
    path = "/api/v1/tags",
    params(TagQuery),
    responses(
        (status = 200, description = "Tag list retrieved successfully", body = PagedResponse<TagVO>)
    ),
    tag = "Tags"
)]
#[instrument(skip(state))]
async fn list_tags(
    state: web::Data<AppState>,
    query: web::Query<TagQuery>,
) -> ApiResult<HttpResponse> {
    let tag_repo = TagRepoImpl::new(state.pg_pool.clone());
    
    let page = query.page.unwrap_or(1);
    let size = query.size.unwrap_or(20).min(100); // 限制最大分页大小
    let offset = (page - 1) * size;
    
    let filter = TagFilter {
        device_id: query.device_id,
        data_type: query.data_type.as_ref().map(|d| convert_tag_data_type(d)).transpose()?,
        enabled: query.enabled,
        limit: Some(size as i64),
        offset: Some(offset as i64),
    };
    
    // 并行查询列表和总数
    let filter_for_count = TagFilter {
        device_id: filter.device_id,
        data_type: filter.data_type.clone(),
        enabled: filter.enabled,
        limit: None,
        offset: None,
    };
    let (tags, total) = tokio::try_join!(
        tag_repo.list(filter),
        tag_repo.count(filter_for_count)
    )?;
    
    let items: Vec<TagVO> = tags.into_iter()
        .map(|tag| Ok(TagVO {
            id: tag.id,
            device_id: tag.device_id,
            name: tag.name,
            address: tag.address,
            data_type: convert_db_tag_data_type(&tag.data_type)?,
            scaling: tag.scaling,
            offset: tag.offset,
            unit: tag.unit,
            description: tag.description,
            enabled: tag.enabled,
            created_at: tag.created_at,
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

/// 获取点位详情
/// 
/// 根据ID获取点位的详细信息
#[utoipa::path(
    get,
    path = "/api/v1/tags/{id}",
    params(
        ("id" = Uuid, Path, description = "Tag ID")
    ),
    responses(
        (status = 200, description = "Tag retrieved successfully", body = TagVO),
        (status = 404, description = "Tag not found")
    ),
    tag = "Tags"
)]
#[instrument(skip(state))]
async fn get_tag(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let tag_id = path.into_inner();
    let tag_repo = TagRepoImpl::new(state.pg_pool.clone());
    
    let tag = tag_repo.get_by_id(tag_id).await?
        .ok_or_else(|| ApiError::not_found("Tag"))?;
    
    let response = TagVO {
        id: tag.id,
        device_id: tag.device_id,
        name: tag.name,
        address: tag.address,
        data_type: convert_db_tag_data_type(&tag.data_type)?,
        scaling: tag.scaling,
        offset: tag.offset,
        unit: tag.unit,
        description: tag.description,
        enabled: tag.enabled,
        created_at: tag.created_at,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// 更新点位
/// 
/// 更新点位信息，支持部分字段更新
#[utoipa::path(
    put,
    path = "/api/v1/tags/{id}",
    params(
        ("id" = Uuid, Path, description = "Tag ID")
    ),
    request_body = TagPatchReq,
    responses(
        (status = 200, description = "Tag updated successfully", body = TagVO),
        (status = 404, description = "Tag not found"),
        (status = 409, description = "Tag address already exists for this device")
    ),
    tag = "Tags"
)]
#[instrument(skip(state))]
async fn update_tag(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: web::Json<TagPatchReq>,
) -> ApiResult<HttpResponse> {
    let tag_id = path.into_inner();
    let tag_repo = TagRepoImpl::new(state.pg_pool.clone());
    
    // 获取现有点位信息
    let existing_tag = tag_repo.get_by_id(tag_id).await?
        .ok_or_else(|| ApiError::not_found("Tag"))?;
    
    // 检查地址冲突（如果地址有变化）
    if let Some(address) = &req.address {
        if tag_repo.address_exists(existing_tag.device_id, address, Some(tag_id)).await? {
            return Err(ApiError::conflict("Tag address already exists for this device"));
        }
    }
    
    // 构建更新对象
    let update = TagUpdate {
        name: req.name.clone(),
        address: req.address.clone(),
        data_type: req.data_type.as_ref().map(|d| convert_tag_data_type(d)).transpose()?,
        scaling: req.scaling,
        offset: req.offset,
        unit: req.unit.clone(),
        description: req.description.clone(),
        enabled: req.enabled,
    };
    
    // 更新数据库
    let updated_tag = tag_repo.update(tag_id, update).await?
        .ok_or_else(|| ApiError::not_found("Tag"))?;
    
    let response = TagVO {
        id: updated_tag.id,
        device_id: updated_tag.device_id,
        name: updated_tag.name,
        address: updated_tag.address,
        data_type: convert_db_tag_data_type(&updated_tag.data_type)?,
        scaling: updated_tag.scaling,
        offset: updated_tag.offset,
        unit: updated_tag.unit,
        description: updated_tag.description,
        enabled: updated_tag.enabled,
        created_at: updated_tag.created_at,
    };
    
    info!("Tag updated successfully: {}", tag_id);
    Ok(HttpResponse::Ok().json(response))
}

/// 删除点位
/// 
/// 删除指定的点位数据
#[utoipa::path(
    delete,
    path = "/api/v1/tags/{id}",
    params(
        ("id" = Uuid, Path, description = "Tag ID")
    ),
    responses(
        (status = 204, description = "Tag deleted successfully"),
        (status = 404, description = "Tag not found")
    ),
    tag = "Tags"
)]
#[instrument(skip(state))]
async fn delete_tag(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let tag_id = path.into_inner();
    let tag_repo = TagRepoImpl::new(state.pg_pool.clone());
    
    // 检查点位是否存在
    if !tag_repo.exists(tag_id).await? {
        return Err(ApiError::not_found("Tag"));
    }
    
    // 删除点位
    let deleted = tag_repo.delete(tag_id).await?;
    
    if !deleted {
        return Err(ApiError::not_found("Tag"));
    }
    
    info!("Tag deleted successfully: {}", tag_id);
    Ok(HttpResponse::NoContent().finish())
}

// ========== 辅助函数 ==========

fn convert_tag_data_type(data_type: &TagDataType) -> ApiResult<DbTagDataType> {
    match data_type {
        TagDataType::Float => Ok(DbTagDataType::Float),
        TagDataType::Int => Ok(DbTagDataType::Int),
        TagDataType::Bool => Ok(DbTagDataType::Bool),
        TagDataType::String => Ok(DbTagDataType::String),
    }
}

fn convert_db_tag_data_type(data_type: &DbTagDataType) -> ApiResult<TagDataType> {
    match data_type {
        DbTagDataType::Float => Ok(TagDataType::Float),
        DbTagDataType::Int => Ok(TagDataType::Int),
        DbTagDataType::Bool => Ok(TagDataType::Bool),
        DbTagDataType::String => Ok(TagDataType::String),
    }
}