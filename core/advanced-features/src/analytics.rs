/*!
# Real-time Analytics Engine

Advanced analytics capabilities for industrial IoT data streams.
*/

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::{ComponentStatus, HealthLevel};

/// Analytics engine for real-time data processing
pub struct AnalyticsEngine {
    processors: Arc<RwLock<HashMap<String, Box<dyn AnalyticsProcessor>>>>,
    metrics: Arc<RwLock<AnalyticsMetrics>>,
    config: AnalyticsConfig,
    command_tx: mpsc::UnboundedSender<AnalyticsCommand>,
    status: Arc<RwLock<ComponentStatus>>,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enable_real_time: bool,
    pub batch_size: usize,
    pub processing_interval_ms: u64,
    pub retention_hours: u32,
    pub max_memory_mb: usize,
    pub compression_enabled: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enable_real_time: true,
            batch_size: 1000,
            processing_interval_ms: 100,
            retention_hours: 24,
            max_memory_mb: 512,
            compression_enabled: true,
        }
    }
}

/// Analytics command
pub enum AnalyticsCommand {
    ProcessData(AnalyticsData),
    AddProcessor(String, Box<dyn AnalyticsProcessor>),
    RemoveProcessor(String),
    GetMetrics(tokio::sync::oneshot::Sender<AnalyticsMetrics>),
    Stop,
}

/// Analytics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub id: String,
    pub timestamp: SystemTime,
    pub source: String,
    pub data_type: String,
    pub value: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

/// Analytics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult {
    pub id: String,
    pub processor_name: String,
    pub timestamp: SystemTime,
    pub result_type: AnalyticsResultType,
    pub value: serde_json::Value,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

/// Analytics result type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsResultType {
    Aggregation,
    Pattern,
    Anomaly,
    Prediction,
    Classification,
    Trend,
}

/// Analytics metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub total_data_points: u64,
    pub processed_per_second: f64,
    pub error_rate: f64,
    pub memory_usage_mb: f64,
    pub active_processors: usize,
    pub average_processing_time_ms: f64,
    pub queue_size: usize,
}

impl Default for AnalyticsMetrics {
    fn default() -> Self {
        Self {
            total_data_points: 0,
            processed_per_second: 0.0,
            error_rate: 0.0,
            memory_usage_mb: 0.0,
            active_processors: 0,
            average_processing_time_ms: 0.0,
            queue_size: 0,
        }
    }
}

/// Analytics processor trait
#[async_trait::async_trait]
pub trait AnalyticsProcessor: Send + Sync {
    /// Process analytics data
    async fn process(&self, data: &AnalyticsData) -> Result<Vec<AnalyticsResult>>;
    
    /// Get processor name
    fn name(&self) -> &str;
    
    /// Get processor configuration
    fn config(&self) -> HashMap<String, serde_json::Value>;
    
    /// Initialize processor
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Shutdown processor
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl AnalyticsEngine {
    /// Create new analytics engine
    pub async fn new() -> Result<Self> {
        Self::new_with_config(AnalyticsConfig::default()).await
    }

    /// Create new analytics engine with configuration
    pub async fn new_with_config(config: AnalyticsConfig) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        
        let processors = Arc::new(RwLock::new(HashMap::new()));
        let metrics = Arc::new(RwLock::new(AnalyticsMetrics::default()));
        let status = Arc::new(RwLock::new(ComponentStatus::default()));

        let engine = Self {
            processors: Arc::clone(&processors),
            metrics: Arc::clone(&metrics),
            config,
            command_tx,
            status: Arc::clone(&status),
        };

        // Start command processor
        tokio::spawn(Self::command_processor(
            command_rx,
            Arc::clone(&processors),
            Arc::clone(&metrics),
            Arc::clone(&status),
        ));

        info!("Analytics engine created");
        Ok(engine)
    }

