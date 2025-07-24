//! EndpointKit URL解析测试

use endpoint_kit::{EndpointUrl, NormalizedUrl};

#[test]
fn test_simple_tcp_url_parsing() {
    let url = "tcp://192.168.1.100:502";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse URL");
    
    assert_eq!(parsed.scheme(), "tcp");
    assert_eq!(parsed.host(), "192.168.1.100");
    assert_eq!(parsed.port(), Some(502));
    assert!(parsed.query_params().is_empty());
}

#[test]
fn test_tls_url_parsing() {
    let url = "tls://example.com:8883";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse URL");
    
    assert_eq!(parsed.scheme(), "tls");
    assert_eq!(parsed.host(), "example.com");
    assert_eq!(parsed.port(), Some(8883));
}

#[test]
fn test_scheme_stacking() {
    let url = "tcp+tls://secure.example.com:443";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse URL");
    
    let schemes = parsed.schemes();
    assert_eq!(schemes.len(), 2);
    assert_eq!(schemes[0], endpoint_kit::Scheme::Tcp);
    assert_eq!(schemes[1], endpoint_kit::Scheme::Tls);
    assert_eq!(parsed.host(), "secure.example.com");
    assert_eq!(parsed.port(), Some(443));
}

#[test]
fn test_serial_url_parsing() {
    let url = "serial:///dev/ttyUSB0?baud=9600&data_bits=8";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse URL");
    
    assert_eq!(parsed.scheme(), "serial");
    assert_eq!(parsed.path(), Some("/dev/ttyUSB0"));
    
    let params = parsed.query_params();
    assert_eq!(params.get("baud"), Some(&"9600".to_string()));
    assert_eq!(params.get("data_bits"), Some(&"8".to_string()));
}

#[test]
fn test_query_parameters() {
    let url = "tcp://host:1234?timeout=30&retry=3&keepalive=true";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse URL");
    
    let params = parsed.query_params();
    assert_eq!(params.get("timeout"), Some(&"30".to_string()));
    assert_eq!(params.get("retry"), Some(&"3".to_string()));
    assert_eq!(params.get("keepalive"), Some(&"true".to_string()));
}

#[test]
fn test_url_normalization() {
    let url = "tcp://HOST.EXAMPLE.COM:502";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse URL");
    let normalized = NormalizedUrl::from(parsed);
    
    // 主机名应该被标准化为小写
    assert_eq!(normalized.host, "host.example.com");
    assert_eq!(normalized.port, Some(502));
}

#[test]
fn test_default_ports() {
    // HTTP默认端口
    let url1 = "http://example.com";
    let parsed1 = EndpointUrl::parse(url1).expect("Failed to parse URL");
    assert_eq!(parsed1.effective_port(), Some(80));
    
    // HTTPS默认端口
    let url2 = "https://example.com";
    let parsed2 = EndpointUrl::parse(url2).expect("Failed to parse URL");
    assert_eq!(parsed2.effective_port(), Some(443));
    
    // Modbus默认端口
    let url3 = "modbus://plc.local";
    let parsed3 = EndpointUrl::parse(url3).expect("Failed to parse URL");
    assert_eq!(parsed3.effective_port(), Some(502));
}

#[test]
fn test_ipv6_addresses() {
    let url = "tcp://[::1]:502";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse IPv6 URL");
    
    assert_eq!(parsed.scheme(), "tcp");
    assert_eq!(parsed.host(), "::1");
    assert_eq!(parsed.port(), Some(502));
}

#[test]
fn test_invalid_urls() {
    // 无效的scheme
    assert!(EndpointUrl::parse("invalid://host:123").is_err());
    
    // 缺少主机名
    assert!(EndpointUrl::parse("tcp://:502").is_err());
    
    // 无效的端口
    assert!(EndpointUrl::parse("tcp://host:99999").is_err());
    
    // 空URL
    assert!(EndpointUrl::parse("").is_err());
}

#[test]
fn test_url_equality() {
    let url1 = EndpointUrl::parse("tcp://example.com:502").unwrap();
    let url2 = EndpointUrl::parse("tcp://example.com:502").unwrap();
    let url3 = EndpointUrl::parse("tcp://other.com:502").unwrap();
    
    assert_eq!(url1, url2);
    assert_ne!(url1, url3);
}

#[test]
fn test_url_display() {
    let url = EndpointUrl::parse("tcp+tls://example.com:8883?sni=test").unwrap();
    let display_str = format!("{}", url);
    
    assert!(display_str.contains("tcp+tls"));
    assert!(display_str.contains("example.com:8883"));
    assert!(display_str.contains("sni=test"));
}

#[test]
fn test_url_cloning() {
    let original = EndpointUrl::parse("tcp://host:1234?param=value").unwrap();
    let cloned = original.clone();
    
    assert_eq!(original, cloned);
    assert_eq!(original.scheme(), cloned.scheme());
    assert_eq!(original.host(), cloned.host());
    assert_eq!(original.port(), cloned.port());
    assert_eq!(original.query_params(), cloned.query_params());
}

#[test]
fn test_complex_serial_url() {
    let url = "serial:///dev/ttyUSB0?baud=19200&data_bits=8&stop_bits=1&parity=none&flow_control=none";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse complex serial URL");
    
    assert_eq!(parsed.scheme(), "serial");
    assert_eq!(parsed.path(), Some("/dev/ttyUSB0"));
    
    let params = parsed.query_params();
    assert_eq!(params.get("baud"), Some(&"19200".to_string()));
    assert_eq!(params.get("data_bits"), Some(&"8".to_string()));
    assert_eq!(params.get("stop_bits"), Some(&"1".to_string()));
    assert_eq!(params.get("parity"), Some(&"none".to_string()));
    assert_eq!(params.get("flow_control"), Some(&"none".to_string()));
}

#[test]
fn test_url_with_special_characters() {
    let url = "tcp://test-host.example-domain.com:502?param=value%20with%20spaces";
    let parsed = EndpointUrl::parse(url).expect("Failed to parse URL with special chars");
    
    assert_eq!(parsed.host(), "test-host.example-domain.com");
    assert_eq!(parsed.port(), Some(502));
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_port_range_validation(port in 1u16..=65535) {
            let url = format!("tcp://localhost:{}", port);
            let parsed = EndpointUrl::parse(&url).unwrap();
            assert_eq!(parsed.port(), Some(port));
        }
        
        #[test]
        fn test_invalid_port_range(port in 65536u32..=100000) {
            let url = format!("tcp://localhost:{}", port);
            assert!(EndpointUrl::parse(&url).is_err());
        }
        
        #[test]
        fn test_hostname_parsing(
            hostname in "[a-zA-Z][a-zA-Z0-9\\-]{0,20}\\.[a-zA-Z]{2,4}"
        ) {
            let url = format!("tcp://{}:502", hostname);
            let parsed = EndpointUrl::parse(&url).unwrap();
            assert_eq!(parsed.host(), hostname);
        }
    }
}