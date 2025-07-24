/*!
# Time Series Processing

High-performance time series data processing and analysis.
*/

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::{ComponentStatus, HealthLevel};

/// Time series processor
pub struct TimeSeriesProcessor {
    series: Arc<RwLock<HashMap<String, TimeSeries>>>,
    config: TimeSeriesConfig,
    status: Arc<RwLock<ComponentStatus>>,
}

/// Time series configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesConfig {
    pub max_series: usize,
    pub max_points_per_series: usize,
    pub retention_duration: Duration,
    pub compression_enabled: bool,
    pub aggregation_interval: Duration,
}

impl Default for TimeSeriesConfig {
    fn default() -> Self {
        Self {
            max_series: 1000,
            max_points_per_series: 10000,
            retention_duration: Duration::from_secs(86400), // 24 hours
            compression_enabled: true,
            aggregation_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

/// Time series data structure
#[derive(Debug, Clone)]
pub struct TimeSeries {
    pub id: String,
    pub name: String,
    pub data_points: VecDeque<DataPoint>,
    pub metadata: HashMap<String, String>,
    pub stats: SeriesStats,
}

/// Data point in time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: SystemTime,
    pub value: f64,
    pub quality: DataQuality,
    pub tags: HashMap<String, String>,
}

/// Data quality indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataQuality {
    Good,
    Uncertain,
    Bad,
    Substituted,
}

/// Series statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesStats {
    pub count: usize,
    pub min_value: f64,
    pub max_value: f64,
    pub avg_value: f64,
    pub last_update: SystemTime,
    pub total_bytes: usize,
}

impl Default for SeriesStats {
    fn default() -> Self {
        Self {
            count: 0,
            min_value: f64::INFINITY,
            max_value: f64::NEG_INFINITY,
            avg_value: 0.0,
            last_update: SystemTime::UNIX_EPOCH,
            total_bytes: 0,
        }
    }
}

impl TimeSeriesProcessor {
    /// Create new time series processor
    pub async fn new() -> Result<Self> {
        Ok(Self {
            series: Arc::new(RwLock::new(HashMap::new())),
            config: TimeSeriesConfig::default(),
            status: Arc::new(RwLock::new(ComponentStatus::default())),
        })
    }

    /// Start processor
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = true;
        status.health = HealthLevel::Good;
        info!("Time series processor started");
        Ok(())
    }

    /// Stop processor
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = false;
        info!("Time series processor stopped");
        Ok(())
    }

    /// Add data point
    pub async fn add_point(&self, series_id: &str, point: DataPoint) -> Result<()> {
        let mut series_map = self.series.write().await;
        
        let series = series_map.entry(series_id.to_string())
            .or_insert_with(|| TimeSeries {
                id: series_id.to_string(),
                name: series_id.to_string(),
                data_points: VecDeque::new(),
                metadata: HashMap::new(),
                stats: SeriesStats::default(),
            });

        // Add point and update stats
        series.data_points.push_back(point.clone());
        self.update_stats(series, &point);

        // Enforce retention policy
        self.enforce_retention(series);

        debug!("Added point to series {}: {}", series_id, point.value);
        Ok(())
    }

    /// Get series data
    pub async fn get_series(&self, series_id: &str) -> Option<TimeSeries> {
        let series_map = self.series.read().await;
        series_map.get(series_id).cloned()
    }

    /// Query time range
    pub async fn query_range(
        &self,
        series_id: &str,
        start: SystemTime,
        end: SystemTime,
    ) -> Vec<DataPoint> {
        let series_map = self.series.read().await;
        
        if let Some(series) = series_map.get(series_id) {
            series.data_points.iter()
                .filter(|point| point.timestamp >= start && point.timestamp <= end)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get status
    pub async fn get_status(&self) -> Result<ComponentStatus> {
        let status = self.status.read().await;
        Ok(status.clone())
    }

    /// Update series statistics
    fn update_stats(&self, series: &mut TimeSeries, point: &DataPoint) {
        let stats = &mut series.stats;
        stats.count += 1;
        stats.min_value = stats.min_value.min(point.value);
        stats.max_value = stats.max_value.max(point.value);
        stats.avg_value = (stats.avg_value * (stats.count - 1) as f64 + point.value) / stats.count as f64;
        stats.last_update = point.timestamp;
        stats.total_bytes += std::mem::size_of::<DataPoint>();
    }

    /// Enforce retention policy
    fn enforce_retention(&self, series: &mut TimeSeries) {
        let cutoff = SystemTime::now() - self.config.retention_duration;
        
        while let Some(point) = series.data_points.front() {
            if point.timestamp < cutoff {
                series.data_points.pop_front();
                series.stats.count = series.stats.count.saturating_sub(1);
            } else {
                break;
            }
        }

        // Enforce max points limit
        while series.data_points.len() > self.config.max_points_per_series {
            series.data_points.pop_front();
            series.stats.count = series.stats.count.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_time_series_processor() {
        let processor = TimeSeriesProcessor::new().await.unwrap();
        processor.start().await.unwrap();

        let point = DataPoint {
            timestamp: SystemTime::now(),
            value: 42.0,
            quality: DataQuality::Good,
            tags: HashMap::new(),
        };

        processor.add_point("test_series", point).await.unwrap();
        
        let series = processor.get_series("test_series").await.unwrap();
        assert_eq!(series.data_points.len(), 1);
        assert_eq!(series.stats.count, 1);
    }
}