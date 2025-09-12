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
use crate::error::ErrorResponse;
use actix_web::{web, HttpResponse, Result};
use actix_web::web::ServiceConfig;
use std::fs;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;
use tracing::info;

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
        crate::routes::history::query_points,
        crate::routes::history::query_stats,
    ),
    components(
        schemas(
            // 基础响应类型
            ErrorResponse,
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
            DriverInfo, DriverReloadRequest,
            
            // 历史数据相关类型
            HistoryQuery, HistoryPointVO, HistoryStatsVO, HistoryExportRequest,
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

/// 注册 OpenAPI 路由（使用绝对路径，避免 scope 嵌套引起的路径混淆）
pub fn configure(cfg: &mut ServiceConfig) {
    info!("Registering OpenAPI routes at /docs ...");
    cfg
        // OpenAPI 文档导出
        .route("/docs/openapi.json", web::get().to(serve_openapi_json))
        .route("/docs/openapi.yaml", web::get().to(serve_openapi_yaml))
        // 兜底：直接返回一个基于 CDN 的 Swagger UI HTML 页面（含有/无尾斜杠两种）
        .route("/docs", web::get().to(serve_swagger_ui))
        .route("/docs/", web::get().to(serve_swagger_ui))
        .route("/docs/swagger-ui", web::get().to(serve_swagger_ui))
        .route("/docs/swagger-ui/", web::get().to(serve_swagger_ui))
        ;
}

/// 提供OpenAPI JSON格式文档
async fn serve_openapi_json() -> Result<HttpResponse> {
    let openapi = ApiDoc::openapi();
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(openapi))
}

/// 兜底的 Swagger UI 页面（使用 CDN 资源），避免静态资源路由异常导致 404
pub async fn serve_swagger_ui() -> Result<HttpResponse> {
    let html = r#"<!DOCTYPE html>
<html lang=\"en\"> 
  <head>
    <meta charset=\"utf-8\" />
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
    <title>Gateway Rust API Docs</title>
    <link rel=\"stylesheet\" href=\"https://unpkg.com/swagger-ui-dist@5/swagger-ui.css\" />
    <style> body { margin: 0; padding: 0 } #swagger-ui { height: 100vh; } </style>
  </head>
  <body>
    <div id=\"swagger-ui\"></div>
    <script src=\"https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js\"></script>
    <script>
      window.addEventListener('load', () => {
        window.ui = SwaggerUIBundle({
          url: '/docs/openapi.json',
          dom_id: '#swagger-ui',
          presets: [SwaggerUIBundle.presets.apis],
          layout: 'BaseLayout',
        });
      });
    </script>
  </body>
</html>\n"#;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
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