/*!
# Driver Metadata

Defines metadata structures for dynamic drivers.
*/

use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use crate::abi::DriverMetadataAbi;
use crate::error::DynamicDriverError;

/// Driver metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriverMetadata {
    /// Driver name
    pub name: String,
    
    /// Driver version (semver)
    pub version: String,
    
    /// Driver description
    pub description: String,
    
    /// Supported protocols
    pub protocols: Vec<String>,
    
    /// Author information
    pub author: String,
    
    /// Minimum ABI version required
    pub min_abi_version: u32,
    
    /// Maximum ABI version supported
    pub max_abi_version: u32,
}

impl DriverMetadata {
    /// Create new driver metadata
    pub fn new(
        name: String,
        version: String,
        description: String,
        protocols: Vec<String>,
        author: String,
    ) -> Self {
        Self {
            name,
            version,
            description,
            protocols,
            author,
            min_abi_version: 1,
            max_abi_version: 1,
        }
    }
    
    /// Set ABI version range
    pub fn with_abi_version_range(mut self, min: u32, max: u32) -> Self {
        self.min_abi_version = min;
        self.max_abi_version = max;
        self
    }
    
    /// Convert from ABI metadata
    pub fn from_abi(abi_metadata: &DriverMetadataAbi) -> Result<Self> {
        Ok(Self {
            name: abi_metadata.name.as_str().to_string(),
            version: abi_metadata.version.as_str().to_string(),
            description: abi_metadata.description.as_str().to_string(),
            protocols: abi_metadata.protocols.iter()
                .map(|s| s.as_str().to_string())
                .collect(),
            author: abi_metadata.author.as_str().to_string(),
            min_abi_version: abi_metadata.min_abi_version,
            max_abi_version: abi_metadata.max_abi_version,
        })
    }
    
    /// Validate metadata
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(DynamicDriverError::InvalidMetadata("Name cannot be empty".to_string()).into());
        }
        
        if self.version.is_empty() {
            return Err(DynamicDriverError::InvalidMetadata("Version cannot be empty".to_string()).into());
        }
        
        // Basic semver validation
        if !self.is_valid_semver(&self.version) {
            return Err(DynamicDriverError::InvalidMetadata(
                format!("Invalid version format: {}", self.version)
            ).into());
        }
        
        if self.protocols.is_empty() {
            return Err(DynamicDriverError::InvalidMetadata("At least one protocol must be specified".to_string()).into());
        }
        
        if self.min_abi_version > self.max_abi_version {
            return Err(DynamicDriverError::InvalidMetadata(
                format!("min_abi_version ({}) cannot be greater than max_abi_version ({})", 
                        self.min_abi_version, self.max_abi_version)
            ).into());
        }
        
        Ok(())
    }
    
    /// Check if version string is valid semver format
    fn is_valid_semver(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        
        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }
    
    /// Check if this driver supports a specific protocol
    pub fn supports_protocol(&self, protocol: &str) -> bool {
        self.protocols.iter().any(|p| p.eq_ignore_ascii_case(protocol))
    }
    
    /// Get driver identifier (name + version)
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
    
    /// Check ABI compatibility with current version
    pub fn is_abi_compatible(&self, abi_version: u32) -> bool {
        abi_version >= self.min_abi_version && abi_version <= self.max_abi_version
    }
}

/// Extended driver metadata with additional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedDriverMetadata {
    /// Base metadata
    #[serde(flatten)]
    pub base: DriverMetadata,
    
    /// License information
    pub license: Option<String>,
    
    /// Homepage URL
    pub homepage: Option<String>,
    
    /// Repository URL
    pub repository: Option<String>,
    
    /// Documentation URL
    pub documentation: Option<String>,
    
    /// Keywords for discovery
    pub keywords: Vec<String>,
    
    /// Driver category
    pub category: Option<String>,
    
    /// Required permissions
    pub permissions: Vec<String>,
    
    /// Dependencies on other drivers
    pub dependencies: Vec<DriverDependency>,
    
    /// Configuration schema
    pub config_schema: Option<serde_json::Value>,
}

/// Driver dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverDependency {
    /// Dependency name
    pub name: String,
    
    /// Version requirement (semver range)
    pub version: String,
    
    /// Whether dependency is optional
    pub optional: bool,
}

impl ExtendedDriverMetadata {
    /// Create from base metadata
    pub fn from_base(base: DriverMetadata) -> Self {
        Self {
            base,
            license: None,
            homepage: None,
            repository: None,
            documentation: None,
            keywords: Vec::new(),
            category: None,
            permissions: Vec::new(),
            dependencies: Vec::new(),
            config_schema: None,
        }
    }
    
    /// Add dependency
    pub fn add_dependency(mut self, name: String, version: String, optional: bool) -> Self {
        self.dependencies.push(DriverDependency {
            name,
            version,
            optional,
        });
        self
    }
    
    /// Add permission
    pub fn add_permission(mut self, permission: String) -> Self {
        self.permissions.push(permission);
        self
    }
    
    /// Add keyword
    pub fn add_keyword(mut self, keyword: String) -> Self {
        self.keywords.push(keyword);
        self
    }
    
