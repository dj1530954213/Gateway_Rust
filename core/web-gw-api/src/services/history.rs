//! history.rs —— 历史数据查询服务
//!
//! - InfluxDB时间序列查询封装
//! - 数据聚合与统计功能  
//! - CSV导出支持
//! - 分页与性能优化
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::dto::{HistoryQuery, HistoryPointVO, HistoryStatsVO, HistoryExportRequest};
use crate::error::{ApiError, ApiResult};
use anyhow::Context;
use chrono::{DateTime, Utc};
use influxdb2::{Client, FromDataPoint};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 历史数据查询服务
#[derive(Clone)]
pub struct HistoryService {
    client: Client,
    org: String,
    bucket: String,
}

/// InfluxDB查询结果数据点
#[derive(Debug, Clone, Serialize, Deserialize, FromDataPoint)]
pub struct InfluxDataPoint {
    #[serde(rename = "_time")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "_value")]
    pub value: f64,
    #[serde(rename = "device_id")]
    pub device_id: String,
    #[serde(rename = "tag_id")]
    pub tag_id: String,
    #[serde(rename = "unit")]
    pub unit: Option<String>,
}

/// 聚合统计结果
#[derive(Debug, Clone, Serialize, Deserialize, FromDataPoint)]
pub struct InfluxStatsPoint {
    #[serde(rename = "_time")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "min")]
    pub min_value: Option<f64>,
    #[serde(rename = "max")]
    pub max_value: Option<f64>,
    #[serde(rename = "mean")]
    pub avg_value: Option<f64>,
    #[serde(rename = "count")]
    pub count: Option<i64>,
    #[serde(rename = "device_id")]
    pub device_id: String,
    #[serde(rename = "tag_id")]
    pub tag_id: String,
}

impl HistoryService {
    /// 创建历史数据查询服务
    pub fn new(client: Client, org: String, bucket: String) -> Self {
        Self { client, org, bucket }
    }

    /// 查询历史数据点
    /// 
    /// # Parameters
    /// * `query` – 查询参数，包含时间范围、设备ID、点位ID等
    /// 
    /// # Returns
    /// * `Ok(Vec<HistoryPointVO>)` – 查询结果数据点列表
    /// * `Err(ApiError)` – 查询失败
    /// 
    /// # Example
    /// ```rust
    /// let query = HistoryQuery {
    ///     device_id: Some(uuid!("...")),
    ///     tag_id: Some(uuid!("...")),
    ///     start_time: chrono::Utc::now() - chrono::Duration::hours(1),
    ///     end_time: chrono::Utc::now(),
    ///     ..Default::default()
    /// };
    /// let points = service.query_points(query).await?;
    /// ```
    pub async fn query_points(&self, query: HistoryQuery) -> ApiResult<Vec<HistoryPointVO>> {
        tracing::debug!(
            device_id = ?query.device_id,
            tag_id = ?query.tag_id,
            start = %query.start_time,
            end = %query.end_time,
            "Querying history points"
        );

        let flux_query = self.build_points_query(&query)?;
        
        tracing::debug!(flux = %flux_query, "Executing Flux query");

        let query_result = self.client
            .query(Some(&flux_query))
            .await
            .context("Failed to execute InfluxDB query")?;

        let mut points = Vec::new();
        
        for table in query_result {
            for record in table.records {
                match InfluxDataPoint::from_data_point(&record) {
                    Ok(influx_point) => {
                        let point = HistoryPointVO {
                            device_id: influx_point.device_id.parse::<Uuid>()
                                .map_err(|_| ApiError::internal_error("Invalid device_id in InfluxDB"))?,
                            tag_id: influx_point.tag_id.parse::<Uuid>()
                                .map_err(|_| ApiError::internal_error("Invalid tag_id in InfluxDB"))?,
                            timestamp: influx_point.timestamp,
                            value: influx_point.value,
                            unit: influx_point.unit,
                        };
                        points.push(point);
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to parse InfluxDB data point");
                    }
                }
            }
        }

        // 按时间排序
        points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // 分页
        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(1000) as usize;
        
        let paginated = points
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        tracing::debug!(count = paginated.len(), "Query completed");
        Ok(paginated)
    }

