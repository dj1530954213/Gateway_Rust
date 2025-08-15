//! 连接池管理

use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use bb8::{Pool, PooledConnection};
use async_trait::async_trait;
use dashmap::DashMap;

use crate::{
    EndpointUrl, NormalizedUrl, Scheme, EndpointBox, EndpointError,
    decorator::{EndpointDecorator, TlsDecorator, RateLimitDecorator, HalfDuplexDecorator},
    control::{subscribe_control, ControlMsg},
    metrics::METRICS,
    circuitbreaker::{CircuitBreaker, CircuitBreakerConfig},
};

/// 连接制造器
pub struct ConnMaker {
    url: EndpointUrl,
}

#[async_trait]
impl bb8::ManageConnection for ConnMaker {
    type Connection = EndpointBox;
    type Error = EndpointError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        build_connection_stack(&self.url).await
    }

    async fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // 简单实现：假设连接总是有效
        // 实际中可能需要发送心跳包验证
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

/// 端点句柄
pub struct EndpointHandle {
    normalized_url: NormalizedUrl,
    pool: Pool<ConnMaker>,
    paused: Arc<RwLock<bool>>,
    circuit_breaker: CircuitBreaker,
}

impl EndpointHandle {
    /// 获取主机名
    pub fn host(&self) -> &str {
        &self.normalized_url.host
    }

    /// 获取端口
    pub fn port(&self) -> Option<u16> {
        self.normalized_url.port
    }

    /// 获取连接
    pub async fn acquire(&self) -> Result<PooledConnection<'_, ConnMaker>, EndpointError> {
        let start = Instant::now();
        
        // 检查熔断器状态
        if !self.circuit_breaker.can_request() {
            return Err(EndpointError::Pool("Circuit breaker is open".to_string()));
        }
        
        // 检查是否被暂停
        if *self.paused.read().await {
            return Err(EndpointError::Paused);
        }

        // 执行连接获取操作，由熔断器保护
        let result = self.circuit_breaker.call(async {
            self.pool.get().await
                .map_err(|e| EndpointError::Pool(format!("Failed to acquire connection: {}", e)))
        }).await;

        let conn = match result {
            Ok(conn) => conn,
            Err(crate::circuitbreaker::CircuitBreakerError::CircuitOpen) => {
                return Err(EndpointError::Pool("Circuit breaker is open".to_string()));
            }
            Err(crate::circuitbreaker::CircuitBreakerError::OperationFailed(e)) => {
                return Err(e);
            }
        };

        // 记录指标
        let elapsed = start.elapsed();
        METRICS.acquire_latency.observe(elapsed.as_micros() as f64);
        METRICS.pool_size.set(self.pool.state().connections as i64);
        
        // Hot Acquire性能要求检查 (≤1µs)
        if elapsed.as_micros() > 1 {
            tracing::warn!("Hot acquire exceeded 1µs threshold: {}µs", elapsed.as_micros());
        }

        Ok(conn)
    }
}

/// 端点工厂
pub struct EndpointFactory {
    pools: DashMap<NormalizedUrl, Arc<EndpointHandle>>,
}

impl Default for EndpointFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl EndpointFactory {
    pub fn new() -> Self {
        Self {
            pools: DashMap::new(),
        }
    }

    /// 从URL创建端点句柄
    pub async fn from_url(&self, url_str: &str) -> Result<Arc<EndpointHandle>, EndpointError> {
        let url = EndpointUrl::parse(url_str)?;
        let normalized = url.normalize();
        
        // 检查缓存
        if let Some(handle) = self.pools.get(&normalized) {
            return Ok(handle.clone());
        }

        // 创建新的连接池
        let manager = ConnMaker { url };
        let pool = Pool::builder()
            .max_size(4) // MVP-3要求：默认连接池大小为4
            .build(manager)
            .await
            .map_err(|e| EndpointError::Pool(format!("Failed to create pool: {}", e)))?;

        // 创建熔断器
        let circuit_breaker_config = CircuitBreakerConfig {
            failure_threshold: 3,
            failure_rate_threshold: 0.5,
            min_request_threshold: 10,
            timeout: std::time::Duration::from_secs(30),
            max_half_open_requests: 2,
        };
        let circuit_breaker = CircuitBreaker::new(circuit_breaker_config);

        let handle = Arc::new(EndpointHandle {
            normalized_url: normalized.clone(),
            pool,
            paused: Arc::new(RwLock::new(false)),
            circuit_breaker,
        });

        // 启动控制消息监听
        let handle_clone = handle.clone();
        let normalized_clone = normalized.clone();
        tokio::spawn(async move {
            if let Ok(mut rx) = subscribe_control() {
                while let Ok(msg) = rx.recv().await {
                    match msg {
                        ControlMsg::Pause(url) if url == normalized_clone => {
                            *handle_clone.paused.write().await = true;
                            METRICS.pause_total.inc();
                        }
                        ControlMsg::Resume(url) if url == normalized_clone => {
                            *handle_clone.paused.write().await = false;
                        }
                        ControlMsg::Drain(url) if url == normalized_clone => {
                            // 关闭所有连接，强制重建
                            // bb8不直接支持drain，这里可以考虑重建池
                            tracing::info!("Drain signal received for {:?}", url);
                        }
                        _ => {}
                    }
                }
            }
        });

        self.pools.insert(normalized, handle.clone());
        Ok(handle)
    }
}

/// 构建连接装饰器链
async fn build_connection_stack(url: &EndpointUrl) -> Result<EndpointBox, EndpointError> {
    // 1. 创建基础传输层
    let mut stream: EndpointBox = match url.scheme_stack.last().unwrap() {
        Scheme::Serial => {
            // 串口连接 - MVP-0暂不实现
            return Err(EndpointError::UnsupportedScheme("Serial not implemented in MVP-0".to_string()));
        }
        Scheme::Tcp | Scheme::Tls | Scheme::Tsn | Scheme::Prp | Scheme::Quic => {
            // TCP连接
            let addr = url.socket_addr()?;
            let tcp_stream = TcpStream::connect(addr).await
                .map_err(EndpointError::Io)?;
            Box::pin(tcp_stream)
        }
        Scheme::Udp | Scheme::Dtls => {
            return Err(EndpointError::UnsupportedScheme("UDP not yet implemented".to_string()));
        }
        other => {
            return Err(EndpointError::UnsupportedScheme(format!("Scheme {:?} not implemented", other)));
        }
    };

    // 2. 应用装饰器链 (按scheme顺序)
    for scheme in &url.scheme_stack {
        stream = match scheme {
            Scheme::Tls => {
                TlsDecorator.decorate(stream, url).await?
            }
            Scheme::Tcp | Scheme::Serial | Scheme::Udp => {
                // 基础传输层，无需装饰
                stream
            }
            _ => {
                // 其他高级协议暂未实现
                stream
            }
        };
    }

    // 3. 应用查询参数装饰器
    if url.query.contains_key("rate") {
        stream = RateLimitDecorator.decorate(stream, url).await?;
    }

    if url.query.get("halfduplex") == Some(&"1".to_string()) {
        stream = HalfDuplexDecorator.decorate(stream, url).await?;
    }

    Ok(stream)
}

/// 全局工厂实例
static FACTORY: once_cell::sync::OnceCell<EndpointFactory> = once_cell::sync::OnceCell::new();

/// 获取全局工厂
pub fn get_factory() -> &'static EndpointFactory {
    FACTORY.get_or_init(EndpointFactory::new)
}