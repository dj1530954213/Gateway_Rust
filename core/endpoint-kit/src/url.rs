//! URL解析和标准化
//! 
//! 支持多层scheme堆叠，如 tls+tcp://host:port

use std::collections::{HashMap, BTreeMap};
use std::fmt;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::error::EndpointError;

/// 支持的协议scheme
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    Tcp,
    Udp,
    Serial,
    Tls,
    Dtls,
    Quic,
    Tsn,
    Prp,
    Can,
    Bluetooth,
    Http,
    Https,
    Modbus,
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Scheme {
    /// 获取字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Scheme::Tcp => "tcp",
            Scheme::Udp => "udp", 
            Scheme::Serial => "serial",
            Scheme::Tls => "tls",
            Scheme::Dtls => "dtls",
            Scheme::Quic => "quic",
            Scheme::Tsn => "tsn",
            Scheme::Prp => "prp",
            Scheme::Can => "can",
            Scheme::Bluetooth => "bluetooth",
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::Modbus => "modbus",
        }
    }
}

impl std::str::FromStr for Scheme {
    type Err = EndpointError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcp" => Ok(Scheme::Tcp),
            "udp" => Ok(Scheme::Udp),
            "serial" => Ok(Scheme::Serial),
            "tls" => Ok(Scheme::Tls),
            "dtls" => Ok(Scheme::Dtls),
            "quic" => Ok(Scheme::Quic),
            "tsn" => Ok(Scheme::Tsn),
            "prp" => Ok(Scheme::Prp),
            "can" => Ok(Scheme::Can),
            "bluetooth" => Ok(Scheme::Bluetooth),
            "http" => Ok(Scheme::Http),
            "https" => Ok(Scheme::Https),
            "modbus" => Ok(Scheme::Modbus),
            _ => Err(EndpointError::UnsupportedScheme(s.to_string())),
        }
    }
}

/// 解析后的端点URL
#[derive(Debug, Clone, PartialEq)]
pub struct EndpointUrl {
    pub scheme_stack: Vec<Scheme>,
    pub host: String,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub query: HashMap<String, String>,
}

/// 标准化的URL (用作池键)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedUrl {
    pub scheme_stack: Vec<Scheme>,
    pub host: String,
    pub port: Option<u16>,
    pub query: BTreeMap<String, String>, // 排序保证稳定
}

impl EndpointUrl {
    /// 解析URL字符串
    pub fn parse(url_str: &str) -> Result<Self, EndpointError> {
        // 处理多层scheme: tls+tcp://...
        let (scheme_part, rest) = if let Some(pos) = url_str.find("://") {
            (&url_str[..pos], &url_str[pos + 3..])
        } else {
            return Err(EndpointError::InvalidUrl("Missing ://".to_string()));
        };

        // 解析scheme堆栈
        let scheme_stack: Result<Vec<Scheme>, _> = scheme_part
            .split('+')
            .map(|s| s.parse())
            .collect();
        let scheme_stack = scheme_stack?;

        if scheme_stack.is_empty() {
            return Err(EndpointError::InvalidUrl("Empty scheme".to_string()));
        }

        // 构造标准URL进行解析 (使用最后一个scheme)
        let base_scheme = &scheme_stack[scheme_stack.len() - 1];
        let standard_url = format!("{}://{}", base_scheme, rest);
        
        let url = Url::parse(&standard_url)
            .map_err(|e| EndpointError::InvalidUrl(format!("URL parse error: {}", e)))?;

        let host = match url.host_str() {
            Some(h) => {
                // 处理IPv6地址，移除方括号
                if h.starts_with('[') && h.ends_with(']') {
                    h[1..h.len()-1].to_string()
                } else {
                    h.to_string()
                }
            },
            None => {
                // 对于serial://设备路径的特殊处理
                if *base_scheme == Scheme::Serial {
                    url.path().to_string()
                } else {
                    return Err(EndpointError::InvalidUrl("Missing host".to_string()));
                }
            }
        };

        let port = url.port();
        let username = if url.username().is_empty() {
            None
        } else {
            Some(url.username().to_string())
        };
        let password = url.password().map(|p| p.to_string());

        // 解析查询参数
        let mut query = HashMap::new();
        for (key, value) in url.query_pairs() {
            query.insert(key.into_owned(), value.into_owned());
        }

        // 获取路径
        let path = if url.path().is_empty() || url.path() == "/" {
            None
        } else {
            Some(url.path().to_string())
        };

        Ok(EndpointUrl {
            scheme_stack,
            host,
            port,
            path,
            username,
            password,
            query,
        })
    }