    /// 查询聚合统计数据
    /// 
    /// # Parameters
    /// * `query` – 查询参数，包含聚合窗口大小
    /// 
    /// # Returns
    /// * `Ok(Vec<HistoryStatsVO>)` – 聚合统计结果
    /// * `Err(ApiError)` – 查询失败
    /// 
    /// # Side Effects
    /// * 执行InfluxDB聚合查询，可能影响性能
    /// 
    /// # Example
    /// ```rust
    /// let query = HistoryQuery {
    ///     aggregation_window: Some("5m".to_string()),
    ///     ..Default::default()
    /// };
    /// let stats = service.query_stats(query).await?;
    /// ```
    pub async fn query_stats(&self, query: HistoryQuery) -> ApiResult<Vec<HistoryStatsVO>> {
        tracing::debug!(
            device_id = ?query.device_id,
            tag_id = ?query.tag_id,
            window = ?query.aggregation_window,
            "Querying aggregated stats"
        );

        let flux_query = self.build_stats_query(&query)?;
        
        tracing::debug!(flux = %flux_query, "Executing Flux aggregation query");

        let query_result = self.client
            .query(Some(&flux_query))
            .await
            .context("Failed to execute InfluxDB aggregation query")?;

        let mut stats = Vec::new();
        
        for table in query_result {
            for record in table.records {
                match InfluxStatsPoint::from_data_point(&record) {
                    Ok(influx_stats) => {
                        let stat = HistoryStatsVO {
                            device_id: influx_stats.device_id.parse::<Uuid>()
                                .map_err(|_| ApiError::internal_error("Invalid device_id in InfluxDB"))?,
                            tag_id: influx_stats.tag_id.parse::<Uuid>()
                                .map_err(|_| ApiError::internal_error("Invalid tag_id in InfluxDB"))?,
                            timestamp: influx_stats.timestamp,
                            min_value: influx_stats.min_value,
                            max_value: influx_stats.max_value,
                            avg_value: influx_stats.avg_value,
                            count: influx_stats.count.unwrap_or(0),
                        };
                        stats.push(stat);
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to parse InfluxDB stats point");
                    }
                }
            }
        }

        // 按时间排序
        stats.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        tracing::debug!(count = stats.len(), "Stats query completed");
        Ok(stats)
    }

    /// 导出历史数据为CSV格式
    /// 
    /// # Parameters
    /// * `request` – 导出请求参数
    /// 
    /// # Returns
    /// * `Ok(String)` – CSV格式的数据
    /// * `Err(ApiError)` – 导出失败
    /// 
    /// # Errors
    /// 详见 [`ApiError`]。
    /// 
    /// # Example
    /// ```rust
    /// let request = HistoryExportRequest {
    ///     query: HistoryQuery { .. },
    ///     include_headers: true,
    ///     timestamp_format: "ISO8601".to_string(),
    /// };
    /// let csv = service.export_csv(request).await?;
    /// ```
    pub async fn export_csv(&self, request: HistoryExportRequest) -> ApiResult<String> {
        tracing::info!(
            device_id = ?request.query.device_id,
            tag_id = ?request.query.tag_id,
            "Exporting history data to CSV"
        );

        let points = self.query_points(request.query).await?;
        
        let mut csv_lines = Vec::new();
        
        // 添加CSV头部
        if request.include_headers {
            csv_lines.push("device_id,tag_id,timestamp,value,unit".to_string());
        }
        
        // 转换数据为CSV行
        for point in points {
            let timestamp_str = match request.timestamp_format.as_str() {
                "ISO8601" => point.timestamp.to_rfc3339(),
                "UNIX" => point.timestamp.timestamp().to_string(),
                "UNIX_MS" => point.timestamp.timestamp_millis().to_string(),
                _ => point.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            };
            
            let unit_str = point.unit.unwrap_or_default();
            
            let line = format!(
                "{},{},{},{},{}",
                point.device_id,
                point.tag_id,
                timestamp_str,
                point.value,
                unit_str
            );
            csv_lines.push(line);
        }
        
        let csv_content = csv_lines.join("\n");
        
        tracing::info!(lines = csv_lines.len(), "CSV export completed");
        Ok(csv_content)
    }

    /// 检查InfluxDB连接健康状态
    pub async fn health_check(&self) -> ApiResult<HashMap<String, String>> {
        let mut health = HashMap::new();
        
        match self.client.health().await {
            Ok(_) => {
                health.insert("influxdb".to_string(), "healthy".to_string());
                health.insert("org".to_string(), self.org.clone());
                health.insert("bucket".to_string(), self.bucket.clone());
            }
            Err(e) => {
                health.insert("influxdb".to_string(), "unhealthy".to_string());
                health.insert("error".to_string(), e.to_string());
            }
        }
        
        Ok(health)
    }

