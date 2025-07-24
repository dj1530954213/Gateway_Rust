//! 简化的 frame-bus 测试

use frame_bus::{DataFrame, Value};
use tempfile::tempdir;

#[tokio::test]
async fn test_frame_creation() {
    // 测试不同类型的 Value 创建
    let bool_value = Value::bool(true);
    let int_value = Value::int(42);
    let float_value = Value::float(3.14);
    let string_value = Value::string("test");
    let bytes_value = Value::bytes(vec![1, 2, 3]);

    // 测试 DataFrame 创建
    let frame1 = DataFrame::new("test.bool", bool_value);
    let frame2 = DataFrame::new("test.int", int_value);
    let frame3 = DataFrame::new("test.float", float_value);
    let frame4 = DataFrame::new("test.string", string_value);
    let frame5 = DataFrame::new("test.bytes", bytes_value);

    // 验证标签正确设置
    assert_eq!(frame1.tag, "test.bool");
    assert_eq!(frame2.tag, "test.int");
    assert_eq!(frame3.tag, "test.float");
    assert_eq!(frame4.tag, "test.string");
    assert_eq!(frame5.tag, "test.bytes");
}

#[tokio::test]
async fn test_frame_bus_init() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let result = frame_bus::init(1024, temp_dir.path());
    assert!(result.is_ok());
}