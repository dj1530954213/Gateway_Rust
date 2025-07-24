/*!
# Dynamic Driver Registry

Manages registration and discovery of dynamic drivers.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use anyhow::{Context, Result};
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

use crate::loader::{DynamicLoader, DynamicLibrary};
use crate::metadata::DriverMetadata;
use crate::error::DynamicDriverError;

/// Driver registration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverRegistration {
    /// Driver ID (unique identifier)
    pub id: String,
    
    /// Path to driver library
    pub path: PathBuf,
    
    /// Driver metadata
    pub metadata: DriverMetadata,
    
    /// Whether driver is currently loaded
    pub loaded: bool,
    
    /// Whether driver is enabled for auto-loading
    pub enabled: bool,
    
    /// Load order priority (lower = higher priority)
    pub priority: u32,
    
    /// Driver configuration
    pub config: serde_json::Value,
}

impl DriverRegistration {
    /// Create new driver registration
    pub fn new<P: AsRef<Path>>(
        id: String,
        path: P,
        metadata: DriverMetadata,
    ) -> Self {
        Self {
            id,
            path: path.as_ref().to_path_buf(),
            metadata,
            loaded: false,
            enabled: true,
            priority: 100,
            config: serde_json::Value::Null,
        }
    }
    
    /// Set driver configuration
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }
    
    /// Set load priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Dynamic driver registry
pub struct DynamicDriverRegistry {
    /// Dynamic loader for loading libraries
    loader: Arc<DynamicLoader>,
    
    /// Registered drivers
    registrations: Arc<RwLock<HashMap<String, DriverRegistration>>>,
    
    /// Loaded driver instances
    loaded_drivers: Arc<RwLock<HashMap<String, Arc<DynamicLibrary>>>>,
}

impl DynamicDriverRegistry {
    /// Create new driver registry
    pub fn new() -> Result<Self> {
        Ok(Self {
            loader: Arc::new(DynamicLoader::new()?),
            registrations: Arc::new(RwLock::new(HashMap::new())),
            loaded_drivers: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Create registry with custom loader
    pub fn with_loader(loader: DynamicLoader) -> Self {
        Self {
            loader: Arc::new(loader),
            registrations: Arc::new(RwLock::new(HashMap::new())),
            loaded_drivers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a driver
    pub fn register_driver(&self, registration: DriverRegistration) -> Result<()> {
        let driver_id = registration.id.clone();
        
        info!("Registering driver: {}", driver_id);
        
        // Validate driver path exists
        if !registration.path.exists() {
            return Err(DynamicDriverError::LibraryNotFound(registration.path.clone()).into());
        }
        
        // Check for duplicate registration
        {
            let registrations = self.registrations.read().unwrap();
            if registrations.contains_key(&driver_id) {
                warn!("Driver already registered: {}", driver_id);
                return Err(DynamicDriverError::DriverAlreadyRegistered(driver_id).into());
            }
        }
        
        // Add to registry
        {
            let mut registrations = self.registrations.write().unwrap();
            registrations.insert(driver_id.clone(), registration);
        }
        
        info!("Successfully registered driver: {}", driver_id);
        Ok(())
    }
    
    /// Unregister a driver
    pub fn unregister_driver(&self, driver_id: &str) -> Result<()> {
        info!("Unregistering driver: {}", driver_id);
        
        // Unload if currently loaded
        self.unload_driver(driver_id)?;
        
        // Remove from registry
        {
            let mut registrations = self.registrations.write().unwrap();
            if registrations.remove(driver_id).is_none() {
                warn!("Attempted to unregister non-existent driver: {}", driver_id);
                return Err(DynamicDriverError::DriverNotFound(driver_id.to_string()).into());
            }
        }
        
        info!("Successfully unregistered driver: {}", driver_id);
        Ok(())
    }
    
    /// Load a registered driver
    pub fn load_driver(&self, driver_id: &str) -> Result<Arc<DynamicLibrary>> {
        info!("Loading driver: {}", driver_id);
        
        // Get registration
        let registration = {
            let registrations = self.registrations.read().unwrap();
            registrations
                .get(driver_id)
                .ok_or_else(|| DynamicDriverError::DriverNotFound(driver_id.to_string()))?
                .clone()
        };
        
        // Check if driver is enabled
        if !registration.enabled {
            return Err(DynamicDriverError::DriverDisabled(driver_id.to_string()).into());
        }
        
        // Check if already loaded
        {
            let loaded_drivers = self.loaded_drivers.read().unwrap();
            if let Some(driver) = loaded_drivers.get(driver_id) {
                debug!("Driver already loaded: {}", driver_id);
                return Ok(driver.clone());
            }
        }
        
        // Load the driver library
        let library = self.loader.load_driver(&registration.path)
            .with_context(|| format!("Failed to load driver: {}", driver_id))?;
        
        // Update registration status
        {
            let mut registrations = self.registrations.write().unwrap();
            if let Some(reg) = registrations.get_mut(driver_id) {
                reg.loaded = true;
            }
        }
        
        // Cache loaded driver
        {
            let mut loaded_drivers = self.loaded_drivers.write().unwrap();
            loaded_drivers.insert(driver_id.to_string(), library.clone());
        }
        
        info!("Successfully loaded driver: {}", driver_id);
        Ok(library)
    }
    
    /// Unload a driver
    pub fn unload_driver(&self, driver_id: &str) -> Result<()> {
        info!("Unloading driver: {}", driver_id);
        
        // Remove from loaded drivers
        let library_path = {
            let mut loaded_drivers = self.loaded_drivers.write().unwrap();
            if let Some(library) = loaded_drivers.remove(driver_id) {
                Some(library.path().to_path_buf())
            } else {
                debug!("Driver not currently loaded: {}", driver_id);
                None
            }
        };
        
        // Unload from loader cache if it was loaded
        if let Some(path) = library_path {
            self.loader.unload_driver(&path)?;
        }
        
        // Update registration status
        {
            let mut registrations = self.registrations.write().unwrap();
            if let Some(reg) = registrations.get_mut(driver_id) {
                reg.loaded = false;
            }
        }
        
        info!("Successfully unloaded driver: {}", driver_id);
        Ok(())
    }
    
    /// Get driver registration
    pub fn get_registration(&self, driver_id: &str) -> Option<DriverRegistration> {
        let registrations = self.registrations.read().unwrap();
        registrations.get(driver_id).cloned()
    }
    
    /// Get all registered drivers
    pub fn list_drivers(&self) -> Vec<DriverRegistration> {
        let registrations = self.registrations.read().unwrap();
        registrations.values().cloned().collect()
    }
    
    /// Get loaded drivers
    pub fn list_loaded_drivers(&self) -> Vec<String> {
        let loaded_drivers = self.loaded_drivers.read().unwrap();
        loaded_drivers.keys().cloned().collect()
    }
    
    /// Load all enabled drivers
    pub fn load_all_enabled(&self) -> Result<Vec<String>> {
        let mut loaded = Vec::new();
        
        // Get enabled drivers sorted by priority
        let mut enabled_drivers = {
            let registrations = self.registrations.read().unwrap();
            registrations
                .values()
                .filter(|reg| reg.enabled && !reg.loaded)
                .cloned()
                .collect::<Vec<_>>()
        };
        
        // Sort by priority (lower number = higher priority)
        enabled_drivers.sort_by_key(|reg| reg.priority);
        
        for registration in enabled_drivers {
            match self.load_driver(&registration.id) {
                Ok(_) => {
                    loaded.push(registration.id.clone());
                    info!("Auto-loaded driver: {}", registration.id);
                }
                Err(e) => {
                    error!("Failed to auto-load driver {}: {}", registration.id, e);
                }
            }
        }
        
        Ok(loaded)
    }
    
    /// Unload all drivers
    pub fn unload_all(&self) -> Result<()> {
        let driver_ids = self.list_loaded_drivers();
        
        for driver_id in driver_ids {
            if let Err(e) = self.unload_driver(&driver_id) {
                error!("Failed to unload driver {}: {}", driver_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let registrations = self.registrations.read().unwrap();
        let loaded_drivers = self.loaded_drivers.read().unwrap();
        
        let total = registrations.len();
        let loaded = loaded_drivers.len();
        let enabled = registrations.values().filter(|reg| reg.enabled).count();
        let disabled = total - enabled;
        
        RegistryStats {
            total,
            loaded,
            enabled,
            disabled,
        }
    }
    
    /// Discover drivers in a directory
    pub fn discover_drivers<P: AsRef<Path>>(&self, directory: P) -> Result<Vec<DriverRegistration>> {
        let directory = directory.as_ref();
        
        if !directory.exists() || !directory.is_dir() {
            return Err(DynamicDriverError::DirectoryNotFound(directory.to_path_buf()).into());
        }
        
        let mut discovered = Vec::new();
        
        // Look for .so, .dll, .dylib files
        let extensions = ["so", "dll", "dylib"];
        
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extensions.contains(&extension.to_string_lossy().as_ref()) {
                    match self.try_discover_driver(&path) {
                        Ok(registration) => {
                            discovered.push(registration);
                            info!("Discovered driver: {:?}", path);
                        }
                        Err(e) => {
                            debug!("Failed to discover driver {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        
        Ok(discovered)
    }
    
    /// Try to discover a single driver
    fn try_discover_driver(&self, path: &Path) -> Result<DriverRegistration> {
        // Try to load library to get metadata
        let library = self.loader.load_driver(path)?;
        let metadata = library.metadata().clone();
        
        // Generate driver ID from filename and metadata
        let filename = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let driver_id = format!("{}_{}", filename, metadata.name);
        
        // Unload the library (we only wanted metadata)
        self.loader.unload_driver(path)?;
        
        Ok(DriverRegistration::new(driver_id, path, metadata))
    }
}

impl Default for DynamicDriverRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create default DynamicDriverRegistry")
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total: usize,
    pub loaded: usize,
    pub enabled: usize,
    pub disabled: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_registry_creation() {
        let registry = DynamicDriverRegistry::new();
        assert!(registry.is_ok());
        
        let registry = registry.unwrap();
        let stats = registry.stats();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.loaded, 0);
    }
    
    #[test]
    fn test_driver_registration() {
        let registry = DynamicDriverRegistry::new().unwrap();
        
        // Create a dummy registration
        let metadata = DriverMetadata {
            name: "test-driver".to_string(),
            version: "1.0.0".to_string(),
            description: "Test driver".to_string(),
            protocols: vec!["test".to_string()],
            author: "Test Author".to_string(),
            min_abi_version: 1,
            max_abi_version: 1,
        };
        
        let temp_dir = TempDir::new().unwrap();
        let driver_path = temp_dir.path().join("test.so");
        std::fs::write(&driver_path, b"dummy").unwrap();
        
        let registration = DriverRegistration::new(
            "test-driver".to_string(),
            &driver_path,
            metadata,
        );
        
        // Registration should fail because the file is not a valid library
        // but we're testing the registration logic
        let result = registry.register_driver(registration);
        assert!(result.is_ok());
        
        let stats = registry.stats();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.enabled, 1);
    }
    
    #[test]
    fn test_discovery() {
        let registry = DynamicDriverRegistry::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        
        // Create some dummy files
        std::fs::write(temp_dir.path().join("driver1.so"), b"dummy").unwrap();
        std::fs::write(temp_dir.path().join("driver2.dll"), b"dummy").unwrap();
        std::fs::write(temp_dir.path().join("not_a_driver.txt"), b"dummy").unwrap();
        
        let discovered = registry.discover_drivers(temp_dir.path()).unwrap();
        
        // Should find the .so and .dll files but not .txt
        // They will fail to load but discovery should handle that gracefully
        assert_eq!(discovered.len(), 0); // No valid drivers found
    }
}