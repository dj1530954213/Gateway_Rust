/*!
# Dynamic Driver System

This module provides dynamic loading and hot-plugging capabilities for drivers,
allowing runtime loading of .so/.dll libraries that implement the Driver trait.

## Core Components

- `DriverAbi`: Stable ABI interface for cross-library compatibility
- `DynamicLoader`: Loads and manages dynamic driver libraries  
- `DriverRegistry`: Registry for dynamic driver discovery and management
- `SecurityVerifier`: Validates driver signatures and permissions
- `HotSwapManager`: Handles hot-plugging operations

## Usage

```rust,no_run
use dynamic_driver::{DynamicLoader, DriverMetadata};

let loader = DynamicLoader::new().unwrap();
let driver = loader.load_driver("path/to/driver.so").unwrap();
```
*/

pub mod abi;
pub mod loader;
pub mod registry;
pub mod metadata;
pub mod security;
pub mod hotswap;
pub mod error;

pub use abi::*;
pub use loader::*;
pub use registry::*;
pub use metadata::*;
pub use security::*;
pub use hotswap::*;
pub use error::*;

/// Re-export core traits for convenience
pub use driver_manager::Driver;
pub use frame_bus::{FrameSender, FrameReceiver};
pub use endpoint_kit::EndpointHandle;