//! 驱动监督器 - 错误重启和生命周期管理

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Notify};
use tokio::time::sleep;

use crate::driver::Driver;

/// 驱动监督器
#[derive(Clone)]
pub struct DriverSupervisor {
    driver_id: String,
    driver: Arc<RwLock<Box<dyn Driver>>>,
    shutdown_notify: Arc<Notify>,
    restart_count: Arc<RwLock<u32>>,
}

impl DriverSupervisor {
    pub fn new(driver_id: String, driver: Box<dyn Driver>) -> Self {
        Self {
            driver_id,
            driver: Arc::new(RwLock::new(driver)),
            shutdown_notify: Arc::new(Notify::new()),
            restart_count: Arc::new(RwLock::new(0)),
        }
    }

    /// 运行监督循环
    pub async fn run(&self) {
        let mut backoff_seconds = 1;
        
        loop {
            tokio::select! {
                _ = self.shutdown_notify.notified() => {
                    tracing::info!("Driver supervisor {} shutting down", self.driver_id);
                    break;
                }
                result = self.run_driver() => {
                    match result {
                        Ok(_) => {
                            tracing::info!("Driver {} completed normally", self.driver_id);
                            break;
                        }
                        Err(e) => {
                            let mut count = self.restart_count.write().await;
                            *count += 1;
                            
                            tracing::error!(
                                "Driver {} failed (attempt {}): {}",
                                self.driver_id, *count, e
                            );

                            // 如果重启次数过多，进入故障状态
                            if *count >= 10 {
                                tracing::error!(
                                    "Driver {} exceeded max restart attempts, marking as fault",
                                    self.driver_id
                                );
                                break;
                            }

                            // 指数退避重启
                            sleep(Duration::from_secs(backoff_seconds)).await;
                            backoff_seconds = (backoff_seconds * 2).min(60);
                        }
                    }
                }
            }
        }
    }

    /// 运行单个驱动实例（优化版本，使用批量发布）
    async fn run_driver(&self) -> anyhow::Result<()> {
        // 获取frame-bus sender
        let frame_tx = frame_bus::ring::get_publisher()?.clone();
        
        // 创建一个测试用的endpoint handle
        let endpoint_handle = endpoint_kit::from_url("tcp://localhost:502").await
            .map_err(|e| anyhow::anyhow!("Failed to create endpoint handle: {}", e))?;
        
        // 连接驱动
        {
            let mut driver = self.driver.write().await;
            driver.connect(endpoint_handle).await?;
        }
        
        // 运行读取循环
        let mut driver = self.driver.write().await;
        driver.read_loop(frame_tx).await?;
        
        Ok(())
    }

    /// 关闭监督器
    pub async fn shutdown(&self) {
        // 先关闭驱动
        {
            let mut driver = self.driver.write().await;
            if let Err(e) = driver.shutdown().await {
                tracing::error!("Error shutting down driver {}: {}", self.driver_id, e);
            }
        }
        
        // 通知监督循环退出
        self.shutdown_notify.notify_one();
    }

    /// 获取重启次数
    pub async fn restart_count(&self) -> u32 {
        *self.restart_count.read().await
    }

    /// 重置重启计数器
    pub async fn reset_restart_count(&self) {
        let mut count = self.restart_count.write().await;
        *count = 0;
    }
}