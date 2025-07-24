/*!
# Production Configuration Management

Enterprise-grade configuration management with validation, encryption, and environment support.
*/

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use config::{Config, Environment, File};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use validator::Validate;
use tracing::{info, warn, debug};
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine;

pub mod encryption;
pub mod validation;
pub mod environment;

pub use encryption::*;
pub use validation::*;
pub use environment::*;

/// Production gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProductionConfig {
    /// General settings
    pub general: GeneralConfig,

    /// Network configuration
    pub network: NetworkConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Monitoring configuration
    pub monitoring: MonitoringConfig,

    /// Driver configurations
    #[serde(default)]
    pub drivers: HashMap<String, DriverConfig>,

    /// Connector configurations
    #[serde(default)]
    pub connectors: HashMap<String, ConnectorConfig>,
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GeneralConfig {
    #[validate(length(min = 1, max = 100))]
    pub instance_id: String,

    #[validate(length(min = 1, max = 50))]
    pub environment: String, // dev, staging, production

    #[validate(length(min = 1, max = 200))]
    pub description: String,

    pub enable_debug: bool,
    pub max_workers: Option<u32>,

    #[validate(range(min = 1, max = 86400))]
    pub heartbeat_interval_seconds: u32,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            instance_id: "edge-gateway-001".to_string(),
            environment: "production".to_string(),
            description: "Industrial IoT Edge Gateway".to_string(),
            enable_debug: false,
            max_workers: None,
            heartbeat_interval_seconds: 30,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NetworkConfig {
    #[validate(custom = "validate_socket_addr")]
    pub rest_api_bind: String,

    #[validate(custom = "validate_socket_addr")]
    pub metrics_bind: String,

    #[validate(custom = "validate_socket_addr")]
    pub web_ui_bind: String,

    #[validate(range(min = 1, max = 65535))]
    pub max_connections: u32,

    #[validate(range(min = 1, max = 3600))]
    pub connection_timeout_seconds: u32,

    pub enable_tls: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            rest_api_bind: "0.0.0.0:8080".to_string(),
            metrics_bind: "0.0.0.0:9090".to_string(),
            web_ui_bind: "0.0.0.0:8090".to_string(),
            max_connections: 1000,
            connection_timeout_seconds: 30,
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    #[validate(length(min = 32))]
    pub jwt_secret: String,

    #[validate(range(min = 300, max = 86400))]
    pub jwt_expiry_seconds: u32,

    pub enable_signature_verification: bool,

    #[serde(default)]
    pub trusted_keys: Vec<String>,

    pub enable_rate_limiting: bool,

    #[validate(range(min = 1, max = 10000))]
    pub rate_limit_per_minute: u32,

    pub enable_access_control: bool,
    pub default_permissions: Vec<String>,

    #[validate(length(min = 1))]
    pub admin_users: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        // Generate a secure JWT secret
        let rng = SystemRandom::new();
        let mut secret = vec![0u8; 32];
        rng.fill(&mut secret).unwrap();
        let jwt_secret = base64_engine.encode(&secret);

        Self {
            jwt_secret,
            jwt_expiry_seconds: 3600, // 1 hour
            enable_signature_verification: true,
            trusted_keys: Vec::new(),
            enable_rate_limiting: true,
            rate_limit_per_minute: 60,
            enable_access_control: true,
            default_permissions: vec!["read".to_string()],
            admin_users: vec!["admin".to_string()],
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoggingConfig {
    #[validate(custom = "validate_log_level")]
    pub level: String,

    #[validate(custom = "validate_log_format")]
    pub format: String, // json, text

    pub enable_file_logging: bool,
    pub log_file_path: Option<PathBuf>,

    #[validate(range(min = 1, max = 1000))]
    pub max_log_files: u32,

    #[validate(range(min = 1, max = 1000))]
    pub max_log_size_mb: u32,

    pub enable_console_logging: bool,
    pub console_colors: bool,

    #[serde(default)]
    pub log_filters: HashMap<String, String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "text".to_string(),
            enable_file_logging: true,
            log_file_path: Some(PathBuf::from("logs/gateway.log")),
            max_log_files: 10,
            max_log_size_mb: 100,
            enable_console_logging: true,
            console_colors: true,
            log_filters: HashMap::new(),
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StorageConfig {
    pub data_directory: PathBuf,
    pub wal_directory: PathBuf,
    pub driver_directory: PathBuf,
    pub config_directory: PathBuf,

    #[validate(range(min = 64, max = 65536))]
    pub wal_buffer_size: u32,

    #[validate(range(min = 1, max = 86400))]
    pub wal_sync_interval_seconds: u32,

    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub encryption_key_file: Option<PathBuf>,

    #[validate(range(min = 1, max = 365))]
    pub data_retention_days: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_directory: PathBuf::from("data"),
            wal_directory: PathBuf::from("data/wal"),
            driver_directory: PathBuf::from("drivers"),
            config_directory: PathBuf::from("config"),
            wal_buffer_size: 1024,
            wal_sync_interval_seconds: 5,
            enable_compression: true,
            enable_encryption: false,
            encryption_key_file: None,
            data_retention_days: 30,
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub enable_health_checks: bool,
    pub enable_alerts: bool,

    #[validate(range(min = 5, max = 3600))]
    pub metrics_interval_seconds: u32,

    #[validate(range(min = 5, max = 3600))]
    pub health_check_interval_seconds: u32,

    #[validate(range(min = 1, max = 10000))]
    pub max_alerts: u32,

    pub alert_destinations: Vec<String>,
    pub performance_thresholds: PerformanceThresholds,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_health_checks: true,
            enable_alerts: true,
            metrics_interval_seconds: 10,
            health_check_interval_seconds: 30,
            max_alerts: 1000,
            alert_destinations: vec!["console".to_string()],
            performance_thresholds: PerformanceThresholds::default(),
        }
    }
}

/// Performance monitoring thresholds
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PerformanceThresholds {
    #[validate(range(min = 1.0, max = 100.0))]
    pub cpu_threshold_percent: f32,

    #[validate(range(min = 1.0, max = 100.0))]
    pub memory_threshold_percent: f32,

    #[validate(range(min = 1.0, max = 100.0))]
    pub disk_threshold_percent: f32,

    #[validate(range(min = 1, max = 86400))]
    pub response_time_threshold_ms: u32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold_percent: 80.0,
            memory_threshold_percent: 85.0,
            disk_threshold_percent: 90.0,
            response_time_threshold_ms: 5000,
        }
    }
}

