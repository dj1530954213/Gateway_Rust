/*!
# Monitoring System

System monitoring, health checks, and metrics collection.
*/

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use prometheus::{Counter, Gauge, Registry, Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, debug};

pub mod health;
pub mod alerts;
pub mod diagnostics;

pub use health::*;
pub use alerts::*;
pub use diagnostics::*;

/// Metrics collector for system and business metrics
pub struct MetricsCollector {
    registry: Arc<Registry>,
    gateway_uptime: Gauge,
    data_frames_total: Counter,
    commands_total: Counter,
    status: Arc<RwLock<ComponentStatus>>,
}

/// Health monitor for system components
pub struct HealthMonitor {
    checkers: Arc<RwLock<Vec<Box<dyn HealthChecker>>>>,
    status: Arc<RwLock<ComponentStatus>>,
}

/// Alert manager for system alerts
pub struct AlertManager {
    handlers: Arc<RwLock<Vec<Box<dyn AlertHandler>>>>,
    status: Arc<RwLock<ComponentStatus>>,
}

/// Component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub running: bool,
    pub health: HealthLevel,
    pub uptime_seconds: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub metrics: std::collections::HashMap<String, serde_json::Value>,
}

/// Health level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Arc::new(Registry::new());
        
        let gateway_uptime = Gauge::with_opts(
            prometheus::Opts::new("gateway_uptime_seconds", "Gateway uptime in seconds")
        )?;
        registry.register(Box::new(gateway_uptime.clone()))?;

        let data_frames_total = Counter::with_opts(
            prometheus::Opts::new("gateway_data_frames_total", "Total data frames processed")
        )?;
        registry.register(Box::new(data_frames_total.clone()))?;

        let commands_total = Counter::with_opts(
            prometheus::Opts::new("gateway_commands_total", "Total commands processed")
        )?;
        registry.register(Box::new(commands_total.clone()))?;

        Ok(Self {
            registry,
            gateway_uptime,
            data_frames_total,
            commands_total,
            status: Arc::new(RwLock::new(ComponentStatus::default())),
        })
    }

    /// Start metrics collection
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = true;
        status.health = HealthLevel::Good;
        
        info!("Metrics collector started");
        Ok(())
    }

    /// Stop metrics collection
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = false;
        
        info!("Metrics collector stopped");
        Ok(())
    }

    /// Get metrics as Prometheus format
    pub fn get_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut output = Vec::new();
        encoder.encode(&metric_families, &mut output)?;
        Ok(String::from_utf8(output)?)
    }

    /// Increment data frame counter
    pub fn increment_data_frames(&self) {
        self.data_frames_total.inc();
    }

    /// Increment command counter
    pub fn increment_commands(&self) {
        self.commands_total.inc();
    }

    /// Update uptime
    pub fn update_uptime(&self, seconds: f64) {
        self.gateway_uptime.set(seconds);
    }

    /// Get component status
    pub async fn get_status(&self) -> ComponentStatus {
        let status = self.status.read().await;
        status.clone()
    }
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new() -> Result<Self> {
        Ok(Self {
            checkers: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(ComponentStatus::default())),
        })
    }

    /// Start health monitoring
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = true;
        status.health = HealthLevel::Good;
        
        info!("Health monitor started");
        Ok(())
    }

    /// Stop health monitoring
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = false;
        
        info!("Health monitor stopped");
        Ok(())
    }

    /// Add health checker
    pub async fn add_checker(&self, checker: Box<dyn HealthChecker>) {
        let mut checkers = self.checkers.write().await;
        checkers.push(checker);
    }

    /// Run all health checks
    pub async fn check_health(&self) -> Vec<HealthCheckResult> {
        let checkers = self.checkers.read().await;
        let mut results = Vec::new();
        
        for checker in checkers.iter() {
            let result = checker.check_health().await;
            results.push(result);
        }
        
        results
    }

    /// Get component status
    pub async fn get_status(&self) -> ComponentStatus {
        let status = self.status.read().await;
        status.clone()
    }
}

impl AlertManager {
    /// Create new alert manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(ComponentStatus::default())),
        })
    }

    /// Start alert management
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = true;
        status.health = HealthLevel::Good;
        
        info!("Alert manager started");
        Ok(())
    }

    /// Stop alert management
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = false;
        
        info!("Alert manager stopped");
        Ok(())
    }

    /// Add alert handler
    pub async fn add_handler(&self, handler: Box<dyn AlertHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }

    /// Send alert
    pub async fn send_alert(&self, alert: Alert) -> Result<()> {
        let handlers = self.handlers.read().await;
        
        for handler in handlers.iter() {
            if handler.should_handle(&alert) {
                handler.handle_alert(&alert).await?;
            }
        }
        
        Ok(())
    }

    /// Get component status
    pub async fn get_status(&self) -> ComponentStatus {
        let status = self.status.read().await;
        status.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new().unwrap();
        collector.start().await.unwrap();
        
        collector.increment_data_frames();
        collector.increment_commands();
        collector.update_uptime(123.0);
        
        let metrics = collector.get_metrics().unwrap();
        assert!(metrics.contains("gateway_data_frames_total"));
        assert!(metrics.contains("gateway_commands_total"));
        assert!(metrics.contains("gateway_uptime_seconds"));
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new().unwrap();
        monitor.start().await.unwrap();
        
        let results = monitor.check_health().await;
        assert_eq!(results.len(), 0); // No checkers added
    }

    #[tokio::test]
    async fn test_alert_manager() {
        let manager = AlertManager::new().unwrap();
        manager.start().await.unwrap();
        
        // Test with no handlers
        let alert = Alert {
            id: "test".to_string(),
            severity: AlertSeverity::Info,
            title: "Test Alert".to_string(),
            message: "Test message".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags: vec!["test".to_string()],
        };
        
        assert!(manager.send_alert(alert).await.is_ok());
    }
}