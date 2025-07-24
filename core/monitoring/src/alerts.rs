/*!
# Alert Management System

Comprehensive alerting and notification system for the Edge Gateway.
*/

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn, error, debug};

/// Alert handler trait
#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync {
    /// Handle an alert
    async fn handle_alert(&self, alert: &Alert) -> Result<()>;
    
    /// Check if this handler should handle the alert
    fn should_handle(&self, alert: &Alert) -> bool;
    
    /// Get handler name
    fn name(&self) -> &str;
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertSeverity {
    /// Get severity as string
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARNING",
            AlertSeverity::Error => "ERROR",
            AlertSeverity::Critical => "CRITICAL",
        }
    }

    /// Get severity color for display
    pub fn color(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "#3498db",      // Blue
            AlertSeverity::Warning => "#f39c12",   // Orange
            AlertSeverity::Error => "#e74c3c",     // Red
            AlertSeverity::Critical => "#9b59b6",  // Purple
        }
    }
}

/// Alert structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
    pub tags: Vec<String>,
}

impl Alert {
    /// Create new alert
    pub fn new(severity: AlertSeverity, title: String, message: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: format!("alert_{}_{}", severity.as_str().to_lowercase(), timestamp),
            severity,
            title,
            message,
            timestamp,
            tags: Vec::new(),
        }
    }

    /// Add tags to alert
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Check if alert matches filter
    pub fn matches_filter(&self, filter: &AlertFilter) -> bool {
        // Check severity
        if let Some(min_severity) = &filter.min_severity {
            if self.severity < *min_severity {
                return false;
            }
        }

        // Check tags
        if !filter.tags.is_empty() {
            let has_matching_tag = filter.tags.iter()
                .any(|filter_tag| self.tags.contains(filter_tag));
            if !has_matching_tag {
                return false;
            }
        }

        // Check time range
        if let Some(since) = filter.since {
            if self.timestamp < since {
                return false;
            }
        }

        if let Some(until) = filter.until {
            if self.timestamp > until {
                return false;
            }
        }

        true
    }
}

/// Alert filter for querying alerts
#[derive(Debug, Clone, Default)]
pub struct AlertFilter {
    pub min_severity: Option<AlertSeverity>,
    pub tags: Vec<String>,
    pub since: Option<u64>,
    pub until: Option<u64>,
    pub limit: Option<usize>,
}

impl AlertFilter {
    /// Create new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by minimum severity
    pub fn with_min_severity(mut self, severity: AlertSeverity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    /// Filter by tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Filter by time range
    pub fn with_time_range(mut self, since: u64, until: u64) -> Self {
        self.since = Some(since);
        self.until = Some(until);
        self
    }

    /// Limit number of results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    pub total_alerts: usize,
    pub alerts_by_severity: HashMap<String, usize>,
    pub recent_alerts_24h: usize,
    pub most_common_tags: Vec<(String, usize)>,
}

/// Alert manager for handling all alerting functionality
pub struct AlertManager {
    alerts: Arc<RwLock<VecDeque<Alert>>>,
    alert_tx: mpsc::UnboundedSender<Alert>,
    alert_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<Alert>>>>,
    max_alerts: usize,
    notification_handlers: Vec<Arc<dyn NotificationHandler>>,
}

impl AlertManager {
    /// Create new alert manager
    pub fn new(max_alerts: usize) -> Self {
        let (alert_tx, alert_rx) = mpsc::unbounded_channel();

        Self {
            alerts: Arc::new(RwLock::new(VecDeque::new())),
            alert_tx,
            alert_rx: Arc::new(RwLock::new(Some(alert_rx))),
            max_alerts,
            notification_handlers: Vec::new(),
        }
    }

    /// Add notification handler
    pub fn add_notification_handler(&mut self, handler: Arc<dyn NotificationHandler>) {
        info!("Added notification handler: {}", handler.name());
        self.notification_handlers.push(handler);
    }

    /// Start alert processing
    pub async fn start_processing(&self) {
        let alerts = self.alerts.clone();
        let max_alerts = self.max_alerts;
        let handlers = self.notification_handlers.clone();

        // Take the receiver
        let mut alert_rx = {
            let mut rx_guard = self.alert_rx.write().await;
            rx_guard.take().expect("Alert receiver already taken")
        };

        tokio::spawn(async move {
            while let Some(alert) = alert_rx.recv().await {
                debug!("Processing alert: {} - {}", alert.severity.as_str(), alert.title);

                // Store alert
                {
                    let mut alerts_guard = alerts.write().await;
                    alerts_guard.push_back(alert.clone());

                    // Keep only max_alerts
                    while alerts_guard.len() > max_alerts {
                        alerts_guard.pop_front();
                    }
                }

                // Send notifications
                for handler in &handlers {
                    if let Err(e) = handler.handle_alert(&alert).await {
                        error!("Notification handler '{}' failed: {}", handler.name(), e);
                    }
                }
            }
        });

        info!("Alert processing started with {} notification handlers", self.notification_handlers.len());
    }

    /// Trigger new alert
    pub async fn trigger_alert(&self, alert: Alert) {
        if let Err(e) = self.alert_tx.send(alert.clone()) {
            error!("Failed to send alert: {}", e);
        } else {
            debug!("Alert triggered: {} - {}", alert.severity.as_str(), alert.title);
        }
    }

    /// Get alerts with filter
    pub async fn get_alerts(&self, filter: AlertFilter) -> Vec<Alert> {
        let alerts_guard = self.alerts.read().await;
        let mut matching_alerts: Vec<Alert> = alerts_guard
            .iter()
            .filter(|alert| alert.matches_filter(&filter))
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        matching_alerts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = filter.limit {
            matching_alerts.truncate(limit);
        }

        matching_alerts
    }