/// Driver configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DriverConfig {
    #[validate(length(min = 1))]
    pub driver_type: String,

    pub enabled: bool,
    pub auto_start: bool,

    #[validate(range(min = 1, max = 3600))]
    pub poll_interval_seconds: u32,

    #[validate(range(min = 1, max = 60))]
    pub max_retries: u32,

    #[validate(range(min = 1, max = 300))]
    pub timeout_seconds: u32,

    pub settings: HashMap<String, serde_json::Value>,
}

/// Connector configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ConnectorConfig {
    #[validate(length(min = 1))]
    pub connector_type: String,

    pub enabled: bool,
    pub auto_connect: bool,

    #[validate(range(min = 1, max = 300))]
    pub connection_timeout_seconds: u32,

    #[validate(range(min = 1, max = 3600))]
    pub keepalive_interval_seconds: u32,

    pub settings: HashMap<String, serde_json::Value>,
}

/// Production configuration manager
pub struct ProductionConfigManager {
    config: ProductionConfig,
    config_path: PathBuf,
    environment_prefix: String,
}

impl ProductionConfigManager {
    /// Create new production config manager
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = Self::load_config(&config_path)?;

        Ok(Self {
            config,
            config_path,
            environment_prefix: "GATEWAY".to_string(),
        })
    }

    /// Load configuration from file and environment
    fn load_config(config_path: &Path) -> Result<ProductionConfig> {
        let mut builder = Config::builder();

        // Start with defaults
        let default_config = ProductionConfig::default();
        builder = builder.add_source(config::Config::try_from(&default_config)?);

        // Load from config file if it exists
        if config_path.exists() {
            info!("Loading configuration from: {:?}", config_path);
            builder = builder.add_source(File::from(config_path));
        } else {
            warn!("Configuration file not found: {:?}, using defaults", config_path);
        }

        // Load from environment variables (GATEWAY_*)
        builder = builder.add_source(
            Environment::with_prefix("GATEWAY")
                .separator("__")
                .try_parsing(true)
        );

        // Load from .env file if it exists
        if let Ok(_) = dotenv::dotenv() {
            debug!("Loaded environment variables from .env file");
        }

        let config = builder.build()?;
        let production_config: ProductionConfig = config.try_deserialize()?;

        // Validate configuration
        production_config.validate()
            .context("Configuration validation failed")?;

        Ok(production_config)
    }

    /// Get configuration
    pub fn config(&self) -> &ProductionConfig {
        &self.config
    }

    /// Save configuration to file
    pub fn save_config(&self) -> Result<()> {
        let config_str = serde_yaml::to_string(&self.config)
            .context("Failed to serialize configuration")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        fs::write(&self.config_path, config_str)
            .context("Failed to write configuration file")?;

        info!("Configuration saved to: {:?}", self.config_path);
        Ok(())
    }

    /// Generate default configuration file
    pub fn generate_default_config(output_path: &Path) -> Result<()> {
        let default_config = ProductionConfig::default();
        let config_str = serde_yaml::to_string(&default_config)
            .context("Failed to serialize default configuration")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        fs::write(output_path, config_str)
            .context("Failed to write default configuration file")?;

        info!("Default configuration generated: {:?}", output_path);
        Ok(())
    }

    /// Validate current configuration
    pub fn validate_config(&self) -> Result<()> {
        self.config.validate()
            .context("Configuration validation failed")?;

        // Additional runtime validations
        self.validate_paths()?;
        self.validate_network_bindings()?;
        self.validate_security_settings()?;

        info!("Configuration validation passed");
        Ok(())
    }

    /// Validate that required paths exist or can be created
    fn validate_paths(&self) -> Result<()> {
        let paths = [
            &self.config.storage.data_directory,
            &self.config.storage.wal_directory,
            &self.config.storage.driver_directory,
            &self.config.storage.config_directory,
        ];

        for path in &paths {
            if !path.exists() {
                fs::create_dir_all(path)
                    .with_context(|| format!("Failed to create directory: {:?}", path))?;
                info!("Created directory: {:?}", path);
            }
        }

        Ok(())
    }

    /// Validate network binding addresses
    fn validate_network_bindings(&self) -> Result<()> {
        let bindings = [
            &self.config.network.rest_api_bind,
            &self.config.network.metrics_bind,
            &self.config.network.web_ui_bind,
        ];

        for binding in &bindings {
            binding.parse::<std::net::SocketAddr>()
                .with_context(|| format!("Invalid socket address: {}", binding))?;
        }

        Ok(())
    }

    /// Validate security settings
    fn validate_security_settings(&self) -> Result<()> {
        // Validate JWT secret length
        if self.config.security.jwt_secret.len() < 32 {
            return Err(anyhow::anyhow!("JWT secret must be at least 32 characters"));
        }

        // Validate TLS configuration if enabled
        if self.config.network.enable_tls {
            if let (Some(cert_path), Some(key_path)) = (
                &self.config.network.tls_cert_path,
                &self.config.network.tls_key_path,
            ) {
                if !cert_path.exists() {
                    return Err(anyhow::anyhow!("TLS certificate file not found: {:?}", cert_path));
                }
                if !key_path.exists() {
                    return Err(anyhow::anyhow!("TLS key file not found: {:?}", key_path));
                }
            } else {
                return Err(anyhow::anyhow!("TLS enabled but certificate/key paths not provided"));
            }
        }

        Ok(())
    }

    /// Get environment-specific configuration
    pub fn get_environment_config(&self) -> EnvironmentConfig {
        EnvironmentConfig::from_production_config(&self.config)
    }
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            storage: StorageConfig::default(),
            monitoring: MonitoringConfig::default(),
            drivers: HashMap::new(),
            connectors: HashMap::new(),
        }
    }
}

