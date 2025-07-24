/*!
# Dynamic Driver Loader

Provides functionality to load, validate, and manage dynamic driver libraries.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use libloading::{Library, Symbol};
use anyhow::{Context, Result};
use tracing::{info, warn, error, debug};

use crate::abi::{
    DriverAbi, RegisterDriverFn, REGISTER_DRIVER_SYMBOL, 
    validate_abi_compatibility
};
use crate::metadata::DriverMetadata;
use crate::security::SecurityVerifier;
use crate::error::DynamicDriverError;

/// Dynamic driver library wrapper
pub struct DynamicLibrary {
    /// The loaded library
    library: Library,
    
    /// Driver ABI interface
    abi: &'static DriverAbi,
    
    /// Driver metadata
    metadata: DriverMetadata,
    
    /// Path to the library file
    path: PathBuf,
    
    /// Security verification result
    verified: bool,
}

impl DynamicLibrary {
    /// Get driver metadata
    pub fn metadata(&self) -> &DriverMetadata {
        &self.metadata
    }
    
    /// Get driver ABI interface
    pub fn abi(&self) -> &'static DriverAbi {
        self.abi
    }
    
    /// Get library path
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Check if library is verified
    pub fn is_verified(&self) -> bool {
        self.verified
    }
}

/// Dynamic driver loader
pub struct DynamicLoader {
    /// Security verifier for driver validation
    verifier: Arc<SecurityVerifier>,
    
    /// Cache of loaded libraries
    cache: Arc<RwLock<HashMap<PathBuf, Arc<DynamicLibrary>>>>,
    
    /// Whether to enforce signature verification
    enforce_verification: bool,
}

impl DynamicLoader {
    /// Create new dynamic loader
    pub fn new() -> Result<Self> {
        Ok(Self {
            verifier: Arc::new(SecurityVerifier::new()?),
            cache: Arc::new(RwLock::new(HashMap::new())),
            enforce_verification: true,
        })
    }
    
    /// Create loader with custom verifier
    pub fn with_verifier(verifier: SecurityVerifier) -> Self {
        Self {
            verifier: Arc::new(verifier),
            cache: Arc::new(RwLock::new(HashMap::new())),
            enforce_verification: true,
        }
    }
    
    /// Set whether to enforce signature verification
    pub fn set_enforce_verification(&mut self, enforce: bool) {
        self.enforce_verification = enforce;
    }
    
    /// Load a driver from library path
    pub fn load_driver<P: AsRef<Path>>(&self, path: P) -> Result<Arc<DynamicLibrary>> {
        let path = path.as_ref().to_path_buf();
        
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(library) = cache.get(&path) {
                debug!("Driver already loaded from cache: {:?}", path);
                return Ok(library.clone());
            }
        }
        
        info!("Loading dynamic driver: {:?}", path);
        
        // Verify library exists
        if !path.exists() {
            return Err(DynamicDriverError::LibraryNotFound(path).into());
        }
        
        // Load the library
        let library = unsafe {
            Library::new(&path)
                .with_context(|| format!("Failed to load library: {:?}", path))?
        };
        
        // Get the registration function
        let register_fn: Symbol<RegisterDriverFn> = unsafe {
            library
                .get(REGISTER_DRIVER_SYMBOL.as_bytes())
                .with_context(|| {
                    format!("Driver missing registration function: {}", REGISTER_DRIVER_SYMBOL)
                })?
        };
        
        // Call registration function to get ABI
        let abi_ptr = register_fn();
        if abi_ptr.is_null() {
            return Err(DynamicDriverError::InvalidAbi("Null ABI pointer".to_string()).into());
        }
        
        let abi = unsafe { &*abi_ptr };
        
        // Validate ABI compatibility
        validate_abi_compatibility(abi)
            .map_err(|e| DynamicDriverError::InvalidAbi(e))?;
        
        // Get driver metadata
        let metadata_abi = (abi.get_metadata)();
        let metadata = DriverMetadata::from_abi(&metadata_abi)?;
        
        // Verify driver signature if enforcement is enabled
        let verified = if self.enforce_verification {
            match self.verifier.verify_driver(&path, &metadata) {
                Ok(()) => {
                    info!("Driver signature verified: {:?}", path);
                    true
                }
                Err(e) => {
                    error!("Driver signature verification failed: {:?} - {}", path, e);
                    return Err(e);
                }
            }
        } else {
            warn!("Signature verification disabled for: {:?}", path);
            false
        };
        
        let dynamic_lib = Arc::new(DynamicLibrary {
            library,
            abi,
            metadata,
            path: path.clone(),
            verified,
        });
        
        // Cache the loaded library
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(path.clone(), dynamic_lib.clone());
        }
        
        info!("Successfully loaded driver: {} v{}", 
               dynamic_lib.metadata.name, dynamic_lib.metadata.version);
        
        Ok(dynamic_lib)
    }
    
    /// Unload a driver library
    pub fn unload_driver<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        
        let mut cache = self.cache.write().unwrap();
        if let Some(_library) = cache.remove(&path) {
            info!("Unloaded driver: {:?}", path);
            // Library will be dropped and unloaded when Arc count reaches 0
        } else {
            warn!("Attempted to unload non-cached driver: {:?}", path);
        }
        
        Ok(())
    }
    
    /// Get list of loaded drivers
    pub fn loaded_drivers(&self) -> Vec<(PathBuf, DriverMetadata)> {
        let cache = self.cache.read().unwrap();
        cache
            .iter()
            .map(|(path, lib)| (path.clone(), lib.metadata.clone()))
            .collect()
    }
    
    /// Clear all cached drivers
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        info!("Driver cache cleared");
    }
    
    /// Check if driver is loaded
    pub fn is_loaded<P: AsRef<Path>>(&self, path: P) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains_key(path.as_ref())
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().unwrap();
        let loaded = cache.len();
        let verified = cache.values().filter(|lib| lib.verified).count();
        (loaded, verified)
    }
}

impl Default for DynamicLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create default DynamicLoader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_loader_creation() {
        let loader = DynamicLoader::new();
        assert!(loader.is_ok());
        
        let loader = loader.unwrap();
        let (loaded, verified) = loader.cache_stats();
        assert_eq!(loaded, 0);
        assert_eq!(verified, 0);
    }
    
    #[test]
    fn test_library_not_found() {
        let loader = DynamicLoader::new().unwrap();
        let result = loader.load_driver("/nonexistent/path.so");
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("not found") || e.to_string().contains("No such file"));
        }
    }
    
    #[test]
    fn test_cache_operations() {
        let loader = DynamicLoader::new().unwrap();
        
        // Initially empty
        assert_eq!(loader.loaded_drivers().len(), 0);
        assert!(!loader.is_loaded("/some/path.so"));
        
        // Clear empty cache
        loader.clear_cache();
        assert_eq!(loader.loaded_drivers().len(), 0);
    }
    
    #[test]
    fn test_verification_toggle() {
        let mut loader = DynamicLoader::new().unwrap();
        
        // Default should enforce verification
        loader.set_enforce_verification(false);
        loader.set_enforce_verification(true);
        
        // Should not panic or error
    }
}