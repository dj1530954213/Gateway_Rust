//! Modbus编解码测试

use modbus_static::codec::{decode_registers, encode_value, apply_scale};
use modbus_static::config::{DataType, Endian, RegPoint, Access};
use frame_bus::Value;
use tokio_modbus::FunctionCode;

/// 创建测试用的寄存器点位
fn create_test_point(addr: u16, datatype: DataType, len: u16) -> RegPoint {
    RegPoint {
        tag: format!("test.point.{}", addr),
        func: FunctionCode::ReadHoldingRegisters,
        addr,
        len,
        datatype,
        scale: None,
        access: Access::R,
    }
}

#[test]
fn test_uint16_decoding_big_endian() {
    let regs = vec![0x1234];
    let point = create_test_point(0, DataType::Uint16, 1);
    
    let value = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value.to_i64(), Some(0x1234));
}

#[test]
fn test_uint16_decoding_little_endian() {
    let regs = vec![0x1234];
    let point = create_test_point(0, DataType::Uint16, 1);
    
    let value = decode_registers(&regs, &point, 0, &Endian::Little).unwrap();
    assert_eq!(value.to_i64(), Some(0x3412)); // 字节交换
}

#[test]
fn test_int16_decoding() {
    // 测试正数
    let regs_positive = vec![0x1234];
    let point = create_test_point(0, DataType::Int16, 1);
    
    let value = decode_registers(&regs_positive, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value.to_i64(), Some(0x1234));
    
    // 测试负数
    let regs_negative = vec![0x8000]; // -32768
    let value_neg = decode_registers(&regs_negative, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value_neg.to_i64(), Some(-32768));
}

#[test]
fn test_uint32_decoding() {
    let regs = vec![0x1234, 0x5678];
    let point = create_test_point(0, DataType::Uint32, 2);
    
    // Big endian: 高位在前
    let value_be = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value_be.to_i64(), Some(0x12345678));
    
    // Little endian: 低位在前
    let value_le = decode_registers(&regs, &point, 0, &Endian::Little).unwrap();
    assert_eq!(value_le.to_i64(), Some(0x56781234));
}

#[test]
fn test_int32_decoding() {
    let regs = vec![0x8000, 0x0000]; // -2147483648
    let point = create_test_point(0, DataType::Int32, 2);
    
    let value = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value.to_i64(), Some(-2147483648));
}

#[test]
fn test_float32_decoding() {
    // IEEE 754 float32: 3.14159 ≈ 0x40490FDB
    let regs = vec![0x4049, 0x0FDB];
    let point = create_test_point(0, DataType::Float32, 2);
    
    let value = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
    let float_val = value.to_f64().unwrap();
    
    // 允许浮点误差
    assert!((float_val - 3.14159).abs() < 0.0001);
}

#[test]
fn test_float64_decoding() {
    // IEEE 754 float64: 3.141592653589793 ≈ 0x400921FB54442D18
    let regs = vec![0x4009, 0x21FB, 0x5444, 0x2D18];
    let point = create_test_point(0, DataType::Float64, 4);
    
    let value = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
    let float_val = value.to_f64().unwrap();
    
    assert!((float_val - 3.141592653589793).abs() < 0.000000000001);
}

#[test]
fn test_bool_decoding() {
    let point = create_test_point(0, DataType::Bool, 1);
    
    // 测试true值
    let regs_true = vec![0x0001];
    let value_true = decode_registers(&regs_true, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value_true.to_bool(), Some(true));
    
    // 测试false值
    let regs_false = vec![0x0000];
    let value_false = decode_registers(&regs_false, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value_false.to_bool(), Some(false));
    
    // 测试非零值应该为true
    let regs_nonzero = vec![0x1234];
    let value_nonzero = decode_registers(&regs_nonzero, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value_nonzero.to_bool(), Some(true));
}

#[test]
fn test_address_offset() {
    let regs = vec![0x1111, 0x2222, 0x3333];
    let point = create_test_point(2, DataType::Uint16, 1); // 从偏移2开始
    
    let value = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value.to_i64(), Some(0x3333)); // 应该读取第三个寄存器
}

