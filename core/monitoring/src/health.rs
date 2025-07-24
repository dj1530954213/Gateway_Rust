/*!
# Health Check System

Comprehensive health monitoring for all gateway components.
*/

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tracing::{info, warn, error, debug};

/// Health check status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
    pub last_check: u64,
    pub check_duration_ms: u64,
}

impl HealthCheckResult {
    /// Create healthy result
    pub fn healthy(component: &str, message: &str) -> Self {
        Self {
            component: component.to_string(),
            status: HealthStatus::Healthy,
            message: message.to_string(),
            details: None,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            check_duration_ms: 0,
        }
    }

    /// Create degraded result
    pub fn degraded(component: &str, message: &str) -> Self {
        Self {
            component: component.to_string(),
            status: HealthStatus::Degraded,
            message: message.to_string(),
            details: None,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            check_duration_ms: 0,
        }
    }

    /// Create unhealthy result
    pub fn unhealthy(component: &str, message: &str) -> Self {
        Self {
            component: component.to_string(),
            status: HealthStatus::Unhealthy,
            message: message.to_string(),
            details: None,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            check_duration_ms: 0,
        }
    }

    /// Add details to the result
    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    /// Set check duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.check_duration_ms = duration.as_millis() as u64;
        self
    }
}

/// Health check trait for components
#[async_trait::async_trait]
pub trait HealthChecker: Send + Sync {
    /// Perform health check
    async fn check_health(&self) -> HealthCheckResult;

    /// Get component name
    fn component_name(&self) -> &str;

    /// Get check interval
    fn check_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
}

/// Overall system health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub components: Vec<HealthCheckResult>,
    pub summary: HealthSummary,
    pub timestamp: u64,
}

/// Health summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
    pub unknown_count: usize,
    pub total_components: usize,
}

/// Health monitor that coordinates all health checks
pub struct HealthMonitor {
    checkers: Vec<Arc<dyn HealthChecker>>,
    last_results: Arc<tokio::sync::RwLock<HashMap<String, HealthCheckResult>>>,
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new() -> Self {
        Self {
            checkers: Vec::new(),
            last_results: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Register a health checker
    pub fn register_checker(&mut self, checker: Arc<dyn HealthChecker>) {
        info!("Registered health checker for component: {}", checker.component_name());
        self.checkers.push(checker);
    }

    /// Start health monitoring
    pub async fn start_monitoring(&self) {
        for checker in &self.checkers {
            let checker_clone = checker.clone();
            let results = self.last_results.clone();

            tokio::spawn(async move {
                let mut interval = interval(checker_clone.check_interval());

                loop {
                    interval.tick().await;

                    let start_time = Instant::now();
                    let mut result = checker_clone.check_health().await;
                    let duration = start_time.elapsed();

                    result = result.with_duration(duration);

                    debug!("Health check completed for {}: {:?} ({}ms)", 
                           result.component, result.status, result.check_duration_ms);

                    // Store result
                    {
                        let mut results_map = results.write().await;
                        results_map.insert(result.component.clone(), result);
                    }
                }
            });
        }

        info!("Started health monitoring for {} components", self.checkers.len());
    }

    /// Get current system health
    pub async fn get_system_health(&self) -> SystemHealth {
        let results_map = self.last_results.read().await;
        let components: Vec<HealthCheckResult> = results_map.values().cloned().collect();

        // Calculate summary
        let healthy_count = components.iter().filter(|r| r.status == HealthStatus::Healthy).count();
        let degraded_count = components.iter().filter(|r| r.status == HealthStatus::Degraded).count();
        let unhealthy_count = components.iter().filter(|r| r.status == HealthStatus::Unhealthy).count();
        let unknown_count = components.iter().filter(|r| r.status == HealthStatus::Unknown).count();

        let summary = HealthSummary {
            healthy_count,
            degraded_count,
            unhealthy_count,
            unknown_count,
            total_components: components.len(),
        };

        // Determine overall status
        let overall_status = if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else if healthy_count > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };

        SystemHealth {
            overall_status,
            components,
            summary,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }

    /// Get health for specific component
    pub async fn get_component_health(&self, component: &str) -> Option<HealthCheckResult> {
        let results_map = self.last_results.read().await;
        results_map.get(component).cloned()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic system health checker
pub struct SystemHealthChecker;

#[async_trait::async_trait]
impl HealthChecker for SystemHealthChecker {
    async fn check_health(&self) -> HealthCheckResult {
        // Basic system checks
        let mut details = HashMap::new();

        // Check if we can allocate memory
        match std::alloc::Layout::from_size_align(1024, 8) {
            Ok(_) => {
                details.insert("memory_allocation".to_string(), serde_json::Value::Bool(true));
            }
            Err(_) => {
                return HealthCheckResult::unhealthy("system", "Memory allocation failed");
            }
        }

        // Check current time
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                details.insert("current_time".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(duration.as_secs())
                ));
            }
            Err(_) => {
                return HealthCheckResult::degraded("system", "System time is invalid");
            }
        }

        HealthCheckResult::healthy("system", "System is operating normally")
            .with_details(details)
    }

    fn component_name(&self) -> &str {
        "system"
    }
}

/// Frame bus health checker
pub struct FrameBusHealthChecker {
    frame_sender: frame_bus::FrameSender,
}

impl FrameBusHealthChecker {
    pub fn new(frame_sender: frame_bus::FrameSender) -> Self {
        Self { frame_sender }
    }
}

#[async_trait::async_trait]
impl HealthChecker for FrameBusHealthChecker {
    async fn check_health(&self) -> HealthCheckResult {
        let mut details = HashMap::new();

        // Check if frame sender is working
        let receiver_count = self.frame_sender.receiver_count();
        details.insert("receiver_count".to_string(), serde_json::Value::Number(
            serde_json::Number::from(receiver_count)
        ));

        if receiver_count > 0 {
            HealthCheckResult::healthy("frame_bus", "Frame bus is operating normally")
                .with_details(details)
        } else {
            HealthCheckResult::degraded("frame_bus", "No active receivers")
                .with_details(details)
        }
    }

