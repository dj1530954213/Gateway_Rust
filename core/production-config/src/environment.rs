/*!
# Environment Configuration

Environment-specific configuration management and overrides.
*/

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use tracing::{info, warn, debug};

use crate::ProductionConfig;

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub is_production: bool,
    pub debug_enabled: bool,
    pub log_level: String,
    pub overrides: HashMap<String, EnvironmentOverride>,
}

/// Environment configuration override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentOverride {
    pub key: String,
    pub value: serde_json::Value,
    pub description: String,
    pub required: bool,
}

impl EnvironmentConfig {
    /// Create environment config from production config
    pub fn from_production_config(config: &ProductionConfig) -> Self {
        let name = config.general.environment.clone();
        let is_production = name.to_lowercase() == "production" || name.to_lowercase() == "prod";
        
        Self {
            name: name.clone(),
            is_production,
            debug_enabled: config.general.enable_debug,
            log_level: config.logging.level.clone(),
            overrides: Self::create_default_overrides(&name),
        }
    }

    /// Create default environment overrides
    fn create_default_overrides(environment: &str) -> HashMap<String, EnvironmentOverride> {
        let mut overrides = HashMap::new();

        match environment.to_lowercase().as_str() {
            "development" | "dev" => {
                overrides.insert("debug_mode".to_string(), EnvironmentOverride {
                    key: "GATEWAY__GENERAL__ENABLE_DEBUG".to_string(),
                    value: serde_json::Value::Bool(true),
                    description: "Enable debug mode in development".to_string(),
                    required: false,
                });

                overrides.insert("log_level".to_string(), EnvironmentOverride {
                    key: "GATEWAY__LOGGING__LEVEL".to_string(),
                    value: serde_json::Value::String("debug".to_string()),
                    description: "Set log level to debug".to_string(),
                    required: false,
                });

                overrides.insert("metrics_interval".to_string(), EnvironmentOverride {
                    key: "GATEWAY__MONITORING__METRICS_INTERVAL_SECONDS".to_string(),
                    value: serde_json::Value::Number(serde_json::Number::from(5)),
                    description: "Faster metrics collection in dev".to_string(),
                    required: false,
                });
            }

            "staging" | "stage" => {
                overrides.insert("debug_mode".to_string(), EnvironmentOverride {
                    key: "GATEWAY__GENERAL__ENABLE_DEBUG".to_string(),
                    value: serde_json::Value::Bool(false),
                    description: "Disable debug mode in staging".to_string(),
                    required: false,
                });

                overrides.insert("log_level".to_string(), EnvironmentOverride {
                    key: "GATEWAY__LOGGING__LEVEL".to_string(),
                    value: serde_json::Value::String("info".to_string()),
                    description: "Set log level to info".to_string(),
                    required: false,
                });
            }

            "production" | "prod" => {
                overrides.insert("jwt_secret".to_string(), EnvironmentOverride {
                    key: "GATEWAY__SECURITY__JWT_SECRET".to_string(),
                    value: serde_json::Value::String("REQUIRED".to_string()),
                    description: "JWT secret must be provided via environment".to_string(),
                    required: true,
                });

                overrides.insert("debug_mode".to_string(), EnvironmentOverride {
                    key: "GATEWAY__GENERAL__ENABLE_DEBUG".to_string(),
                    value: serde_json::Value::Bool(false),
                    description: "Debug mode must be disabled in production".to_string(),
                    required: true,
                });

                overrides.insert("log_level".to_string(), EnvironmentOverride {
                    key: "GATEWAY__LOGGING__LEVEL".to_string(),
                    value: serde_json::Value::String("warn".to_string()),
                    description: "Set log level to warn in production".to_string(),
                    required: false,
                });

                overrides.insert("tls_enabled".to_string(), EnvironmentOverride {
                    key: "GATEWAY__NETWORK__ENABLE_TLS".to_string(),
                    value: serde_json::Value::Bool(true),
                    description: "TLS should be enabled in production".to_string(),
                    required: true,
                });
            }

            _ => {
                // Default overrides for unknown environments
            }
        }

        overrides
    }

