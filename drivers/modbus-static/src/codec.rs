//! Modbus数据编解码

use crate::config::{DataType, Endian, RegPoint};
use frame_bus::Value;
use anyhow::Result;

/// 解码Modbus寄存器数据
pub fn decode_registers(
    regs: &[u16],
    point: &RegPoint,
    start_addr: u16,
    endian: &Endian,
) -> Result<Value> {
    let offset = (point.addr.saturating_sub(start_addr)) as usize;
    
    if offset >= regs.len() {
        return Err(anyhow::anyhow!("Register offset {} out of bounds", offset));
    }

    let value = match point.datatype {
        DataType::Bool => {
            let reg = regs[offset];
            Value::bool(reg != 0)
        }
        DataType::Uint16 => {
            let reg = regs[offset];
            let val = match endian {
                Endian::Big => reg,
                Endian::Little => reg.swap_bytes(),
            };
            Value::int(val as i64)
        }
        DataType::Int16 => {
            let reg = regs[offset];
            let val = match endian {
                Endian::Big => reg as i16,
                Endian::Little => (reg.swap_bytes()) as i16,
            };
            Value::int(val as i64)
        }
        DataType::Uint32 => {
            if offset + 1 >= regs.len() {
                return Err(anyhow::anyhow!("Not enough registers for uint32"));
            }
            let val = match endian {
                Endian::Big => {
                    ((regs[offset] as u32) << 16) | (regs[offset + 1] as u32)
                }
                Endian::Little => {
                    ((regs[offset + 1] as u32) << 16) | (regs[offset] as u32)
                }
            };
            Value::int(val as i64)
        }
        DataType::Int32 => {
            if offset + 1 >= regs.len() {
                return Err(anyhow::anyhow!("Not enough registers for int32"));
            }
            let val = match endian {
                Endian::Big => {
                    ((regs[offset] as u32) << 16) | (regs[offset + 1] as u32)
                }
                Endian::Little => {
                    ((regs[offset + 1] as u32) << 16) | (regs[offset] as u32)
                }
            } as i32;
            Value::int(val as i64)
        }
        DataType::Float32 => {
            if offset + 1 >= regs.len() {
                return Err(anyhow::anyhow!("Not enough registers for float32"));
            }
            let bits = match endian {
                Endian::Big => {
                    ((regs[offset] as u32) << 16) | (regs[offset + 1] as u32)
                }
                Endian::Little => {
                    ((regs[offset + 1] as u32) << 16) | (regs[offset] as u32)
                }
            };
            let val = f32::from_bits(bits);
            Value::float(val as f64)
        }
        DataType::Float64 => {
            if offset + 3 >= regs.len() {
                return Err(anyhow::anyhow!("Not enough registers for float64"));
            }
            let bits = match endian {
                Endian::Big => {
                    ((regs[offset] as u64) << 48)
                        | ((regs[offset + 1] as u64) << 32)
                        | ((regs[offset + 2] as u64) << 16)
                        | (regs[offset + 3] as u64)
                }
                Endian::Little => {
                    ((regs[offset + 3] as u64) << 48)
                        | ((regs[offset + 2] as u64) << 32)
                        | ((regs[offset + 1] as u64) << 16)
                        | (regs[offset] as u64)
                }
            };
            let val = f64::from_bits(bits);
            Value::float(val)
        }
    };

    Ok(value)
}