    /// Get recent alerts
    pub async fn get_recent_alerts(&self, duration: Duration) -> Vec<Alert> {
        let since = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - duration.as_secs();

        let filter = AlertFilter::new()
            .with_time_range(since, u64::MAX)
            .with_limit(100);

        self.get_alerts(filter).await
    }

    /// Get alert statistics
    pub async fn get_alert_stats(&self) -> AlertStats {
        let alerts_guard = self.alerts.read().await;
        let total_alerts = alerts_guard.len();

        // Count by severity
        let mut alerts_by_severity = HashMap::new();
        for alert in alerts_guard.iter() {
            let severity_str = alert.severity.as_str().to_string();
            *alerts_by_severity.entry(severity_str).or_insert(0) += 1;
        }

        // Recent alerts (24h)
        let recent_cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (24 * 3600);

        let recent_alerts_24h = alerts_guard
            .iter()
            .filter(|alert| alert.timestamp >= recent_cutoff)
            .count();

        // Most common tags
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for alert in alerts_guard.iter() {
            for tag in &alert.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let mut most_common_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        most_common_tags.sort_by(|a, b| b.1.cmp(&a.1));
        most_common_tags.truncate(10); // Top 10 tags

        AlertStats {
            total_alerts,
            alerts_by_severity,
            recent_alerts_24h,
            most_common_tags,
        }
    }

    /// Clear all alerts
    pub async fn clear_alerts(&self) {
        let mut alerts_guard = self.alerts.write().await;
        alerts_guard.clear();
        info!("All alerts cleared");
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new(1000) // Default max 1000 alerts
    }
}

/// Notification handler trait
#[async_trait::async_trait]
pub trait NotificationHandler: Send + Sync {
    /// Handle alert notification
    async fn handle_alert(&self, alert: &Alert) -> Result<()>;

    /// Get handler name
    fn name(&self) -> &str;

    /// Check if handler should process this alert
    fn should_handle(&self, alert: &Alert) -> bool {
        // Default: handle all alerts
        true
    }
}

/// Console notification handler
pub struct ConsoleNotificationHandler;

#[async_trait::async_trait]
impl NotificationHandler for ConsoleNotificationHandler {
    async fn handle_alert(&self, alert: &Alert) -> Result<()> {
        let timestamp = chrono::DateTime::from_timestamp(alert.timestamp as i64, 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S");

        match alert.severity {
            AlertSeverity::Info => info!("[ALERT] {} - {}: {}", timestamp, alert.title, alert.message),
            AlertSeverity::Warning => warn!("[ALERT] {} - {}: {}", timestamp, alert.title, alert.message),
            AlertSeverity::Error | AlertSeverity::Critical => {
                error!("[ALERT] {} - {}: {}", timestamp, alert.title, alert.message)
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

/// Log file notification handler
pub struct LogFileNotificationHandler {
    log_file: String,
}

impl LogFileNotificationHandler {
    pub fn new(log_file: String) -> Self {
        Self { log_file }
    }
}

#[async_trait::async_trait]
impl NotificationHandler for LogFileNotificationHandler {
    async fn handle_alert(&self, alert: &Alert) -> Result<()> {
        let timestamp = chrono::DateTime::from_timestamp(alert.timestamp as i64, 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S");

        let log_entry = format!(
            "{} [{}] {} - {} (tags: {})\n",
            timestamp,
            alert.severity.as_str(),
            alert.title,
            alert.message,
            alert.tags.join(", ")
        );

        tokio::fs::write(&self.log_file, log_entry).await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "log_file"
    }

    fn should_handle(&self, alert: &Alert) -> bool {
        // Only handle warnings and errors
        matches!(alert.severity, AlertSeverity::Warning | AlertSeverity::Error | AlertSeverity::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            AlertSeverity::Warning,
            "Test Alert".to_string(),
            "This is a test alert".to_string(),
        );

        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.title, "Test Alert");
        assert_eq!(alert.message, "This is a test alert");
        assert!(alert.timestamp > 0);
    }

    #[test]
    fn test_alert_filter() {
        let alert = Alert::new(
            AlertSeverity::Error,
            "Error Alert".to_string(),
            "An error occurred".to_string(),
        ).with_tags(vec!["system".to_string(), "critical".to_string()]);

        let filter = AlertFilter::new()
            .with_min_severity(AlertSeverity::Warning)
            .with_tags(vec!["system".to_string()]);

        assert!(alert.matches_filter(&filter));

        let filter2 = AlertFilter::new()
            .with_min_severity(AlertSeverity::Critical);

        assert!(!alert.matches_filter(&filter2));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(AlertSeverity::Critical > AlertSeverity::Error);
        assert!(AlertSeverity::Error > AlertSeverity::Warning);
        assert!(AlertSeverity::Warning > AlertSeverity::Info);
    }

    #[tokio::test]
    async fn test_alert_manager() {
        let mut manager = AlertManager::new(10);
        manager.start_processing().await;

        let alert = Alert::new(
            AlertSeverity::Info,
            "Test Alert".to_string(),
            "Test message".to_string(),
        );

        manager.trigger_alert(alert).await;

        // Wait a bit for processing
        tokio::time::sleep(Duration::from_millis(10)).await;

        let alerts = manager.get_alerts(AlertFilter::new()).await;
        assert_eq!(alerts.len(), 1);
    }

    #[tokio::test]
    async fn test_console_notification_handler() {
        let handler = ConsoleNotificationHandler;
        let alert = Alert::new(
            AlertSeverity::Info,
            "Test Alert".to_string(),
            "Test message".to_string(),
        );

        let result = handler.handle_alert(&alert).await;
        assert!(result.is_ok());
    }
}