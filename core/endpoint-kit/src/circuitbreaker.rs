/*!
# Circuit Breaker Implementation

三态熔断器实现 (Closed/Open/Half-Open)，用于连接池故障保护。
*/

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// 熔断器状态
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    /// 关闭状态：正常工作
    Closed,
    /// 打开状态：拒绝所有请求
    Open,
    /// 半开状态：允许少量请求测试服务恢复
    HalfOpen,
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 故障阈值：连续失败次数触发熔断
    pub failure_threshold: u32,
    /// 故障率阈值：失败率超过此值触发熔断
    pub failure_rate_threshold: f64,
    /// 最小请求数：计算失败率的最小请求数
    pub min_request_threshold: u32,
    /// 熔断器打开时间：熔断后等待时间
    pub timeout: Duration,
    /// 半开状态最大请求数
    pub max_half_open_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_rate_threshold: 0.5,
            min_request_threshold: 20,
            timeout: Duration::from_secs(60),
            max_half_open_requests: 3,
        }
    }
}

/// 熔断器统计信息
#[derive(Debug, Default)]
struct CircuitBreakerStats {
    /// 总请求数
    total_requests: AtomicU64,
    /// 失败请求数
    failed_requests: AtomicU64,
    /// 连续失败次数
    consecutive_failures: AtomicU32,
    /// 半开状态请求数
    half_open_requests: AtomicU32,
}

/// 熔断器内部状态
#[derive(Debug)]
struct CircuitBreakerInner {
    state: CircuitBreakerState,
    last_failure_time: Option<Instant>,
    config: CircuitBreakerConfig,
}

/// 三态熔断器
#[derive(Debug)]
pub struct CircuitBreaker {
    inner: Arc<Mutex<CircuitBreakerInner>>,
    stats: CircuitBreakerStats,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CircuitBreakerInner {
                state: CircuitBreakerState::Closed,
                last_failure_time: None,
                config,
            })),
            stats: CircuitBreakerStats::default(),
        }
    }

    /// 使用默认配置创建熔断器
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// 检查是否可以执行请求
    pub fn can_request(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        
        match inner.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // 检查是否到了尝试恢复的时间
                if let Some(last_failure) = inner.last_failure_time {
                    if last_failure.elapsed() >= inner.config.timeout {
                        inner.state = CircuitBreakerState::HalfOpen;
                        self.stats.half_open_requests.store(0, Ordering::Relaxed);
                        tracing::info!("Circuit breaker transitioning to half-open state");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                let current_half_open = self.stats.half_open_requests.load(Ordering::Relaxed);
                current_half_open < inner.config.max_half_open_requests
            }
        }
    }

    /// 记录成功请求
    pub fn record_success(&self) {
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);
        self.stats.consecutive_failures.store(0, Ordering::Relaxed);

        let mut inner = self.inner.lock().unwrap();
        
        if inner.state == CircuitBreakerState::HalfOpen {
            let half_open_requests = self.stats.half_open_requests.fetch_add(1, Ordering::Relaxed) + 1;
            
            // 如果半开状态测试成功，转为关闭状态
            if half_open_requests >= inner.config.max_half_open_requests {
                inner.state = CircuitBreakerState::Closed;
                inner.last_failure_time = None;
                tracing::info!("Circuit breaker recovered, transitioning to closed state");
            }
        }
    }

    /// 记录失败请求
    pub fn record_failure(&self) {
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);
        self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
        let consecutive_failures = self.stats.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

        let mut inner = self.inner.lock().unwrap();
        inner.last_failure_time = Some(Instant::now());

        match inner.state {
            CircuitBreakerState::Closed => {
                if self.should_trip(&inner, consecutive_failures) {
                    inner.state = CircuitBreakerState::Open;
                    tracing::warn!("Circuit breaker tripped, transitioning to open state");
                }
            }
            CircuitBreakerState::HalfOpen => {
                // 半开状态下的失败直接转为打开状态
                inner.state = CircuitBreakerState::Open;
                tracing::warn!("Circuit breaker failed in half-open state, transitioning back to open");
            }
            CircuitBreakerState::Open => {
                // 保持打开状态
            }
        }
    }

    /// 判断是否应该触发熔断
    fn should_trip(&self, inner: &CircuitBreakerInner, consecutive_failures: u32) -> bool {
        // 检查连续失败次数
        if consecutive_failures >= inner.config.failure_threshold {
            return true;
        }

        // 检查失败率
        let total = self.stats.total_requests.load(Ordering::Relaxed);
        if total >= inner.config.min_request_threshold as u64 {
            let failed = self.stats.failed_requests.load(Ordering::Relaxed);
            let failure_rate = failed as f64 / total as f64;
            
            if failure_rate >= inner.config.failure_rate_threshold {
                return true;
            }
        }

        false
    }

    /// 获取当前状态
    pub fn state(&self) -> CircuitBreakerState {
        self.inner.lock().unwrap().state.clone()
    }

    /// 获取统计信息
    pub fn stats(&self) -> CircuitBreakerMetrics {
        let inner = self.inner.lock().unwrap();
        CircuitBreakerMetrics {
            state: inner.state.clone(),
            total_requests: self.stats.total_requests.load(Ordering::Relaxed),
            failed_requests: self.stats.failed_requests.load(Ordering::Relaxed),
            consecutive_failures: self.stats.consecutive_failures.load(Ordering::Relaxed),
            failure_rate: {
                let total = self.stats.total_requests.load(Ordering::Relaxed);
                if total > 0 {
                    self.stats.failed_requests.load(Ordering::Relaxed) as f64 / total as f64
                } else {
                    0.0
                }
            },
        }
    }

    /// 手动重置熔断器
    pub fn reset(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.state = CircuitBreakerState::Closed;
        inner.last_failure_time = None;
        
        self.stats.total_requests.store(0, Ordering::Relaxed);
        self.stats.failed_requests.store(0, Ordering::Relaxed);
        self.stats.consecutive_failures.store(0, Ordering::Relaxed);
        self.stats.half_open_requests.store(0, Ordering::Relaxed);
        
        tracing::info!("Circuit breaker manually reset");
    }
}

