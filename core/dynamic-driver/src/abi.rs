/*!
# Stable ABI Interface

Defines the Application Binary Interface (ABI) for dynamic drivers.
This interface must remain stable across versions to ensure compatibility.
*/

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use abi_stable::{
    std_types::{RResult, RString, RVec},
    StableAbi, 
};

/// ABI version for compatibility checking
pub const DRIVER_ABI_VERSION: u32 = 1;

/// Magic number to identify valid driver libraries
pub const DRIVER_MAGIC: u64 = 0x4452495645524142; // "DRIVERAB"

/// Driver ABI result type
pub type DriverResult<T> = RResult<T, RString>;

/// Stable driver interface that crosses the ABI boundary
#[repr(C)]
#[derive(StableAbi)]
pub struct DriverAbi {
    /// ABI version this driver was compiled with
    pub abi_version: u32,
    
    /// Magic number for validation
    pub magic: u64,
    
    /// Driver metadata
    pub get_metadata: extern "C" fn() -> DriverMetadataAbi,
    
    /// Create new driver instance
    pub create_driver: extern "C" fn() -> DriverResult<*mut c_void>,
    
    /// Initialize driver with configuration
    pub init_driver: extern "C" fn(
        driver: *mut c_void,
        config: *const c_char,
    ) -> DriverResult<()>,
    
    /// Connect driver to endpoint
    pub connect_driver: extern "C" fn(
        driver: *mut c_void,
        endpoint_handle: *const c_void,
    ) -> DriverResult<()>,
    
    /// Start driver read loop
    pub start_read_loop: extern "C" fn(
        driver: *mut c_void,
        frame_sender: *const c_void,
    ) -> DriverResult<()>,
    
    /// Write command to driver
    pub write_command: extern "C" fn(
        driver: *mut c_void,
        cmd_data: *const c_char,
    ) -> DriverResult<()>,
    
    /// Stop driver gracefully
    pub stop_driver: extern "C" fn(driver: *mut c_void) -> DriverResult<()>,
    
    /// Destroy driver instance
    pub destroy_driver: extern "C" fn(driver: *mut c_void),
}

/// Driver metadata that crosses the ABI boundary
#[repr(C)]
#[derive(StableAbi)]
pub struct DriverMetadataAbi {
    /// Driver name
    pub name: RString,
    
    /// Driver version
    pub version: RString,
    
    /// Driver description
    pub description: RString,
    
    /// Supported protocols
    pub protocols: RVec<RString>,
    
    /// Author information
    pub author: RString,
    
    /// Minimum ABI version required
    pub min_abi_version: u32,
    
    /// Maximum ABI version supported
    pub max_abi_version: u32,
}

/// Function signature for the main driver registration function
/// This is the entry point that every dynamic driver must export
pub type RegisterDriverFn = extern "C" fn() -> *const DriverAbi;

/// Standard symbol name for driver registration function
pub const REGISTER_DRIVER_SYMBOL: &str = "register_driver";

/// Validates that a driver ABI is compatible with current version
pub fn validate_abi_compatibility(abi: &DriverAbi) -> Result<(), String> {
    // Check magic number
    if abi.magic != DRIVER_MAGIC {
        return Err(format!(
            "Invalid driver magic number: expected {:#x}, got {:#x}",
            DRIVER_MAGIC, abi.magic
        ));
    }
    
    // Check ABI version
    if abi.abi_version < 1 || abi.abi_version > DRIVER_ABI_VERSION {
        return Err(format!(
            "Incompatible ABI version: expected {}, got {}",
            DRIVER_ABI_VERSION, abi.abi_version
        ));
    }
    
    Ok(())
}

/// Helper to safely convert C string to Rust string
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Result<String, String> {
    if ptr.is_null() {
        return Err("Null pointer".to_string());
    }
    
    CStr::from_ptr(ptr)
        .to_str()
        .map(|s| s.to_string())
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// Helper to convert Rust string to C string  
pub fn string_to_c_str(s: &str) -> Result<CString, String> {
    CString::new(s).map_err(|e| format!("String contains null byte: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_validation() {
        let valid_abi = DriverAbi {
            abi_version: DRIVER_ABI_VERSION,
            magic: DRIVER_MAGIC,
            get_metadata: dummy_get_metadata,
            create_driver: dummy_create_driver,
            init_driver: dummy_init_driver,
            connect_driver: dummy_connect_driver,
            start_read_loop: dummy_start_read_loop,
            write_command: dummy_write_command,
            stop_driver: dummy_stop_driver,
            destroy_driver: dummy_destroy_driver,
        };
        
        assert!(validate_abi_compatibility(&valid_abi).is_ok());
        
        // Test invalid magic
        let invalid_magic = DriverAbi {
            magic: 0x1234,
            ..valid_abi
        };
        assert!(validate_abi_compatibility(&invalid_magic).is_err());
        
        // Test invalid version
        let invalid_version = DriverAbi {
            abi_version: 999,
            ..valid_abi
        };
        assert!(validate_abi_compatibility(&invalid_version).is_err());
    }

    // Dummy implementations for testing
    extern "C" fn dummy_get_metadata() -> DriverMetadataAbi {
        unimplemented!()
    }
    
    extern "C" fn dummy_create_driver() -> DriverResult<*mut c_void> {
        unimplemented!()
    }
    
    extern "C" fn dummy_init_driver(
        _driver: *mut c_void,
        _config: *const c_char,
    ) -> DriverResult<()> {
        unimplemented!()
    }
    
    extern "C" fn dummy_connect_driver(
        _driver: *mut c_void,
        _endpoint_handle: *const c_void,
    ) -> DriverResult<()> {
        unimplemented!()
    }
    
    extern "C" fn dummy_start_read_loop(
        _driver: *mut c_void,
        _frame_sender: *const c_void,
    ) -> DriverResult<()> {
        unimplemented!()
    }
    
    extern "C" fn dummy_write_command(
        _driver: *mut c_void,
        _cmd_data: *const c_char,
    ) -> DriverResult<()> {
        unimplemented!()
    }
    
    extern "C" fn dummy_stop_driver(_driver: *mut c_void) -> DriverResult<()> {
        unimplemented!()
    }
    
    extern "C" fn dummy_destroy_driver(_driver: *mut c_void) {
        // No-op for testing
    }
}