#[test]
fn test_decoding_errors() {
    let regs = vec![0x1234];
    let point = create_test_point(5, DataType::Uint16, 1); // 超出范围的地址
    
    let result = decode_registers(&regs, &point, 0, &Endian::Big);
    assert!(result.is_err(), "Should fail when address is out of bounds");
    
    // 测试需要多个寄存器但数据不足
    let point_uint32 = create_test_point(0, DataType::Uint32, 2);
    let result_uint32 = decode_registers(&regs, &point_uint32, 0, &Endian::Big);
    assert!(result_uint32.is_err(), "Should fail when not enough registers for uint32");
}

// 编码测试
#[test]
fn test_uint16_encoding() {
    let value = Value::int(0x1234);
    
    let regs_be = encode_value(&value, &DataType::Uint16, &Endian::Big).unwrap();
    assert_eq!(regs_be, vec![0x1234]);
    
    let regs_le = encode_value(&value, &DataType::Uint16, &Endian::Little).unwrap();
    assert_eq!(regs_le, vec![0x3412]); // 字节交换
}

#[test]
fn test_int16_encoding() {
    let value = Value::int(-1000);
    
    let regs = encode_value(&value, &DataType::Int16, &Endian::Big).unwrap();
    assert_eq!(regs, vec![0xFC18]); // -1000 as uint16
}

#[test]
fn test_uint32_encoding() {
    let value = Value::int(0x12345678);
    
    let regs_be = encode_value(&value, &DataType::Uint32, &Endian::Big).unwrap();
    assert_eq!(regs_be, vec![0x1234, 0x5678]);
    
    let regs_le = encode_value(&value, &DataType::Uint32, &Endian::Little).unwrap();
    assert_eq!(regs_le, vec![0x5678, 0x1234]);
}

#[test]
fn test_float32_encoding() {
    let value = Value::float(3.14159);
    
    let regs = encode_value(&value, &DataType::Float32, &Endian::Big).unwrap();
    
    // 验证往返转换
    let decoded = decode_float32_from_regs(&regs, &Endian::Big).unwrap();
    assert!((decoded - 3.14159).abs() < 0.0001);
}

#[test]
fn test_bool_encoding() {
    let value_true = Value::bool(true);
    let regs_true = encode_value(&value_true, &DataType::Bool, &Endian::Big).unwrap();
    assert_eq!(regs_true, vec![1]);
    
    let value_false = Value::bool(false);
    let regs_false = encode_value(&value_false, &DataType::Bool, &Endian::Big).unwrap();
    assert_eq!(regs_false, vec![0]);
}

#[test]
fn test_roundtrip_encoding_decoding() {
    let test_cases = vec![
        (Value::int(12345), DataType::Uint16),
        (Value::int(-6789), DataType::Int16),
        (Value::int(0x12345678), DataType::Uint32),
        (Value::int(-123456789), DataType::Int32),
        (Value::float(3.14159), DataType::Float32),
        (Value::float(2.718281828459045), DataType::Float64),
        (Value::bool(true), DataType::Bool),
        (Value::bool(false), DataType::Bool),
    ];
    
    for (original_value, datatype) in test_cases {
        let endians = vec![Endian::Big, Endian::Little];
        
        for endian in endians {
            // 编码
            let regs = encode_value(&original_value, &datatype, &endian).unwrap();
            
            // 创建对应的点位
            let point = create_test_point(0, datatype.clone(), regs.len() as u16);
            
            // 解码
            let decoded_value = decode_registers(&regs, &point, 0, &endian).unwrap();
            
            // 验证一致性
            match (&original_value, &decoded_value) {
                (a, b) if a.to_bool().is_some() && b.to_bool().is_some() => assert_eq!(a.to_bool(), b.to_bool()),
                (a, b) if a.to_i64().is_some() && b.to_i64().is_some() => assert_eq!(a.to_i64(), b.to_i64()),
                (a, b) if a.to_f64().is_some() && b.to_f64().is_some() => {
                    let a_val = a.to_f64().unwrap();
                    let b_val = b.to_f64().unwrap();
                    assert!((a_val - b_val).abs() < 0.0001, "Float values differ: {} vs {}", a_val, b_val);
                }
                _ => panic!("Type mismatch in roundtrip test"),
            }
        }
    }
}