    /// Apply environment overrides to current process
    pub fn apply_environment_overrides(&self) -> Result<()> {
        for (_name, override_config) in &self.overrides {
            if let Ok(existing_value) = env::var(&override_config.key) {
                debug!("Environment variable {} already set: {}", override_config.key, existing_value);
                continue;
            }

            if override_config.required {
                if override_config.value == serde_json::Value::String("REQUIRED".to_string()) {
                    return Err(anyhow::anyhow!(
                        "Required environment variable {} is not set: {}",
                        override_config.key,
                        override_config.description
                    ));
                }
            }

            // Set environment variable if not already set
            let value_str = match &override_config.value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => continue,
            };

            env::set_var(&override_config.key, &value_str);
            info!("Set environment variable {}: {} ({})", 
                override_config.key, value_str, override_config.description);
        }

        Ok(())
    }

    /// Validate environment configuration
    pub fn validate(&self) -> Result<()> {
        // Check required overrides for production
        if self.is_production {
            for (_name, override_config) in &self.overrides {
                if override_config.required {
                    match env::var(&override_config.key) {
                        Ok(value) => {
                            if value.is_empty() {
                                return Err(anyhow::anyhow!(
                                    "Required environment variable {} is empty: {}",
                                    override_config.key,
                                    override_config.description
                                ));
                            }
                        }
                        Err(_) => {
                            return Err(anyhow::anyhow!(
                                "Required environment variable {} is not set: {}",
                                override_config.key,
                                override_config.description
                            ));
                        }
                    }
                }
            }

            // Additional production-specific validations
            if self.debug_enabled {
                warn!("Debug mode is enabled in production environment");
            }

            if self.log_level.to_lowercase() == "debug" || self.log_level.to_lowercase() == "trace" {
                warn!("Verbose logging is enabled in production environment");
            }
        }

        Ok(())
    }

    /// Get environment variable with fallback
    pub fn get_env_var(&self, key: &str, fallback: Option<&str>) -> Option<String> {
        env::var(key).ok().or_else(|| fallback.map(|s| s.to_string()))
    }

    /// Check if running in container
    pub fn is_containerized(&self) -> bool {
        // Check for common container indicators
        env::var("KUBERNETES_SERVICE_HOST").is_ok() ||
        env::var("DOCKER_CONTAINER").is_ok() ||
        PathBuf::from("/.dockerenv").exists() ||
        PathBuf::from("/proc/1/cgroup").exists() && 
            std::fs::read_to_string("/proc/1/cgroup")
                .map(|content| content.contains("docker") || content.contains("kubepods"))
                .unwrap_or(false)
    }

    /// Get container runtime information
    pub fn get_container_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();

        info.insert("containerized".to_string(), self.is_containerized().to_string());

        if let Ok(hostname) = env::var("HOSTNAME") {
            info.insert("hostname".to_string(), hostname);
        }

        if let Ok(pod_name) = env::var("POD_NAME") {
            info.insert("pod_name".to_string(), pod_name);
        }

        if let Ok(namespace) = env::var("POD_NAMESPACE") {
            info.insert("namespace".to_string(), namespace);
        }

        if let Ok(node_name) = env::var("NODE_NAME") {
            info.insert("node_name".to_string(), node_name);
        }

        info
    }

    /// Get all environment variables with prefix
    pub fn get_env_vars_with_prefix(&self, prefix: &str) -> HashMap<String, String> {
        env::vars()
            .filter(|(key, _)| key.starts_with(prefix))
            .collect()
    }

    /// Export environment configuration to shell script
    pub fn export_to_shell_script(&self, file_path: &PathBuf) -> Result<()> {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# Generated environment configuration\n");
        script.push_str(&format!("# Environment: {}\n", self.name));
        script.push_str(&format!("# Generated at: {}\n\n", chrono::Utc::now().to_rfc3339()));

        for (_name, override_config) in &self.overrides {
            let value_str = match &override_config.value {
                serde_json::Value::String(s) => {
                    if s == "REQUIRED" {
                        format!("# REQUIRED: {}", override_config.description)
                    } else {
                        format!("export {}=\"{}\"", override_config.key, s)
                    }
                }
                serde_json::Value::Number(n) => {
                    format!("export {}=\"{}\"", override_config.key, n)
                }
                serde_json::Value::Bool(b) => {
                    format!("export {}=\"{}\"", override_config.key, b)
                }
                _ => continue,
            };

            script.push_str(&format!("# {}\n", override_config.description));
            script.push_str(&format!("{}\n\n", value_str));
        }

        std::fs::write(file_path, script)
            .with_context(|| format!("Failed to write environment script to {:?}", file_path))?;

        info!("Environment configuration exported to: {:?}", file_path);
        Ok(())
    }
}