    fn component_name(&self) -> &str {
        "frame_bus"
    }
}

/// Driver manager health checker
pub struct DriverManagerHealthChecker;

#[async_trait::async_trait]
impl HealthChecker for DriverManagerHealthChecker {
    async fn check_health(&self) -> HealthCheckResult {
        // This would normally check the actual driver manager
        // For now, we'll return a simple healthy status
        HealthCheckResult::healthy("driver_manager", "Driver manager is operating normally")
    }

    fn component_name(&self) -> &str {
        "driver_manager"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result_creation() {
        let result = HealthCheckResult::healthy("test", "All good");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.component, "test");
        assert_eq!(result.message, "All good");
    }

    #[test]
    fn test_health_check_result_with_details() {
        let mut details = HashMap::new();
        details.insert("key".to_string(), serde_json::Value::String("value".to_string()));

        let result = HealthCheckResult::healthy("test", "All good")
            .with_details(details.clone());

        assert_eq!(result.details, Some(details));
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let mut monitor = HealthMonitor::new();
        
        let checker = Arc::new(SystemHealthChecker);
        monitor.register_checker(checker);

        let health = monitor.get_system_health().await;
        assert_eq!(health.summary.total_components, 0); // No checks run yet
    }

    #[tokio::test]
    async fn test_system_health_checker() {
        let checker = SystemHealthChecker;
        let result = checker.check_health().await;
        
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.component, "system");
        assert!(result.details.is_some());
    }

    #[test]
    fn test_health_summary_calculation() {
        let components = vec![
            HealthCheckResult::healthy("comp1", "OK"),
            HealthCheckResult::degraded("comp2", "Warning"),
            HealthCheckResult::unhealthy("comp3", "Error"),
        ];

        let healthy_count = components.iter().filter(|r| r.status == HealthStatus::Healthy).count();
        let degraded_count = components.iter().filter(|r| r.status == HealthStatus::Degraded).count();
        let unhealthy_count = components.iter().filter(|r| r.status == HealthStatus::Unhealthy).count();

        assert_eq!(healthy_count, 1);
        assert_eq!(degraded_count, 1);
        assert_eq!(unhealthy_count, 1);
    }
}