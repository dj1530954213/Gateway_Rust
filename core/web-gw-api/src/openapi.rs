//! openapi.rs —— OpenAPI文档配置与生成
//!
//! 提供完整的API文档生成，包括：
//! - Swagger UI界面
//! - OpenAPI JSON/YAML导出
//! - TypeScript客户端生成支持
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::dto::*;
use crate::error::ApiError;
use actix_web::{web, HttpResponse, Result, Scope};
use std::fs;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI文档配置
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Gateway Rust API",
        version = "1.0.0",
        description = "IoT Gateway REST API - 提供设备管理、点位监控、驱动管理、历史数据查询和报警管理等功能",
        contact(
            name = "Gateway Rust Team",
            email = "support@gateway-rust.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    paths(
        // 健康检查
        crate::routes::health::health_check,
        crate::routes::health::readiness_check,
        crate::routes::health::liveness_check,
        
        // 设备管理
        crate::routes::devices::create_device,
        crate::routes::devices::list_devices,
        crate::routes::devices::get_device,
        crate::routes::devices::update_device,
        crate::routes::devices::delete_device,
        
        // 点位管理
        crate::routes::tags::create_tag,
        crate::routes::tags::list_tags,
        crate::routes::tags::get_tag,
        crate::routes::tags::update_tag,
        crate::routes::tags::delete_tag,
        
        // 驱动管理
        crate::routes::drivers::upload_driver,
        crate::routes::drivers::list_drivers,
        crate::routes::drivers::get_driver_details,
        crate::routes::drivers::search_drivers,
        crate::routes::drivers::get_registry_overview,
        crate::routes::drivers::reload_driver,
        crate::routes::drivers::unload_driver,
        
        // 历史数据查询
        crate::routes::history::query_time_series,
        crate::routes::history::query_aggregated,
        crate::routes::history::export_csv,
    ),
    components(
        schemas(
            // 基础响应类型
            ApiError,
            PagedResponse<DeviceVO>,
            PagedResponse<TagVO>,
            PagedResponse<DriverInfo>,
            HealthResponse,
            
            // 设备相关类型
            DeviceVO, DeviceCreateReq, DevicePatchReq, DeviceQuery,
            ProtocolKind,
            
            // 点位相关类型  
            TagVO, TagCreateReq, TagPatchReq, TagQuery,
            TagDataType,
            
            // 驱动相关类型
            DriverInfo, DriverLoadHistory, DriverReloadRequest,
            LoadResult, LoadAction,
            
            // 历史数据相关类型
            TimeSeriesQuery, TimeSeriesPoint, TimeSeriesData,
            AggregatedQuery, AggregatedPoint, AggregatedData,
            AggregationType, CsvExportRequest,
        )
    ),
    tags(
        (name = "Health", description = "健康检查和系统状态"),
        (name = "Devices", description = "设备管理 - 创建、查询、更新、删除设备"),
        (name = "Tags", description = "点位管理 - 创建、查询、更新、删除数据点位"),
        (name = "Drivers", description = "驱动管理 - 上传、重载、卸载协议驱动"),
        (name = "History", description = "历史数据 - 时间序列查询、聚合分析、数据导出"),
        (name = "Alerts", description = "报警管理 - 规则配置、事件查询、通知设置（代理到alert-engine）"),
    ),
    modifiers(&SecurityAddon),
    external_docs(
        url = "https://github.com/your-org/gateway-rust",
        description = "Gateway Rust 项目文档"
    )
)]
pub struct ApiDoc;

/// 安全配置插件
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
        }
    }
}

/// 创建OpenAPI路由
pub fn create_routes() -> Scope {
    web::scope("/docs")
        .route("/openapi.json", web::get().to(serve_openapi_json))
        .route("/openapi.yaml", web::get().to(serve_openapi_yaml))
        .service(
            SwaggerUi::new("/docs/swagger-ui/{_:.*}")
                .url("/docs/openapi.json", ApiDoc::openapi())
        )
}

/// 提供OpenAPI JSON格式文档
async fn serve_openapi_json() -> Result<HttpResponse> {
    let openapi = ApiDoc::openapi();
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(openapi))
}

/// 提供OpenAPI YAML格式文档
async fn serve_openapi_yaml() -> Result<HttpResponse> {
    let openapi = ApiDoc::openapi();
    match serde_yaml::to_string(&openapi) {
        Ok(yaml) => Ok(HttpResponse::Ok()
            .content_type("application/x-yaml")
            .body(yaml)),
        Err(e) => Ok(HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": "Failed to serialize OpenAPI to YAML",
                "details": e.to_string()
            })))
    }
}

/// 生成OpenAPI文档到文件
pub fn generate_openapi_file(output_path: &str) -> anyhow::Result<()> {
    let openapi = ApiDoc::openapi();
    
    // 生成JSON文件
    let json_content = serde_json::to_string_pretty(&openapi)?;
    fs::write(format!("{}/openapi.json", output_path), json_content)?;
    
    // 生成YAML文件
    let yaml_content = serde_yaml::to_string(&openapi)?;
    fs::write(format!("{}/openapi.yaml", output_path), yaml_content)?;
    
    println!("OpenAPI documentation generated:");
    println!("  - {}/openapi.json", output_path);
    println!("  - {}/openapi.yaml", output_path);
    
    Ok(())
}

/// CLI命令：生成OpenAPI文档
pub fn generate_docs_command(output_dir: Option<String>) -> anyhow::Result<()> {
    let output_path = output_dir.unwrap_or_else(|| "docs/api".to_string());
    
    // 确保输出目录存在
    fs::create_dir_all(&output_path)?;
    
    generate_openapi_file(&output_path)?;
    
    println!("\n🎉 API文档生成完成！");
    println!("📖 在线查看: http://localhost:8080/docs/swagger-ui/");
    println!("📄 JSON文档: http://localhost:8080/docs/openapi.json");
    println!("📋 YAML文档: http://localhost:8080/docs/openapi.yaml");
    
    Ok(())
}