/// Environment detection utilities
pub struct EnvironmentDetector;

impl EnvironmentDetector {
    /// Detect current environment
    pub fn detect_environment() -> String {
        // Check explicit environment variable
        if let Ok(env) = env::var("ENVIRONMENT") {
            return env.to_lowercase();
        }
        
        if let Ok(env) = env::var("ENV") {
            return env.to_lowercase();
        }

        if let Ok(env) = env::var("GATEWAY_ENVIRONMENT") {
            return env.to_lowercase();
        }

        // Check for development indicators
        if env::var("CARGO_PKG_NAME").is_ok() || 
           env::var("RUST_LOG").is_ok() ||
           PathBuf::from("Cargo.toml").exists() {
            return "development".to_string();
        }

        // Check for container/cloud indicators
        if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            return "production".to_string();
        }

        // Default to production for safety
        "production".to_string()
    }

    /// Get system information
    pub fn get_system_info() -> HashMap<String, String> {
        let mut info = HashMap::new();

        info.insert("os".to_string(), env::consts::OS.to_string());
        info.insert("arch".to_string(), env::consts::ARCH.to_string());

        if let Ok(hostname) = hostname::get() {
            info.insert("hostname".to_string(), hostname.to_string_lossy().to_string());
        }

        if let Ok(user) = env::var("USER").or_else(|_| env::var("USERNAME")) {
            info.insert("user".to_string(), user);
        }

        if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
            info.insert("home".to_string(), home);
        }

        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProductionConfig;

    #[test]
    fn test_environment_detection() {
        let env = EnvironmentDetector::detect_environment();
        assert!(!env.is_empty());
    }

    #[test]
    fn test_environment_config_creation() {
        let prod_config = ProductionConfig::default();
        let env_config = EnvironmentConfig::from_production_config(&prod_config);
        
        assert_eq!(env_config.name, "production");
        assert!(env_config.is_production);
        assert!(!env_config.overrides.is_empty());
    }

    #[test]
    fn test_development_overrides() {
        let mut prod_config = ProductionConfig::default();
        prod_config.general.environment = "development".to_string();
        
        let env_config = EnvironmentConfig::from_production_config(&prod_config);
        assert!(!env_config.is_production);
        assert!(env_config.overrides.contains_key("debug_mode"));
        assert!(env_config.overrides.contains_key("log_level"));
    }

    #[test]
    fn test_production_overrides() {
        let env_config = EnvironmentConfig::from_production_config(&ProductionConfig::default());
        assert!(env_config.is_production);
        assert!(env_config.overrides.contains_key("jwt_secret"));
        assert!(env_config.overrides.contains_key("tls_enabled"));
        
        // Check required overrides
        let jwt_override = &env_config.overrides["jwt_secret"];
        assert!(jwt_override.required);
    }

    #[test]
    fn test_container_detection() {
        let env_config = EnvironmentConfig::from_production_config(&ProductionConfig::default());
        let _is_containerized = env_config.is_containerized();
        // Test will pass regardless of actual environment
    }

    #[test]
    fn test_system_info() {
        let info = EnvironmentDetector::get_system_info();
        assert!(info.contains_key("os"));
        assert!(info.contains_key("arch"));
    }
}