/*!
# Data Pipeline

High-throughput data processing pipeline with transformations and routing.
*/

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error};

use crate::{ComponentStatus, HealthLevel};

/// Data pipeline engine
pub struct DataPipeline {
    stages: Arc<RwLock<Vec<PipelineStage>>>,
    config: PipelineConfig,
    status: Arc<RwLock<ComponentStatus>>,
    input_tx: mpsc::UnboundedSender<PipelineData>,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub max_throughput: usize,
    pub buffer_size: usize,
    pub enable_metrics: bool,
    pub parallel_processing: bool,
    pub error_handling: ErrorHandling,
}

/// Error handling strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandling {
    Skip,
    Retry,
    DeadLetter,
    Stop,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_throughput: 10000,
            buffer_size: 1000,
            enable_metrics: true,
            parallel_processing: true,
            error_handling: ErrorHandling::Skip,
        }
    }
}

/// Pipeline stage
pub struct PipelineStage {
    pub name: String,
    pub processor: Box<dyn DataProcessor>,
    pub parallel: bool,
}

/// Pipeline data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineData {
    pub id: String,
    pub timestamp: std::time::SystemTime,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub stage_results: HashMap<String, serde_json::Value>,
}

/// Data processor trait
#[async_trait::async_trait]
pub trait DataProcessor: Send + Sync {
    async fn process(&self, data: &mut PipelineData) -> Result<()>;
    fn name(&self) -> &str;
}

impl DataPipeline {
    /// Create new data pipeline
    pub async fn new() -> Result<Self> {
        let (input_tx, input_rx) = mpsc::unbounded_channel();
        
        let pipeline = Self {
            stages: Arc::new(RwLock::new(Vec::new())),
            config: PipelineConfig::default(),
            status: Arc::new(RwLock::new(ComponentStatus::default())),
            input_tx,
        };

        // Start processing loop
        let stages = Arc::clone(&pipeline.stages);
        let status = Arc::clone(&pipeline.status);
        tokio::spawn(Self::processing_loop(input_rx, stages, status));

        Ok(pipeline)
    }

    /// Start pipeline
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = true;
        status.health = HealthLevel::Good;
        info!("Data pipeline started");
        Ok(())
    }

    /// Stop pipeline
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = false;
        info!("Data pipeline stopped");
        Ok(())
    }

    /// Add stage
    pub async fn add_stage(&self, stage: PipelineStage) -> Result<()> {
        let mut stages = self.stages.write().await;
        stages.push(stage);
        Ok(())
    }

    /// Process data
    pub async fn process(&self, data: PipelineData) -> Result<()> {
        self.input_tx.send(data).map_err(|_| anyhow::anyhow!("Pipeline closed"))?;
        Ok(())
    }

    /// Get status
    pub async fn get_status(&self) -> Result<ComponentStatus> {
        let status = self.status.read().await;
        Ok(status.clone())
    }

    /// Processing loop
    async fn processing_loop(
        mut input_rx: mpsc::UnboundedReceiver<PipelineData>,
        stages: Arc<RwLock<Vec<PipelineStage>>>,
        status: Arc<RwLock<ComponentStatus>>,
    ) {
        while let Some(mut data) = input_rx.recv().await {
            let stages = stages.read().await;
            
            for stage in stages.iter() {
                if let Err(e) = stage.processor.process(&mut data).await {
                    error!("Stage {} failed: {}", stage.name, e);
                }
            }
        }
    }
}

/// Filter processor
pub struct FilterProcessor {
    name: String,
    condition: String,
}

impl FilterProcessor {
    pub fn new(name: String, condition: String) -> Self {
        Self { name, condition }
    }
}

#[async_trait::async_trait]
impl DataProcessor for FilterProcessor {
    async fn process(&self, data: &mut PipelineData) -> Result<()> {
        // Simple filter logic (placeholder)
        if data.data.is_object() {
            data.stage_results.insert(self.name.clone(), serde_json::Value::Bool(true));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_pipeline() {
        let pipeline = DataPipeline::new().await.unwrap();
        pipeline.start().await.unwrap();

        let data = PipelineData {
            id: "test".to_string(),
            timestamp: std::time::SystemTime::now(),
            data: serde_json::json!({"value": 42}),
            metadata: HashMap::new(),
            stage_results: HashMap::new(),
        };

        assert!(pipeline.process(data).await.is_ok());
    }
}