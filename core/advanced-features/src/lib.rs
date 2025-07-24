/*!
# Advanced Features Module

High-performance edge computing capabilities, machine learning, and advanced analytics.
Steps 51-70 implementation for production-ready Industrial IoT Edge Gateway.
*/

use std::sync::Arc;
use anyhow::Result;
use tracing::info;

pub mod analytics;
pub mod edge_ml;
pub mod time_series;
pub mod data_pipeline;
pub mod edge_compute;
pub mod optimization;
pub mod predictive;
pub mod automation;

pub use analytics::*;
pub use edge_ml::*;
pub use time_series::*;
pub use data_pipeline::*;
pub use edge_compute::*;
pub use optimization::*;
pub use predictive::*;
pub use automation::*;

/// Advanced features manager
pub struct AdvancedFeaturesManager {
    analytics_engine: Arc<AnalyticsEngine>,
    ml_engine: Arc<EdgeMLEngine>,
    time_series_processor: Arc<TimeSeriesProcessor>,
    data_pipeline: Arc<DataPipeline>,
    edge_compute: Arc<EdgeComputeRuntime>,
    optimization_engine: Arc<OptimizationEngine>,
    predictive_engine: Arc<PredictiveEngine>,
    automation_controller: Arc<AutomationController>,
}

impl AdvancedFeaturesManager {
    /// Create new advanced features manager
    pub async fn new() -> Result<Self> {
        info!("Initializing advanced features manager");

        let analytics_engine = Arc::new(AnalyticsEngine::new().await?);
        let ml_engine = Arc::new(EdgeMLEngine::new().await?);
        let time_series_processor = Arc::new(TimeSeriesProcessor::new().await?);
        let data_pipeline = Arc::new(DataPipeline::new().await?);
        let edge_compute = Arc::new(EdgeComputeRuntime::new().await?);
        let optimization_engine = Arc::new(OptimizationEngine::new().await?);
        let predictive_engine = Arc::new(PredictiveEngine::new().await?);
        let automation_controller = Arc::new(AutomationController::new().await?);

        Ok(Self {
            analytics_engine,
            ml_engine,
            time_series_processor,
            data_pipeline,
            edge_compute,
            optimization_engine,
            predictive_engine,
            automation_controller,
        })
    }

    /// Start all advanced features
    pub async fn start(&self) -> Result<()> {
        info!("Starting advanced features");

        // Start analytics engine
        self.analytics_engine.start().await?;
        
        // Start ML engine
        self.ml_engine.start().await?;
        
        // Start time series processor
        self.time_series_processor.start().await?;
        
        // Start data pipeline
        self.data_pipeline.start().await?;
        
        // Start edge compute runtime
        self.edge_compute.start().await?;
        
        // Start optimization engine
        self.optimization_engine.start().await?;
        
        // Start predictive engine
        self.predictive_engine.start().await?;
        
        // Start automation controller
        self.automation_controller.start().await?;

        info!("All advanced features started successfully");
        Ok(())
    }

    /// Stop all advanced features
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping advanced features");

        // Stop in reverse order
        let _ = self.automation_controller.stop().await;
        let _ = self.predictive_engine.stop().await;
        let _ = self.optimization_engine.stop().await;
        let _ = self.edge_compute.stop().await;
        let _ = self.data_pipeline.stop().await;
        let _ = self.time_series_processor.stop().await;
        let _ = self.ml_engine.stop().await;
        let _ = self.analytics_engine.stop().await;

        info!("All advanced features stopped");
        Ok(())
    }

    /// Get analytics engine
    pub fn analytics(&self) -> Arc<AnalyticsEngine> {
        Arc::clone(&self.analytics_engine)
    }

    /// Get ML engine
    pub fn ml_engine(&self) -> Arc<EdgeMLEngine> {
        Arc::clone(&self.ml_engine)
    }

    /// Get time series processor
    pub fn time_series(&self) -> Arc<TimeSeriesProcessor> {
        Arc::clone(&self.time_series_processor)
    }

    /// Get data pipeline
    pub fn data_pipeline(&self) -> Arc<DataPipeline> {
        Arc::clone(&self.data_pipeline)
    }

    /// Get edge compute runtime
    pub fn edge_compute(&self) -> Arc<EdgeComputeRuntime> {
        Arc::clone(&self.edge_compute)
    }

    /// Get optimization engine
    pub fn optimization(&self) -> Arc<OptimizationEngine> {
        Arc::clone(&self.optimization_engine)
    }

    /// Get predictive engine
    pub fn predictive(&self) -> Arc<PredictiveEngine> {
        Arc::clone(&self.predictive_engine)
    }

    /// Get automation controller
    pub fn automation(&self) -> Arc<AutomationController> {
        Arc::clone(&self.automation_controller)
    }

    /// Get system health metrics
    pub async fn get_health_metrics(&self) -> Result<AdvancedFeaturesHealth> {
        Ok(AdvancedFeaturesHealth {
            analytics_status: self.analytics_engine.get_status().await?,
            ml_status: self.ml_engine.get_status().await?,
            time_series_status: self.time_series_processor.get_status().await?,
            data_pipeline_status: self.data_pipeline.get_status().await?,
            edge_compute_status: self.edge_compute.get_status().await?,
            optimization_status: self.optimization_engine.get_status().await?,
            predictive_status: self.predictive_engine.get_status().await?,
            automation_status: self.automation_controller.get_status().await?,
        })
    }
}

/// Advanced features health status
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AdvancedFeaturesHealth {
    pub analytics_status: ComponentStatus,
    pub ml_status: ComponentStatus,
    pub time_series_status: ComponentStatus,
    pub data_pipeline_status: ComponentStatus,
    pub edge_compute_status: ComponentStatus,
    pub optimization_status: ComponentStatus,
    pub predictive_status: ComponentStatus,
    pub automation_status: ComponentStatus,
}

/// Component status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentStatus {
    pub running: bool,
    pub health: HealthLevel,
    pub uptime_seconds: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub metrics: std::collections::HashMap<String, serde_json::Value>,
}

/// Health level enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HealthLevel {
    Excellent,
    Good,
    Warning,
    Critical,
    Failed,
}

impl Default for ComponentStatus {
    fn default() -> Self {
        Self {
            running: false,
            health: HealthLevel::Good,
            uptime_seconds: 0,
            error_count: 0,
            last_error: None,
            metrics: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_advanced_features_manager_creation() {
        let manager = AdvancedFeaturesManager::new().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_component_status_default() {
        let status = ComponentStatus::default();
        assert!(!status.running);
        assert_eq!(status.error_count, 0);
        assert!(status.last_error.is_none());
    }
}