/// 熔断器指标
#[derive(Debug)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitBreakerState,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub consecutive_failures: u32,
    pub failure_rate: f64,
}

/// 熔断器包装器，用于自动记录结果
pub struct CircuitBreakerGuard<'a> {
    circuit_breaker: &'a CircuitBreaker,
    recorded: bool,
}

impl<'a> CircuitBreakerGuard<'a> {
    fn new(circuit_breaker: &'a CircuitBreaker) -> Self {
        Self {
            circuit_breaker,
            recorded: false,
        }
    }

    /// 记录成功
    pub fn success(mut self) {
        self.circuit_breaker.record_success();
        self.recorded = true;
    }

    /// 记录失败
    pub fn failure(mut self) {
        self.circuit_breaker.record_failure();
        self.recorded = true;
    }
}

impl Drop for CircuitBreakerGuard<'_> {
    fn drop(&mut self) {
        if !self.recorded {
            // 默认记录为失败
            self.circuit_breaker.record_failure();
        }
    }
}

impl CircuitBreaker {
    /// 执行带熔断保护的操作
    pub async fn call<F, R, E>(&self, operation: F) -> Result<R, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<R, E>>,
    {
        if !self.can_request() {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        let guard = CircuitBreakerGuard::new(self);
        
        match operation.await {
            Ok(result) => {
                guard.success();
                Ok(result)
            }
            Err(error) => {
                guard.failure();
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }
}

/// 熔断器错误类型
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(E),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let cb = CircuitBreaker::with_defaults();
        
        assert_eq!(cb.state(), CircuitBreakerState::Closed);
        assert!(cb.can_request());
        
        // 记录成功请求
        cb.record_success();
        let metrics = cb.stats();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_millis(100),
            ..CircuitBreakerConfig::default()
        };
        let cb = CircuitBreaker::new(config);
        
        // 触发熔断
        for _ in 0..3 {
            cb.record_failure();
        }
        
        assert_eq!(cb.state(), CircuitBreakerState::Open);
        assert!(!cb.can_request());
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(50),
            max_half_open_requests: 2,
            ..CircuitBreakerConfig::default()
        };
        let cb = CircuitBreaker::new(config);
        
        // 触发熔断
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);
        
        // 等待超时
        sleep(Duration::from_millis(60)).await;
        
        // 现在应该可以请求了（转为半开状态）
        assert!(cb.can_request());
        assert_eq!(cb.state(), CircuitBreakerState::HalfOpen);
        
        // 成功请求应该恢复熔断器
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_call() {
        let cb = CircuitBreaker::with_defaults();
        
        // 成功操作
        let result = cb.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        // 失败操作
        let result = cb.call(async { Err::<i32, &str>("error") }).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CircuitBreakerError::OperationFailed(e) => assert_eq!(e, "error"),
            _ => panic!("Unexpected error type"),
        }
    }
}