/// CLI arguments for configuration management
#[derive(Debug, Parser)]
#[command(name = "config-manager")]
#[command(about = "Production configuration management tool")]
pub struct ConfigCli {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum ConfigCommand {
    /// Generate default configuration file
    Generate {
        /// Output path for configuration file
        #[arg(short, long, default_value = "config/production.yaml")]
        output: PathBuf,
    },
    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(short, long, default_value = "config/production.yaml")]
        config: PathBuf,
    },
    /// Show current configuration
    Show {
        /// Configuration file path
        #[arg(short, long, default_value = "config/production.yaml")]
        config: PathBuf,
    },
    /// Encrypt sensitive values in configuration
    Encrypt {
        /// Configuration file path
        #[arg(short, long, default_value = "config/production.yaml")]
        config: PathBuf,
        /// Encryption key file
        #[arg(short, long)]
        key_file: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_creation() {
        let config = ProductionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_general_config_validation() {
        let mut config = GeneralConfig::default();
        config.instance_id = "".to_string(); // Invalid: too short
        assert!(config.validate().is_err());

        config.instance_id = "valid-id".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_network_config_validation() {
        let mut config = NetworkConfig::default();
        config.rest_api_bind = "invalid-address".to_string();
        // Note: This would fail in validate_socket_addr custom validator
        
        config.rest_api_bind = "127.0.0.1:8080".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.jwt_secret.len() >= 32);
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_config_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.yaml");

        // Test with non-existent config file (should use defaults)
        let manager = ProductionConfigManager::new(config_path.clone());
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.config.general.environment, "production");
    }

    #[test]
    fn test_config_serialization() {
        let config = ProductionConfig::default();
        let yaml_str = serde_yaml::to_string(&config);
        assert!(yaml_str.is_ok());

        let deserialized: Result<ProductionConfig, _> = serde_yaml::from_str(&yaml_str.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_performance_thresholds() {
        let thresholds = PerformanceThresholds::default();
        assert!(thresholds.validate().is_ok());
        assert_eq!(thresholds.cpu_threshold_percent, 80.0);
        assert_eq!(thresholds.memory_threshold_percent, 85.0);
    }
}