//! response.rs —— 通用HTTP响应辅助（最小实现）

use actix_web::HttpResponse;
use serde::Serialize;

pub fn ok_json<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(data)
}

pub fn created_json<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Created().json(data)
}

pub fn no_content() -> HttpResponse {
    HttpResponse::NoContent().finish()
}