    /// Start analytics engine
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = true;
        status.health = HealthLevel::Good;
        info!("Analytics engine started");
        Ok(())
    }

    /// Stop analytics engine
    pub async fn stop(&self) -> Result<()> {
        let _ = self.command_tx.send(AnalyticsCommand::Stop);
        
        let mut status = self.status.write().await;
        status.running = false;
        info!("Analytics engine stopped");
        Ok(())
    }

    /// Process data point
    pub async fn process_data(&self, data: AnalyticsData) -> Result<()> {
        self.command_tx
            .send(AnalyticsCommand::ProcessData(data))
            .context("Failed to send process data command")?;
        Ok(())
    }

    /// Add analytics processor
    pub async fn add_processor(&self, name: String, processor: Box<dyn AnalyticsProcessor>) -> Result<()> {
        self.command_tx
            .send(AnalyticsCommand::AddProcessor(name, processor))
            .context("Failed to send add processor command")?;
        Ok(())
    }

    /// Remove analytics processor
    pub async fn remove_processor(&self, name: &str) -> Result<()> {
        self.command_tx
            .send(AnalyticsCommand::RemoveProcessor(name.to_string()))
            .context("Failed to send remove processor command")?;
        Ok(())
    }

    /// Get analytics metrics
    pub async fn get_metrics(&self) -> Result<AnalyticsMetrics> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        self.command_tx
            .send(AnalyticsCommand::GetMetrics(tx))
            .context("Failed to send get metrics command")?;
            
        rx.await.context("Failed to receive metrics")
    }

    /// Get component status
    pub async fn get_status(&self) -> Result<ComponentStatus> {
        let status = self.status.read().await;
        Ok(status.clone())
    }

    /// Command processor loop
    async fn command_processor(
        mut command_rx: mpsc::UnboundedReceiver<AnalyticsCommand>,
        processors: Arc<RwLock<HashMap<String, Box<dyn AnalyticsProcessor>>>>,
        metrics: Arc<RwLock<AnalyticsMetrics>>,
        status: Arc<RwLock<ComponentStatus>>,
    ) {
        while let Some(command) = command_rx.recv().await {
            match command {
                AnalyticsCommand::ProcessData(data) => {
                    Self::process_data_internal(&processors, &metrics, data).await;
                }
                
                AnalyticsCommand::AddProcessor(name, mut processor) => {
                    if let Err(e) = processor.initialize().await {
                        error!("Failed to initialize processor {}: {}", name, e);
                        continue;
                    }
                    
                    let mut procs = processors.write().await;
                    procs.insert(name.clone(), processor);
                    info!("Added analytics processor: {}", name);
                }
                
                AnalyticsCommand::RemoveProcessor(name) => {
                    let mut procs = processors.write().await;
                    if let Some(mut processor) = procs.remove(&name) {
                        let _ = processor.shutdown().await;
                        info!("Removed analytics processor: {}", name);
                    }
                }
                
                AnalyticsCommand::GetMetrics(tx) => {
                    let metrics = metrics.read().await;
                    let _ = tx.send(metrics.clone());
                }
                
                AnalyticsCommand::Stop => {
                    // Shutdown all processors
                    let mut procs = processors.write().await;
                    for (name, processor) in procs.iter_mut() {
                        if let Err(e) = processor.shutdown().await {
                            error!("Error shutting down processor {}: {}", name, e);
                        }
                    }
                    procs.clear();
                    break;
                }
            }
        }
    }

    /// Process data with all processors
    async fn process_data_internal(
        processors: &Arc<RwLock<HashMap<String, Box<dyn AnalyticsProcessor>>>>,
        metrics: &Arc<RwLock<AnalyticsMetrics>>,
        data: AnalyticsData,
    ) {
        let start_time = std::time::Instant::now();
        let processors = processors.read().await;
        
        for (name, processor) in processors.iter() {
            match processor.process(&data).await {
                Ok(results) => {
                    debug!("Processor {} produced {} results", name, results.len());
                    // Here you would typically forward results to subscribers
                }
                Err(e) => {
                    error!("Processor {} failed: {}", name, e);
                    // Update error metrics
                }
            }
        }

        // Update metrics
        let processing_time = start_time.elapsed();
        let mut metrics = metrics.write().await;
        metrics.total_data_points += 1;
        metrics.average_processing_time_ms = 
            (metrics.average_processing_time_ms + processing_time.as_millis() as f64) / 2.0;
        metrics.active_processors = processors.len();
    }
}

/// Moving average processor
pub struct MovingAverageProcessor {
    name: String,
    window_size: usize,
    values: RwLock<Vec<f64>>,
}