#[test]
fn test_scale_application() {
    // 测试简单缩放
    let value = Value::int(1500);
    let scaled = apply_scale(value, Some("value / 10.0")).unwrap();
    assert_eq!(scaled.to_f64(), Some(150.0));
    
    // 测试乘法缩放
    let value2 = Value::float(2.5);
    let scaled2 = apply_scale(value2, Some("value * 100.0")).unwrap();
    assert_eq!(scaled2.to_f64(), Some(250.0));
    
    // 测试加法缩放
    let value3 = Value::int(20);
    let scaled3 = apply_scale(value3, Some("value + 273.15")).unwrap();
    assert!((scaled3.to_f64().unwrap() - 293.15).abs() < 0.01);
    
    // 测试减法缩放
    let value4 = Value::int(100);
    let scaled4 = apply_scale(value4, Some("value - 50")).unwrap();
    assert_eq!(scaled4.to_f64(), Some(50.0));
    
    // 测试无缩放
    let value5 = Value::int(42);
    let no_scale = apply_scale(value5.clone(), None).unwrap();
    assert_eq!(no_scale.to_i64(), value5.to_i64());
}

#[test]
fn test_complex_scale_expressions() {
    // 测试组合表达式（当实现更复杂的表达式解析器时）
    let value = Value::int(1000);
    
    // 目前的简单实现只支持基本操作
    // 这些测试用例为未来扩展做准备
    let result = apply_scale(value, Some("value / 10.0"));
    assert!(result.is_ok());
}

#[test]
fn test_invalid_scale_expressions() {
    let value = Value::int(100);
    
    // 测试无效表达式
    let result = apply_scale(value, Some("invalid_expression"));
    assert!(result.is_err(), "Should fail with invalid expression");
    
    // 测试除零
    let value2 = Value::int(100);
    let result2 = apply_scale(value2, Some("value / 0"));
    assert!(result2.is_err(), "Should fail with division by zero");
}

// 辅助函数
fn decode_float32_from_regs(regs: &[u16], endian: &Endian) -> Result<f32, anyhow::Error> {
    if regs.len() < 2 {
        return Err(anyhow::anyhow!("Not enough registers for float32"));
    }
    
    let bits = match endian {
        Endian::Big => ((regs[0] as u32) << 16) | (regs[1] as u32),
        Endian::Little => ((regs[1] as u32) << 16) | (regs[0] as u32),
    };
    
    Ok(f32::from_bits(bits))
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_uint16_roundtrip(value in 0u16..=u16::MAX) {
            let val = Value::int(value as i64);
            let regs = encode_value(&val, &DataType::Uint16, &Endian::Big).unwrap();
            let point = create_test_point(0, DataType::Uint16, 1);
            let decoded = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
            assert_eq!(decoded.to_i64(), Some(value as i64));
        }
        
        #[test]
        fn test_int16_roundtrip(value in i16::MIN..=i16::MAX) {
            let val = Value::int(value as i64);
            let regs = encode_value(&val, &DataType::Int16, &Endian::Big).unwrap();
            let point = create_test_point(0, DataType::Int16, 1);
            let decoded = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
            assert_eq!(decoded.to_i64(), Some(value as i64));
        }
        
        #[test]
        fn test_bool_roundtrip(value in any::<bool>()) {
            let val = Value::bool(value);
            let regs = encode_value(&val, &DataType::Bool, &Endian::Big).unwrap();
            let point = create_test_point(0, DataType::Bool, 1);
            let decoded = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
            assert_eq!(decoded.to_bool(), Some(value));
        }
    }
}