    /// 构建数据点查询的Flux语句
    fn build_points_query(&self, query: &HistoryQuery) -> ApiResult<String> {
        let start_time = query.start_time.to_rfc3339();
        let end_time = query.end_time.to_rfc3339();
        
        let mut flux = format!(
            r#"from(bucket: "{}")
  |> range(start: {}, stop: {})
  |> filter(fn: (r) => r._measurement == "telemetry")"#,
            self.bucket, start_time, end_time
        );

        // 设备ID过滤
        if let Some(device_id) = &query.device_id {
            flux.push_str(&format!(
                r#"
  |> filter(fn: (r) => r.device_id == "{}")"#,
                device_id
            ));
        }

        // 点位ID过滤
        if let Some(tag_id) = &query.tag_id {
            flux.push_str(&format!(
                r#"
  |> filter(fn: (r) => r.tag_id == "{}")"#,
                tag_id
            ));
        }

        // 排序
        flux.push_str(r#"
  |> sort(columns: ["_time"])"#);

        Ok(flux)
    }

    /// 构建统计聚合查询的Flux语句
    fn build_stats_query(&self, query: &HistoryQuery) -> ApiResult<String> {
        let start_time = query.start_time.to_rfc3339();
        let end_time = query.end_time.to_rfc3339();
        let window = query.aggregation_window.as_ref().unwrap_or(&"5m".to_string());
        
        let mut flux = format!(
            r#"from(bucket: "{}")
  |> range(start: {}, stop: {})
  |> filter(fn: (r) => r._measurement == "telemetry")"#,
            self.bucket, start_time, end_time
        );

        // 设备ID过滤
        if let Some(device_id) = &query.device_id {
            flux.push_str(&format!(
                r#"
  |> filter(fn: (r) => r.device_id == "{}")"#,
                device_id
            ));
        }

        // 点位ID过滤
        if let Some(tag_id) = &query.tag_id {
            flux.push_str(&format!(
                r#"
  |> filter(fn: (r) => r.tag_id == "{}")"#,
                tag_id
            ));
        }

        // 聚合统计 - 简化版本
        flux.push_str(&format!(
            r#"
  |> aggregateWindow(every: {}, fn: mean, createEmpty: false)
  |> group(columns: ["device_id", "tag_id", "_time"])
  |> yield(name: "mean")
  
from(bucket: "{}")
  |> range(start: {}, stop: {})
  |> filter(fn: (r) => r._measurement == "telemetry")
  |> aggregateWindow(every: {}, fn: min, createEmpty: false)
  |> group(columns: ["device_id", "tag_id", "_time"])
  |> yield(name: "min")
  
from(bucket: "{}")
  |> range(start: {}, stop: {})
  |> filter(fn: (r) => r._measurement == "telemetry")
  |> aggregateWindow(every: {}, fn: max, createEmpty: false)
  |> group(columns: ["device_id", "tag_id", "_time"])
  |> yield(name: "max")
  
from(bucket: "{}")
  |> range(start: {}, stop: {})
  |> filter(fn: (r) => r._measurement == "telemetry")
  |> aggregateWindow(every: {}, fn: count, createEmpty: false)
  |> group(columns: ["device_id", "tag_id", "_time"])
  |> yield(name: "count")"#,
            window, self.bucket, start_time, end_time, window,
            self.bucket, start_time, end_time, window,
            self.bucket, start_time, end_time, window
        ));

        Ok(flux)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_build_points_query() {
        let client = Client::new("http://localhost:8086", "test", "token");
        let service = HistoryService::new(client, "test".to_string(), "test".to_string());
        
        let query = HistoryQuery {
            device_id: Some(Uuid::new_v4()),
            tag_id: Some(Uuid::new_v4()),
            start_time: Utc::now() - Duration::hours(1),
            end_time: Utc::now(),
            ..Default::default()
        };
        
        let flux = service.build_points_query(&query).unwrap();
        assert!(flux.contains("from(bucket: \"test\")"));
        assert!(flux.contains("_measurement == \"telemetry\""));
        assert!(flux.contains("device_id =="));
        assert!(flux.contains("tag_id =="));
    }

    #[test]
    fn test_build_stats_query() {
        let client = Client::new("http://localhost:8086", "test", "token");
        let service = HistoryService::new(client, "test".to_string(), "test".to_string());
        
        let query = HistoryQuery {
            aggregation_window: Some("10m".to_string()),
            start_time: Utc::now() - Duration::hours(1),
            end_time: Utc::now(),
            ..Default::default()
        };
        
        let flux = service.build_stats_query(&query).unwrap();
        assert!(flux.contains("aggregateWindow(every: 10m"));
        assert!(flux.contains("mean"));
        assert!(flux.contains("min"));
        assert!(flux.contains("max"));
    }
}