    /// 转换为标准化URL (去除凭证，用于池键)
    pub fn normalize(&self) -> NormalizedUrl {
        let mut sorted_schemes = self.scheme_stack.clone();
        // 按功能强度排序：基础传输 < 加密 < 网络增强
        sorted_schemes.sort_by_key(|s| match s {
            Scheme::Serial | Scheme::Can | Scheme::Bluetooth => 0,
            Scheme::Tcp | Scheme::Udp | Scheme::Http => 1,
            Scheme::Tls | Scheme::Dtls | Scheme::Quic | Scheme::Https => 2,
            Scheme::Tsn | Scheme::Prp => 3,
            Scheme::Modbus => 1,
        });

        let query: BTreeMap<String, String> = self.query.iter()
            .filter(|(k, _)| !matches!(k.as_str(), "username" | "password")) // 过滤敏感信息
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        NormalizedUrl {
            scheme_stack: sorted_schemes,
            host: self.host.to_lowercase(),
            port: self.port,
            query,
        }
    }

    /// 获取默认端口
    pub fn default_port(&self) -> Option<u16> {
        match self.scheme_stack.last()? {
            Scheme::Tcp => Some(502), // Modbus default
            Scheme::Tls => Some(502),
            Scheme::Udp => Some(502),
            Scheme::Dtls => Some(5060),
            Scheme::Quic => Some(443),
            Scheme::Http => Some(80),
            Scheme::Https => Some(443),
            Scheme::Modbus => Some(502),
            _ => None,
        }
    }

    /// 获取实际监听端口
    pub fn effective_port(&self) -> Option<u16> {
        self.port.or_else(|| self.default_port())
    }

    /// 获取socket地址 (仅TCP/UDP)
    pub fn socket_addr(&self) -> Result<std::net::SocketAddr, EndpointError> {
        let port = self.effective_port()
            .ok_or_else(|| EndpointError::InvalidUrl("No port specified".to_string()))?;
        
        let addr = format!("{}:{}", self.host, port);
        addr.parse()
            .map_err(|e| EndpointError::InvalidUrl(format!("Invalid socket address: {}", e)))
    }

    /// 获取主scheme
    pub fn scheme(&self) -> &str {
        self.scheme_stack.last()
            .map(|s| s.as_str())
            .unwrap_or("unknown")
    }

    /// 获取主机名
    pub fn host(&self) -> &str {
        &self.host
    }

    /// 获取端口
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// 获取路径
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// 获取所有schemes
    pub fn schemes(&self) -> &[Scheme] {
        &self.scheme_stack
    }

    /// 获取查询参数
    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query
    }
}

impl From<EndpointUrl> for NormalizedUrl {
    fn from(url: EndpointUrl) -> Self {
        url.normalize()
    }
}

impl fmt::Display for EndpointUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 重构scheme
        let scheme_str = self.scheme_stack.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("+");

        write!(f, "{}://{}", scheme_str, self.host)?;
        
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }

        if !self.query.is_empty() {
            write!(f, "?")?;
            let mut first = true;
            for (key, value) in &self.query {
                if !first {
                    write!(f, "&")?;
                }
                write!(f, "{}={}", key, value)?;
                first = false;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tcp_url() {
        let url = EndpointUrl::parse("tcp://[REAL_TEST_IP]:502").unwrap();
        assert_eq!(url.scheme_stack, vec![Scheme::Tcp]);
        assert_eq!(url.host, "[REAL_TEST_IP]");
        assert_eq!(url.port, Some(502));
    }

    #[test]
    fn test_stacked_scheme() {
        let url = EndpointUrl::parse("tls+tcp://plc.local:502?timeout=1s").unwrap();
        assert_eq!(url.scheme_stack, vec![Scheme::Tls, Scheme::Tcp]);
        assert_eq!(url.host, "plc.local");
        assert_eq!(url.query.get("timeout"), Some(&"1s".to_string()));
    }

    #[test]
    fn test_serial_device() {
        let url = EndpointUrl::parse("serial:///dev/ttyUSB0?baud=9600").unwrap();
        assert_eq!(url.scheme_stack, vec![Scheme::Serial]);
        assert_eq!(url.host, "/dev/ttyUSB0");
        assert_eq!(url.query.get("baud"), Some(&"9600".to_string()));
    }

    #[test]
    fn test_normalize() {
        let url = EndpointUrl::parse("tls+tcp://user:pass@host:502?key=val").unwrap();
        let norm = url.normalize();
        
        assert_eq!(norm.host, "host");
        assert_eq!(norm.port, Some(502));
        assert!(norm.query.contains_key("key"));
        // 凭证应被过滤
        assert!(!norm.query.contains_key("username"));
        assert!(!norm.query.contains_key("password"));
    }
}