/// 编码值为Modbus寄存器（用于写入）
pub fn encode_value(
    value: &Value,
    datatype: &DataType,
    endian: &Endian,
) -> Result<Vec<u16>> {
    match datatype {
        DataType::Bool => {
            let val = value.to_bool().unwrap_or(false);
            Ok(vec![if val { 1 } else { 0 }])
        }
        DataType::Uint16 => {
            let val = value.to_i64().unwrap_or(0) as u16;
            let reg = match endian {
                Endian::Big => val,
                Endian::Little => val.swap_bytes(),
            };
            Ok(vec![reg])
        }
        DataType::Int16 => {
            let val = value.to_i64().unwrap_or(0) as i16 as u16;
            let reg = match endian {
                Endian::Big => val,
                Endian::Little => val.swap_bytes(),
            };
            Ok(vec![reg])
        }
        DataType::Uint32 => {
            let val = value.to_i64().unwrap_or(0) as u32;
            match endian {
                Endian::Big => Ok(vec![(val >> 16) as u16, val as u16]),
                Endian::Little => Ok(vec![val as u16, (val >> 16) as u16]),
            }
        }
        DataType::Int32 => {
            let val = value.to_i64().unwrap_or(0) as i32 as u32;
            match endian {
                Endian::Big => Ok(vec![(val >> 16) as u16, val as u16]),
                Endian::Little => Ok(vec![val as u16, (val >> 16) as u16]),
            }
        }
        DataType::Float32 => {
            let val = value.to_f64().unwrap_or(0.0) as f32;
            let bits = val.to_bits();
            match endian {
                Endian::Big => Ok(vec![(bits >> 16) as u16, bits as u16]),
                Endian::Little => Ok(vec![bits as u16, (bits >> 16) as u16]),
            }
        }
        DataType::Float64 => {
            let val = value.to_f64().unwrap_or(0.0);
            let bits = val.to_bits();
            match endian {
                Endian::Big => Ok(vec![
                    (bits >> 48) as u16,
                    (bits >> 32) as u16,
                    (bits >> 16) as u16,
                    bits as u16,
                ]),
                Endian::Little => Ok(vec![
                    bits as u16,
                    (bits >> 16) as u16,
                    (bits >> 32) as u16,
                    (bits >> 48) as u16,
                ]),
            }
        }
    }
}

/// 应用缩放表达式
pub fn apply_scale(value: Value, scale_expr: Option<&str>) -> Result<Value> {
    match scale_expr {
        Some(expr) => {
            // 简单实现：支持基本的数学表达式
            // 完整实现应该使用Lua解释器
            if let Some(raw_val) = value.to_f64() {
                let result = eval_simple_expression(expr, raw_val)?;
                Ok(Value::float(result))
            } else {
                Ok(value)
            }
        }
        None => Ok(value),
    }
}

/// 简单表达式求值器（基础实现）
fn eval_simple_expression(expr: &str, value: f64) -> Result<f64> {
    // 支持形如 "value / 10.0", "value * 0.1", "value + 100" 的表达式
    let expr = expr.trim().replace("value", &value.to_string());
    
    // 非常简单的解析器，仅用于演示
    if let Some(pos) = expr.find('/') {
        let left: f64 = expr[..pos].trim().parse()?;
        let right: f64 = expr[pos + 1..].trim().parse()?;
        
        // 检查除零错误
        if right == 0.0 {
            return Err(anyhow::anyhow!("Division by zero in scale expression"));
        }
        
        Ok(left / right)
    } else if let Some(pos) = expr.find('*') {
        let left: f64 = expr[..pos].trim().parse()?;
        let right: f64 = expr[pos + 1..].trim().parse()?;
        Ok(left * right)
    } else if let Some(pos) = expr.find('+') {
        let left: f64 = expr[..pos].trim().parse()?;
        let right: f64 = expr[pos + 1..].trim().parse()?;
        Ok(left + right)
    } else if let Some(pos) = expr.find('-') {
        let left: f64 = expr[..pos].trim().parse()?;
        let right: f64 = expr[pos + 1..].trim().parse()?;
        Ok(left - right)
    } else {
        // 直接数值
        Ok(expr.parse()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_uint16_big_endian() {
        let regs = vec![0x1234];
        let point = RegPoint {
            tag: "test".to_string(),
            func: tokio_modbus::FunctionCode::ReadHoldingRegisters,
            addr: 0,
            len: 1,
            datatype: DataType::Uint16,
            scale: None,
            access: crate::config::Access::R,
        };
        
        let value = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
        assert_eq!(value.to_i64(), Some(0x1234));
    }

    #[test]
    fn test_simple_scale_expression() {
        let result = eval_simple_expression("123.0 / 10.0", 123.0).unwrap();
        assert_eq!(result, 12.3);
    }
}