    /// Set configuration schema
    pub fn with_config_schema(mut self, schema: serde_json::Value) -> Self {
        self.config_schema = Some(schema);
        self
    }
    
    /// Validate extended metadata
    pub fn validate(&self) -> Result<()> {
        // Validate base metadata first
        self.base.validate()?;
        
        // Validate URLs if present
        if let Some(ref homepage) = self.homepage {
            if !self.is_valid_url(homepage) {
                return Err(DynamicDriverError::InvalidMetadata(
                    format!("Invalid homepage URL: {}", homepage)
                ).into());
            }
        }
        
        if let Some(ref repository) = self.repository {
            if !self.is_valid_url(repository) {
                return Err(DynamicDriverError::InvalidMetadata(
                    format!("Invalid repository URL: {}", repository)
                ).into());
            }
        }
        
        // Validate dependencies
        for dep in &self.dependencies {
            if dep.name.is_empty() {
                return Err(DynamicDriverError::InvalidMetadata(
                    "Dependency name cannot be empty".to_string()
                ).into());
            }
            
            if dep.version.is_empty() {
                return Err(DynamicDriverError::InvalidMetadata(
                    "Dependency version cannot be empty".to_string()
                ).into());
            }
        }
        
        Ok(())
    }
    
    /// Simple URL validation
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let metadata = DriverMetadata::new(
            "test-driver".to_string(),
            "1.0.0".to_string(),
            "Test driver".to_string(),
            vec!["modbus".to_string()],
            "Test Author".to_string(),
        );
        
        assert_eq!(metadata.name, "test-driver");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.protocols.len(), 1);
        assert!(metadata.supports_protocol("modbus"));
        assert!(metadata.supports_protocol("MODBUS")); // Case insensitive
        assert!(!metadata.supports_protocol("opcua"));
    }
    
    #[test]
    fn test_metadata_validation() {
        let valid_metadata = DriverMetadata::new(
            "test-driver".to_string(),
            "1.0.0".to_string(),
            "Test driver".to_string(),
            vec!["modbus".to_string()],
            "Test Author".to_string(),
        );
        assert!(valid_metadata.validate().is_ok());
        
        // Empty name
        let invalid_metadata = DriverMetadata::new(
            "".to_string(),
            "1.0.0".to_string(),
            "Test driver".to_string(),
            vec!["modbus".to_string()],
            "Test Author".to_string(),
        );
        assert!(invalid_metadata.validate().is_err());
        
        // Invalid version
        let invalid_metadata = DriverMetadata::new(
            "test-driver".to_string(),
            "invalid".to_string(),
            "Test driver".to_string(),
            vec!["modbus".to_string()],
            "Test Author".to_string(),
        );
        assert!(invalid_metadata.validate().is_err());
        
        // No protocols
        let invalid_metadata = DriverMetadata::new(
            "test-driver".to_string(),
            "1.0.0".to_string(),
            "Test driver".to_string(),
            vec![],
            "Test Author".to_string(),
        );
        assert!(invalid_metadata.validate().is_err());
    }
    
    #[test]
    fn test_semver_validation() {
        let metadata = DriverMetadata::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "test".to_string(),
            vec!["test".to_string()],
            "test".to_string(),
        );
        
        assert!(metadata.is_valid_semver("1.0.0"));
        assert!(metadata.is_valid_semver("0.1.2"));
        assert!(metadata.is_valid_semver("10.20.30"));
        
        assert!(!metadata.is_valid_semver("1.0"));
        assert!(!metadata.is_valid_semver("1.0.0.1"));
        assert!(!metadata.is_valid_semver("v1.0.0"));
        assert!(!metadata.is_valid_semver("1.0.a"));
    }
    
    #[test]
    fn test_abi_compatibility() {
        let metadata = DriverMetadata::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "test".to_string(),
            vec!["test".to_string()],
            "test".to_string(),
        ).with_abi_version_range(1, 3);
        
        assert!(metadata.is_abi_compatible(1));
        assert!(metadata.is_abi_compatible(2));
        assert!(metadata.is_abi_compatible(3));
        assert!(!metadata.is_abi_compatible(0));
        assert!(!metadata.is_abi_compatible(4));
    }
    
    #[test]
    fn test_identifier() {
        let metadata = DriverMetadata::new(
            "test-driver".to_string(),
            "1.2.3".to_string(),
            "test".to_string(),
            vec!["test".to_string()],
            "test".to_string(),
        );
        
        assert_eq!(metadata.identifier(), "test-driver@1.2.3");
    }
    
    #[test]
    fn test_extended_metadata() {
        let base = DriverMetadata::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "test".to_string(),
            vec!["test".to_string()],
            "test".to_string(),
        );
        
        let extended = ExtendedDriverMetadata::from_base(base)
            .add_dependency("core-lib".to_string(), "^1.0".to_string(), false)
            .add_permission("read_device".to_string())
            .add_keyword("industrial".to_string())
            .with_config_schema(serde_json::json!({"type": "object"}));
        
        assert_eq!(extended.dependencies.len(), 1);
        assert_eq!(extended.permissions.len(), 1);
        assert_eq!(extended.keywords.len(), 1);
        assert!(extended.config_schema.is_some());
        
        assert!(extended.validate().is_ok());
    }
}