/*!
# Hot Swap Management

Provides safe hot-swapping capabilities for dynamic drivers.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use anyhow::{Context, Result};
use tracing::{info, warn, error, debug};

use crate::loader::{DynamicLoader, DynamicLibrary};
use crate::registry::{DynamicDriverRegistry, DriverRegistration};
use crate::metadata::DriverMetadata;
use crate::error::DynamicDriverError;

/// Hot swap operation type
#[derive(Debug, Clone, PartialEq)]
pub enum HotSwapOperation {
    /// Load new driver
    Load {
        driver_id: String,
        library_path: PathBuf,
    },
    /// Unload existing driver
    Unload {
        driver_id: String,
    },
    /// Replace existing driver with new version
    Replace {
        driver_id: String,
        old_path: PathBuf,
        new_path: PathBuf,
    },
    /// Reload driver from same path
    Reload {
        driver_id: String,
        library_path: PathBuf,
    },
}

/// Hot swap operation status
#[derive(Debug, Clone, PartialEq)]
pub enum HotSwapStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    RolledBack,
}

/// Hot swap operation context
#[derive(Debug, Clone)]
pub struct HotSwapContext {
    pub operation: HotSwapOperation,
    pub status: HotSwapStatus,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub rollback_info: Option<RollbackInfo>,
}

impl HotSwapContext {
    /// Create new hot swap context
    pub fn new(operation: HotSwapOperation) -> Self {
        Self {
            operation,
            status: HotSwapStatus::Pending,
            started_at: Instant::now(),
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
            rollback_info: None,
        }
    }
    
    /// Mark as completed
    pub fn complete(&mut self) {
        self.status = HotSwapStatus::Completed;
        self.completed_at = Some(Instant::now());
    }
    
    /// Mark as failed
    pub fn fail(&mut self, error: String) {
        self.status = HotSwapStatus::Failed(error);
        self.completed_at = Some(Instant::now());
    }
    
    /// Check if can retry
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
    
    /// Increment retry count
    pub fn retry(&mut self) {
        self.retry_count += 1;
        self.status = HotSwapStatus::Pending;
    }
    
    /// Get operation duration
    pub fn duration(&self) -> Duration {
        if let Some(completed_at) = self.completed_at {
            completed_at.duration_since(self.started_at)
        } else {
            Instant::now().duration_since(self.started_at)
        }
    }
}

/// Rollback information for failed operations
#[derive(Debug, Clone)]
pub struct RollbackInfo {
    pub previous_library: Option<PathBuf>,
    pub previous_metadata: Option<DriverMetadata>,
    pub backup_path: Option<PathBuf>,
}

/// Hot swap event notification
#[derive(Debug, Clone)]
pub enum HotSwapEvent {
    OperationStarted {
        operation_id: String,
        operation: HotSwapOperation,
    },
    OperationProgress {
        operation_id: String,
        progress: f32,
        message: String,
    },
    OperationCompleted {
        operation_id: String,
        success: bool,
        duration: Duration,
    },
    OperationFailed {
        operation_id: String,
        error: String,
        rollback_attempted: bool,
    },
}

/// Hot swap manager
pub struct HotSwapManager {
    /// Dynamic driver registry
    registry: Arc<DynamicDriverRegistry>,
    
    /// Dynamic loader
    loader: Arc<DynamicLoader>,
    
    /// Active operations
    operations: Arc<RwLock<HashMap<String, HotSwapContext>>>,
    
    /// Event sender for notifications
    event_sender: Arc<Mutex<Option<mpsc::UnboundedSender<HotSwapEvent>>>>,
    
    /// Operation queue
    operation_queue: Arc<Mutex<Vec<(String, HotSwapOperation, oneshot::Sender<Result<()>>)>>>,
    
    /// Whether manager is running
    running: Arc<RwLock<bool>>,
}

impl HotSwapManager {
    /// Create new hot swap manager
    pub fn new(registry: DynamicDriverRegistry, loader: DynamicLoader) -> Self {
        Self {
            registry: Arc::new(registry),
            loader: Arc::new(loader),
            operations: Arc::new(RwLock::new(HashMap::new())),
            event_sender: Arc::new(Mutex::new(None)),
            operation_queue: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start the hot swap manager
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.running.write().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        info!("Starting hot swap manager");
        
        // Start processing loop
        let queue = self.operation_queue.clone();
        let operations = self.operations.clone();
        let registry = self.registry.clone();
        let loader = self.loader.clone();
        let event_sender = self.event_sender.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            while *running.read().unwrap() {
                interval.tick().await;
                
                // Process queued operations
                let pending_ops = {
                    let mut queue_guard = queue.lock().unwrap();
                    std::mem::take(&mut *queue_guard)
                };
                
                for (op_id, operation, response_tx) in pending_ops {
                    let result = Self::execute_operation(
                        &op_id,
                        operation,
                        &registry,
                        &loader,
                        &operations,
                        &event_sender,
                    ).await;
                    
                    let _ = response_tx.send(result);
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the hot swap manager
    pub fn stop(&self) {
        let mut running = self.running.write().unwrap();
        *running = false;
        info!("Hot swap manager stopped");
    }
    
    /// Set event sender for notifications
    pub fn set_event_sender(&self, sender: mpsc::UnboundedSender<HotSwapEvent>) {
        let mut event_sender = self.event_sender.lock().unwrap();
        *event_sender = Some(sender);
    }
    
    /// Execute hot swap operation
    pub async fn execute(&self, operation: HotSwapOperation) -> Result<String> {
        let operation_id = uuid::Uuid::new_v4().to_string();
        let (response_tx, response_rx) = oneshot::channel();
        
        // Queue operation
        {
            let mut queue = self.operation_queue.lock().unwrap();
            queue.push((operation_id.clone(), operation, response_tx));
        }
        
        // Wait for completion
        response_rx.await
            .context("Operation response channel closed")?
            .context("Hot swap operation failed")?;
        
        Ok(operation_id)
    }
    
    /// Execute operation implementation
    async fn execute_operation(
        operation_id: &str,
        operation: HotSwapOperation,
        registry: &DynamicDriverRegistry,
        loader: &DynamicLoader,
        operations: &Arc<RwLock<HashMap<String, HotSwapContext>>>,
        event_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<HotSwapEvent>>>>,
    ) -> Result<()> {
        let mut context = HotSwapContext::new(operation.clone());
        context.status = HotSwapStatus::InProgress;
        
        // Store operation context
        {
            let mut ops = operations.write().unwrap();
            ops.insert(operation_id.to_string(), context.clone());
        }
        
        // Send start event
        Self::send_event(event_sender, HotSwapEvent::OperationStarted {
            operation_id: operation_id.to_string(),
            operation: operation.clone(),
        });
        
        let result = match operation {
            HotSwapOperation::Load { driver_id, library_path } => {
                Self::execute_load(registry, loader, &driver_id, &library_path, event_sender, operation_id).await
            }
            HotSwapOperation::Unload { driver_id } => {
                Self::execute_unload(registry, &driver_id, event_sender, operation_id).await
            }
            HotSwapOperation::Replace { driver_id, old_path, new_path } => {
                Self::execute_replace(registry, loader, &driver_id, &old_path, &new_path, event_sender, operation_id).await
            }
            HotSwapOperation::Reload { driver_id, library_path } => {
                Self::execute_reload(registry, loader, &driver_id, &library_path, event_sender, operation_id).await
            }
        };
        
        // Update operation context
        {
            let mut ops = operations.write().unwrap();
            if let Some(ctx) = ops.get_mut(operation_id) {
                match &result {
                    Ok(()) => {
                        ctx.complete();
                        Self::send_event(event_sender, HotSwapEvent::OperationCompleted {
                            operation_id: operation_id.to_string(),
                            success: true,
                            duration: ctx.duration(),
                        });
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        ctx.fail(error_msg.clone());
                        Self::send_event(event_sender, HotSwapEvent::OperationFailed {
                            operation_id: operation_id.to_string(),
                            error: error_msg,
                            rollback_attempted: false,
                        });
                    }
                }
            }
        }
        
        result
    }
    
    /// Execute load operation
    async fn execute_load(
        registry: &DynamicDriverRegistry,
        loader: &DynamicLoader,
        driver_id: &str,
        library_path: &Path,
        event_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<HotSwapEvent>>>>,
        operation_id: &str,
    ) -> Result<()> {
        info!("Loading driver: {} from {:?}", driver_id, library_path);
        
        Self::send_progress(event_sender, operation_id, 0.1, "Loading library");
        
        // Load the library
        let library = loader.load_driver(library_path)
            .context("Failed to load library")?;
        
        Self::send_progress(event_sender, operation_id, 0.5, "Creating registration");
        
        // Create registration
        let registration = DriverRegistration::new(
            driver_id.to_string(),
            library_path,
            library.metadata().clone(),
        );
        
        Self::send_progress(event_sender, operation_id, 0.8, "Registering driver");
        
        // Register driver
        registry.register_driver(registration)
            .context("Failed to register driver")?;
        
        Self::send_progress(event_sender, operation_id, 1.0, "Load completed");
        
        info!("Successfully loaded driver: {}", driver_id);
        Ok(())
    }
    
    /// Execute unload operation
    async fn execute_unload(
        registry: &DynamicDriverRegistry,
        driver_id: &str,
        event_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<HotSwapEvent>>>>,
        operation_id: &str,
    ) -> Result<()> {
        info!("Unloading driver: {}", driver_id);
        
        Self::send_progress(event_sender, operation_id, 0.5, "Unloading driver");
        
        // Unload driver
        registry.unload_driver(driver_id)
            .context("Failed to unload driver")?;
        
        Self::send_progress(event_sender, operation_id, 0.8, "Unregistering driver");
        
        // Unregister driver
        registry.unregister_driver(driver_id)
            .context("Failed to unregister driver")?;
        
        Self::send_progress(event_sender, operation_id, 1.0, "Unload completed");
        
        info!("Successfully unloaded driver: {}", driver_id);
        Ok(())
    }
    
    /// Execute replace operation
    async fn execute_replace(
        registry: &DynamicDriverRegistry,
        loader: &DynamicLoader,
        driver_id: &str,
        old_path: &Path,
        new_path: &Path,
        event_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<HotSwapEvent>>>>,
        operation_id: &str,
    ) -> Result<()> {
        info!("Replacing driver: {} from {:?} to {:?}", driver_id, old_path, new_path);
        
        Self::send_progress(event_sender, operation_id, 0.2, "Unloading old driver");
        
        // Unload old driver
        registry.unload_driver(driver_id)
            .context("Failed to unload old driver")?;
        
        Self::send_progress(event_sender, operation_id, 0.5, "Loading new driver");
        
        // Load new library
        let library = loader.load_driver(new_path)
            .context("Failed to load new library")?;
        
        Self::send_progress(event_sender, operation_id, 0.8, "Updating registration");
        
        // Update registration
        let registration = DriverRegistration::new(
            driver_id.to_string(),
            new_path,
            library.metadata().clone(),
        );
        
        // Unregister old and register new
        let _ = registry.unregister_driver(driver_id); // Ignore error if not registered
        registry.register_driver(registration)
            .context("Failed to register new driver")?;
        
        Self::send_progress(event_sender, operation_id, 1.0, "Replace completed");
        
        info!("Successfully replaced driver: {}", driver_id);
        Ok(())
    }
    
    /// Execute reload operation
    async fn execute_reload(
        registry: &DynamicDriverRegistry,
        loader: &DynamicLoader,
        driver_id: &str,
        library_path: &Path,
        event_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<HotSwapEvent>>>>,
        operation_id: &str,
    ) -> Result<()> {
        info!("Reloading driver: {} from {:?}", driver_id, library_path);
        
        Self::send_progress(event_sender, operation_id, 0.3, "Unloading current driver");
        
        // Unload current version
        registry.unload_driver(driver_id)
            .context("Failed to unload current driver")?;
        
        Self::send_progress(event_sender, operation_id, 0.6, "Clearing loader cache");
        
        // Clear from loader cache
        loader.unload_driver(library_path)
            .context("Failed to clear loader cache")?;
        
        Self::send_progress(event_sender, operation_id, 0.8, "Reloading driver");
        
        // Load again
        registry.load_driver(driver_id)
            .context("Failed to reload driver")?;
        
        Self::send_progress(event_sender, operation_id, 1.0, "Reload completed");
        
        info!("Successfully reloaded driver: {}", driver_id);
        Ok(())
    }
    
    /// Send progress event
    fn send_progress(
        event_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<HotSwapEvent>>>>,
        operation_id: &str,
        progress: f32,
        message: &str,
    ) {
        Self::send_event(event_sender, HotSwapEvent::OperationProgress {
            operation_id: operation_id.to_string(),
            progress,
            message: message.to_string(),
        });
    }
    
    /// Send event notification
    fn send_event(
        event_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<HotSwapEvent>>>>,
        event: HotSwapEvent,
    ) {
        if let Some(sender) = event_sender.lock().unwrap().as_ref() {
            if sender.send(event).is_err() {
                debug!("Hot swap event receiver dropped");
            }
        }
    }
    
    /// Get operation status
    pub fn get_operation_status(&self, operation_id: &str) -> Option<HotSwapContext> {
        let operations = self.operations.read().unwrap();
        operations.get(operation_id).cloned()
    }
    
    /// Get all operations
    pub fn get_all_operations(&self) -> Vec<(String, HotSwapContext)> {
        let operations = self.operations.read().unwrap();
        operations.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
    
    /// Clear completed operations
    pub fn clear_completed_operations(&self) {
        let mut operations = self.operations.write().unwrap();
        operations.retain(|_, ctx| {
            !matches!(ctx.status, HotSwapStatus::Completed | HotSwapStatus::Failed(_))
        });
    }
    
    /// Cancel operation (if still pending)
    pub fn cancel_operation(&self, operation_id: &str) -> bool {
        let mut operations = self.operations.write().unwrap();
        if let Some(ctx) = operations.get_mut(operation_id) {
            if ctx.status == HotSwapStatus::Pending {
                ctx.status = HotSwapStatus::Failed("Cancelled".to_string());
                ctx.completed_at = Some(Instant::now());
                return true;
            }
        }
        false
    }
}

impl Default for HotSwapManager {
    fn default() -> Self {
        let registry = DynamicDriverRegistry::new().expect("Failed to create registry");
        let loader = DynamicLoader::new().expect("Failed to create loader");
        Self::new(registry, loader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hot_swap_context() {
        let operation = HotSwapOperation::Load {
            driver_id: "test".to_string(),
            library_path: PathBuf::from("/test.so"),
        };
        
        let mut context = HotSwapContext::new(operation);
        assert_eq!(context.status, HotSwapStatus::Pending);
        assert!(context.can_retry());
        
        // 添加小延迟确保时间差可测量
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        context.complete();
        assert_eq!(context.status, HotSwapStatus::Completed);
        assert!(context.completed_at.is_some());
        
        let duration = context.duration();
        assert!(duration.as_nanos() > 0); // 使用纳秒而不是毫秒
    }
    
    #[test]
    fn test_hot_swap_operations() {
        let load_op = HotSwapOperation::Load {
            driver_id: "test".to_string(),
            library_path: PathBuf::from("/test.so"),
        };
        
        let unload_op = HotSwapOperation::Unload {
            driver_id: "test".to_string(),
        };
        
        let replace_op = HotSwapOperation::Replace {
            driver_id: "test".to_string(),
            old_path: PathBuf::from("/old.so"),
            new_path: PathBuf::from("/new.so"),
        };
        
        let reload_op = HotSwapOperation::Reload {
            driver_id: "test".to_string(),
            library_path: PathBuf::from("/test.so"),
        };
        
        // Operations should be different
        assert_ne!(load_op, unload_op);
        assert_ne!(replace_op, reload_op);
    }
    
    #[tokio::test]
    async fn test_hot_swap_manager_creation() {
        let manager = HotSwapManager::default();
        
        // Should be able to start and stop
        assert!(manager.start().await.is_ok());
        manager.stop();
        
        // Should be able to start again
        assert!(manager.start().await.is_ok());
        manager.stop();
    }
    
    #[test]
    fn test_rollback_info() {
        let rollback = RollbackInfo {
            previous_library: Some(PathBuf::from("/old.so")),
            previous_metadata: None,
            backup_path: Some(PathBuf::from("/backup.so")),
        };
        
        assert!(rollback.previous_library.is_some());
        assert!(rollback.previous_metadata.is_none());
        assert!(rollback.backup_path.is_some());
    }
    
    #[test]
    fn test_hot_swap_events() {
        let event = HotSwapEvent::OperationStarted {
            operation_id: "test-123".to_string(),
            operation: HotSwapOperation::Load {
                driver_id: "test".to_string(),
                library_path: PathBuf::from("/test.so"),
            },
        };
        
        match event {
            HotSwapEvent::OperationStarted { operation_id, .. } => {
                assert_eq!(operation_id, "test-123");
            }
            _ => panic!("Wrong event type"),
        }
    }
}