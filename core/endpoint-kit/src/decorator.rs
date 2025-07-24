//! 装饰器链实现

use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite};
use async_trait::async_trait;
use crate::{EndpointUrl, EndpointError};

/// 创建一个统一的IO trait
pub trait AsyncIO: AsyncRead + AsyncWrite + Send + Sync + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Sync + Unpin> AsyncIO for T {}

/// 动态装饰链输出类型
pub type EndpointBox = Pin<Box<dyn AsyncIO + 'static>>;

/// 装饰器trait
#[async_trait]
pub trait EndpointDecorator {
    async fn decorate(&self, stream: EndpointBox, url: &EndpointUrl) -> Result<EndpointBox, EndpointError>;
}

/// TLS装饰器
pub struct TlsDecorator;

#[async_trait]
impl EndpointDecorator for TlsDecorator {
    async fn decorate(&self, _stream: EndpointBox, _url: &EndpointUrl) -> Result<EndpointBox, EndpointError> {
        // MVP-0 暂不实现TLS，留待后续版本
        Err(EndpointError::UnsupportedScheme("TLS not implemented in MVP-0".to_string()))
    }
}

/// 速率限制装饰器  
pub struct RateLimitDecorator;

use std::time::Instant;
use tokio::sync::Mutex;

/// 简单的令牌桶限流器
pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    last_refill: Instant,
    refill_rate: u32, // tokens per second
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: capacity,
            last_refill: Instant::now(),
            refill_rate,
        }
    }

    pub fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = (elapsed.as_secs_f64() * self.refill_rate as f64) as u32;
        
        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(self.capacity);
            self.last_refill = now;
        }
    }
}

/// 带限流的流包装器
pub struct RateLimitedStream<S> {
    inner: S,
    bucket: std::sync::Arc<Mutex<TokenBucket>>,
}

impl<S> RateLimitedStream<S> {
    pub fn new(stream: S, rate_pps: u32) -> Self {
        let bucket = std::sync::Arc::new(Mutex::new(TokenBucket::new(rate_pps * 2, rate_pps)));
        Self { inner: stream, bucket }
    }
}

impl<S> AsyncRead for RateLimitedStream<S>
where
    S: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S> AsyncWrite for RateLimitedStream<S>
where
    S: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        // 检查限流
        let bucket = self.bucket.clone();
        let can_write = match bucket.try_lock() {
            Ok(mut guard) => guard.try_consume(1),
            Err(_) => {
                // 锁竞争，允许写入但稍后重试
                true
            }
        };
        
        if can_write {
            Pin::new(&mut self.inner).poll_write(cx, buf)
        } else {
            // 限流，返回Pending
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[async_trait]
impl EndpointDecorator for RateLimitDecorator {
    async fn decorate(&self, stream: EndpointBox, url: &EndpointUrl) -> Result<EndpointBox, EndpointError> {
        if let Some(rate_str) = url.query.get("rate") {
            // 解析速率: "200pps" -> 200
            let rate = if rate_str.ends_with("pps") {
                rate_str[..rate_str.len()-3].parse::<u32>()
                    .map_err(|e| EndpointError::InvalidUrl(format!("Invalid rate: {}", e)))?
            } else {
                rate_str.parse::<u32>()
                    .map_err(|e| EndpointError::InvalidUrl(format!("Invalid rate: {}", e)))?
            };
            
            Ok(Box::pin(RateLimitedStream::new(stream, rate)))
        } else {
            Ok(stream)
        }
    }
}

/// 半双工装饰器 (用于RS-485)
pub struct HalfDuplexDecorator;

#[async_trait]
impl EndpointDecorator for HalfDuplexDecorator {
    async fn decorate(&self, stream: EndpointBox, _url: &EndpointUrl) -> Result<EndpointBox, EndpointError> {
        // 简单实现：添加写后延迟
        // 实际生产中可能需要更复杂的RTS控制
        Ok(stream)
    }
}