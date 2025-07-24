/*!
# Request Handlers

HTTP request handlers for REST API endpoints.
*/

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use frame_bus::envelope::{CmdFrame, Value};
use frame_bus::command::CommandProcessor;

/// Command execution request
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub tag: String,
    pub value: serde_json::Value,
    pub source: Option<String>,
    pub priority: Option<i32>,
    pub timeout_ms: Option<u32>,
}

/// Command execution response
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub cmd_id: u64,
    pub success: bool,
    pub message: String,
    pub actual_value: Option<serde_json::Value>,
    pub execution_time_ms: Option<u64>,
}

/// Health check status
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub services: HashMap<String, ServiceStatus>,
}

/// Service status
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub healthy: bool,
    pub message: String,
    pub last_check: u64,
}

/// Command handler for processing command requests
pub struct CommandHandler {
    command_processor: Arc<CommandProcessor>,
    default_timeout: Duration,
}

impl CommandHandler {
    /// Create new command handler
    pub fn new(command_processor: Arc<CommandProcessor>) -> Self {
        Self {
            command_processor,
            default_timeout: Duration::from_secs(30),
        }
    }

    /// Execute command request
    pub async fn execute(
        &self,
        request: serde_json::Value,
        user_id: &str,
    ) -> Result<CommandResponse> {
        // Parse request
        let cmd_request: CommandRequest = serde_json::from_value(request)
            .context("Invalid command request format")?;

        info!("Executing command for tag '{}' by user '{}'", cmd_request.tag, user_id);

        // Convert JSON value to frame Value
        let frame_value = self.json_to_frame_value(cmd_request.value)?;

        // Create command frame
        let mut cmd_frame = CmdFrame::new(
            &cmd_request.tag,
            frame_value,
            &cmd_request.source.as_deref().unwrap_or(user_id).to_string(),
        );

        // Set optional parameters
        if let Some(priority) = cmd_request.priority {
            cmd_frame = cmd_frame.with_priority(priority);
        }

        if let Some(timeout_ms) = cmd_request.timeout_ms {
            cmd_frame = cmd_frame.with_timeout_ms(timeout_ms);
        }

        let cmd_id = cmd_frame.cmd_id;
        let cmd_timestamp = cmd_frame.timestamp;

        // Execute command with timeout
        let timeout_duration = if let Some(timeout_ms) = cmd_request.timeout_ms {
            Duration::from_millis(timeout_ms as u64)
        } else {
            self.default_timeout
        };

        match self.command_processor
            .execute_command_with_timeout(cmd_frame, timeout_duration)
            .await
        {
            Ok(ack) => {
                info!("Command {} executed successfully", cmd_id);
                
                Ok(CommandResponse {
                    cmd_id,
                    success: ack.success,
                    message: if ack.success {
                        "Command executed successfully".to_string()
                    } else {
                        ack.error_msg
                    },
                    actual_value: ack.actual_value.map(|v| self.frame_value_to_json(v)),
                    execution_time_ms: Some(
                        (ack.timestamp - cmd_timestamp) / 1_000_000 // Convert ns to ms
                    ),
                })
            }
            Err(e) => {
                warn!("Command {} execution failed: {}", cmd_id, e);
                
                Ok(CommandResponse {
                    cmd_id,
                    success: false,
                    message: format!("Command execution failed: {}", e),
                    actual_value: None,
                    execution_time_ms: None,
                })
            }
        }
    }

