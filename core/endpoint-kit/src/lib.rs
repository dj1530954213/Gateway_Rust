//! EndpointKit - L0 统一连接抽象层
//! 
//! 提供统一的URL-based连接管理，支持装饰器链、连接池、熔断器和背压控制

pub mod url;
pub mod pool;
pub mod decorator;
pub mod metrics;
pub mod error;
pub mod control;
pub mod circuitbreaker;
pub mod backpressure;

pub use url::{EndpointUrl, NormalizedUrl, Scheme};
pub use pool::{EndpointHandle, EndpointFactory, get_factory};
pub use decorator::EndpointBox;
pub use error::EndpointError;
pub use control::{ControlMsg, send_pause, send_resume, send_drain};
pub use circuitbreaker::{CircuitBreaker, CircuitBreakerState, CircuitBreakerConfig, CircuitBreakerError};
pub use backpressure::{BackpressureManager, BackpressureSignal, BackpressureConfig, QueueStatus};

pub type Result<T> = std::result::Result<T, EndpointError>;

/// 初始化EndpointKit全局状态
pub fn init() -> Result<()> {
    metrics::init_metrics();
    control::init_control_channel();
    Ok(())
}

/// 便捷函数：从URL字符串创建端点句柄
pub async fn from_url(url: &str) -> Result<std::sync::Arc<EndpointHandle>> {
    get_factory().from_url(url).await
}