impl MovingAverageProcessor {
    pub fn new(name: String, window_size: usize) -> Self {
        Self {
            name,
            window_size,
            values: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl AnalyticsProcessor for MovingAverageProcessor {
    async fn process(&self, data: &AnalyticsData) -> Result<Vec<AnalyticsResult>> {
        // Extract numeric value
        let value = match &data.value {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            _ => return Ok(vec![]),
        };

        let mut values = self.values.write().await;
        values.push(value);

        // Keep only window_size values
        if values.len() > self.window_size {
            values.remove(0);
        }

        // Calculate moving average
        let average = values.iter().sum::<f64>() / values.len() as f64;

        let result = AnalyticsResult {
            id: Uuid::new_v4().to_string(),
            processor_name: self.name.clone(),
            timestamp: SystemTime::now(),
            result_type: AnalyticsResultType::Aggregation,
            value: serde_json::Value::Number(
                serde_json::Number::from_f64(average).unwrap_or(serde_json::Number::from(0))
            ),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        Ok(vec![result])
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self) -> HashMap<String, serde_json::Value> {
        let mut config = HashMap::new();
        config.insert("window_size".to_string(), 
            serde_json::Value::Number(serde_json::Number::from(self.window_size)));
        config
    }
}

/// Anomaly detection processor
pub struct AnomalyDetectionProcessor {
    name: String,
    threshold: f64,
    baseline: RwLock<f64>,
    sample_count: RwLock<usize>,
}

impl AnomalyDetectionProcessor {
    pub fn new(name: String, threshold: f64) -> Self {
        Self {
            name,
            threshold,
            baseline: RwLock::new(0.0),
            sample_count: RwLock::new(0),
        }
    }
}

#[async_trait::async_trait]
impl AnalyticsProcessor for AnomalyDetectionProcessor {
    async fn process(&self, data: &AnalyticsData) -> Result<Vec<AnalyticsResult>> {
        let value = match &data.value {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            _ => return Ok(vec![]),
        };

        let mut baseline = self.baseline.write().await;
        let mut count = self.sample_count.write().await;

        // Update baseline (simple exponential moving average)
        if *count == 0 {
            *baseline = value;
        } else {
            *baseline = 0.9 * *baseline + 0.1 * value;
        }
        *count += 1;

        // Check for anomaly
        let deviation = (value - *baseline).abs();
        let is_anomaly = deviation > self.threshold;

        if is_anomaly {
            let mut metadata = HashMap::new();
            metadata.insert("baseline".to_string(), baseline.to_string());
            metadata.insert("deviation".to_string(), deviation.to_string());
            metadata.insert("threshold".to_string(), self.threshold.to_string());

            let result = AnalyticsResult {
                id: Uuid::new_v4().to_string(),
                processor_name: self.name.clone(),
                timestamp: SystemTime::now(),
                result_type: AnalyticsResultType::Anomaly,
                value: serde_json::Value::Number(
                    serde_json::Number::from_f64(value).unwrap_or(serde_json::Number::from(0))
                ),
                confidence: (deviation / self.threshold).min(1.0),
                metadata,
            };

            Ok(vec![result])
        } else {
            Ok(vec![])
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self) -> HashMap<String, serde_json::Value> {
        let mut config = HashMap::new();
        config.insert("threshold".to_string(), 
            serde_json::Value::Number(serde_json::Number::from_f64(self.threshold).unwrap_or(serde_json::Number::from(0))));
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analytics_engine_creation() {
        let engine = AnalyticsEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_moving_average_processor() {
        let processor = MovingAverageProcessor::new("test".to_string(), 3);
        
        let data = AnalyticsData {
            id: "test".to_string(),
            timestamp: SystemTime::now(),
            source: "test".to_string(),
            data_type: "numeric".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(10)),
            metadata: HashMap::new(),
        };

        let results = processor.process(&data).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].processor_name, "test");
    }

    #[tokio::test]
    async fn test_anomaly_detection_processor() {
        let processor = AnomalyDetectionProcessor::new("anomaly".to_string(), 5.0);
        
        // Normal value
        let normal_data = AnalyticsData {
            id: "normal".to_string(),
            timestamp: SystemTime::now(),
            source: "test".to_string(),
            data_type: "numeric".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(10)),
            metadata: HashMap::new(),
        };

        let results = processor.process(&normal_data).await.unwrap();
        // First value establishes baseline, no anomaly detected
        assert_eq!(results.len(), 0);

        // Anomalous value
        let anomaly_data = AnalyticsData {
            id: "anomaly".to_string(),
            timestamp: SystemTime::now(),
            source: "test".to_string(),
            data_type: "numeric".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(100)),
            metadata: HashMap::new(),
        };

        let results = processor.process(&anomaly_data).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].result_type, AnalyticsResultType::Anomaly));
    }
}