    /// Convert JSON value to frame Value
    fn json_to_frame_value(&self, json_value: serde_json::Value) -> Result<Value> {
        match json_value {
            serde_json::Value::Bool(b) => Ok(Value::bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::int(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::float(f))
                } else {
                    Err(anyhow::anyhow!("Invalid number format"))
                }
            }
            serde_json::Value::String(s) => Ok(Value::string(s)),
            serde_json::Value::Array(arr) => {
                // Convert array to bytes if it contains only numbers
                if arr.iter().all(|v| v.is_number()) {
                    let bytes: Result<Vec<u8>, _> = arr.iter()
                        .map(|v| v.as_u64().map(|n| n as u8))
                        .collect::<Option<Vec<_>>>()
                        .ok_or_else(|| anyhow::anyhow!("Invalid byte array"));
                    
                    Ok(Value::bytes(bytes?))
                } else {
                    Err(anyhow::anyhow!("Array values are not supported except for byte arrays"))
                }
            }
            serde_json::Value::Null => Err(anyhow::anyhow!("Null values are not supported")),
            serde_json::Value::Object(_) => Err(anyhow::anyhow!("Object values are not supported")),
        }
    }

    /// Convert frame Value to JSON value
    fn frame_value_to_json(&self, frame_value: Value) -> serde_json::Value {
        match frame_value.value {
            Some(frame_bus::envelope::value::Value::BoolV(b)) => serde_json::Value::Bool(b),
            Some(frame_bus::envelope::value::Value::IntV(i)) => serde_json::Value::Number(i.into()),
            Some(frame_bus::envelope::value::Value::FloatV(f)) => {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            Some(frame_bus::envelope::value::Value::StrV(s)) => serde_json::Value::String(s),
            Some(frame_bus::envelope::value::Value::BinV(b)) => {
                serde_json::Value::Array(
                    b.into_iter().map(|byte| serde_json::Value::Number(byte.into())).collect()
                )
            }
            None => serde_json::Value::Null,
        }
    }
}

/// Health handler for system health checks
pub struct HealthHandler {
    start_time: std::time::Instant,
    command_processor: Arc<CommandProcessor>,
}

impl HealthHandler {
    /// Create new health handler
    pub fn new(command_processor: Arc<CommandProcessor>) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            command_processor,
        }
    }

    /// Check system health
    pub async fn check(&self) -> Result<HealthStatus> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uptime = self.start_time.elapsed().as_secs();

        let mut services = HashMap::new();

        // Check command processor
        services.insert("command_processor".to_string(), ServiceStatus {
            healthy: true, // TODO: Add actual health check
            message: "Command processor is running".to_string(),
            last_check: now,
        });

        // Check command queue
        let queue_stats = self.command_processor.queue_stats().await;
        services.insert("command_queue".to_string(), ServiceStatus {
            healthy: queue_stats.total < 1000, // Consider unhealthy if queue is too large
            message: format!("Queue size: {} commands", queue_stats.total),
            last_check: now,
        });

        // Check command processing stats
        let cmd_stats = self.command_processor.stats();
        services.insert("command_stats".to_string(), ServiceStatus {
            healthy: cmd_stats.failed == 0 || (cmd_stats.succeeded > cmd_stats.failed * 10),
            message: format!("Success: {}, Failed: {}, Timeouts: {}", 
                           cmd_stats.succeeded, cmd_stats.failed, cmd_stats.timeouts),
            last_check: now,
        });

        let overall_healthy = services.values().all(|s| s.healthy);

        Ok(HealthStatus {
            status: if overall_healthy { "healthy" } else { "unhealthy" }.to_string(),
            timestamp: now,
            uptime_seconds: uptime,
            services,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_to_frame_value() {
        let (tx, _rx) = tokio::sync::broadcast::channel(1024);
        let command_processor = Arc::new(CommandProcessor::new(tx));
        let handler = CommandHandler::new(command_processor);

        // Test bool
        let bool_val = serde_json::Value::Bool(true);
        let frame_val = handler.json_to_frame_value(bool_val).unwrap();
        assert_eq!(frame_val.to_bool(), Some(true));

        // Test int
        let int_val = serde_json::Value::Number(42.into());
        let frame_val = handler.json_to_frame_value(int_val).unwrap();
        assert_eq!(frame_val.to_i64(), Some(42));

        // Test float
        let float_val = serde_json::Number::from_f64(3.14).unwrap();
        let float_json = serde_json::Value::Number(float_val);
        let frame_val = handler.json_to_frame_value(float_json).unwrap();
        assert_eq!(frame_val.to_f64(), Some(3.14));

        // Test string
        let string_val = serde_json::Value::String("test".to_string());
        let frame_val = handler.json_to_frame_value(string_val).unwrap();
        assert_eq!(frame_val.to_string(), Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_health_check() {
        let (tx, _rx) = tokio::sync::broadcast::channel(1024);
        let command_processor = Arc::new(CommandProcessor::new(tx));
        let handler = HealthHandler::new(command_processor);

        let status = handler.check().await.unwrap();
        assert_eq!(status.status, "healthy");
        assert!(status.services.contains_key("command_processor"));
    }
}