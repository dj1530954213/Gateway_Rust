//! Mock PLC服务器，用于测试Modbus驱动

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;

/// Mock PLC服务器
pub struct MockPLCServer {
    registers: Arc<RwLock<HashMap<u16, u16>>>,
    coils: Arc<RwLock<HashMap<u16, bool>>>,
    address: SocketAddr,
    _listener_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MockPLCServer {
    /// 创建新的Mock PLC服务器
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        
        let registers = Arc::new(RwLock::new(HashMap::new()));
        let coils = Arc::new(RwLock::new(HashMap::new()));
        
        let server_registers = registers.clone();
        let server_coils = coils.clone();
        
        let handle = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let registers = server_registers.clone();
                let coils = server_coils.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = handle_modbus_connection(stream, registers, coils).await {
                        eprintln!("Modbus connection error: {}", e);
                    }
                });
            }
        });
        
        Ok(Self {
            registers,
            coils,
            address,
            _listener_handle: Some(handle),
        })
    }
    
    /// 获取服务器地址
    pub fn address(&self) -> SocketAddr {
        self.address
    }
    
    /// 获取端点URL
    pub fn endpoint_url(&self) -> String {
        format!("tcp://{}", self.address)
    }
    
    /// 设置保持寄存器值
    pub async fn set_holding_register(&self, addr: u16, value: u16) {
        self.registers.write().await.insert(addr, value);
    }
    
    /// 获取保持寄存器值
    pub async fn get_holding_register(&self, addr: u16) -> Option<u16> {
        self.registers.read().await.get(&addr).copied()
    }
    
    /// 设置线圈值
    pub async fn set_coil(&self, addr: u16, value: bool) {
        self.coils.write().await.insert(addr, value);
    }
    
    /// 批量设置寄存器
    pub async fn set_registers(&self, start_addr: u16, values: &[u16]) {
        let mut registers = self.registers.write().await;
        for (i, &value) in values.iter().enumerate() {
            registers.insert(start_addr + i as u16, value);
        }
    }
    
    /// 模拟数据变化
    pub async fn simulate_data_change(&self) {
        let mut registers = self.registers.write().await;
        
        // 模拟流量计数据 (40001)
        let flow = 1000 + (rand::random::<u16>() % 500); // 100.0 - 150.0 m³/h
        registers.insert(40001, flow);
        
        // 模拟压力传感器 (40002)  
        let pressure = 500 + (rand::random::<u16>() % 300); // 5.0 - 8.0 bar
        registers.insert(40002, pressure);
        
        // 模拟温度传感器 (40003)
        let temp = 200 + (rand::random::<u16>() % 150); // 20.0 - 35.0 °C
        registers.insert(40003, temp);
    }
}

/// 处理Modbus TCP连接
async fn handle_modbus_connection(
    mut stream: TcpStream,
    registers: Arc<RwLock<HashMap<u16, u16>>>,
    coils: Arc<RwLock<HashMap<u16, bool>>>,
) -> Result<()> {
    let mut buffer = [0u8; 256];
    
    loop {
        // 读取MBAP头部 (6字节)
        let n = stream.read(&mut buffer[..6]).await?;
        if n == 0 {
            break; // 连接关闭
        }
        if n < 6 {
            continue; // 不完整的请求
        }
        
        // 解析MBAP头部
        let transaction_id = u16::from_be_bytes([buffer[0], buffer[1]]);
        let _protocol_id = u16::from_be_bytes([buffer[2], buffer[3]]);
        let length = u16::from_be_bytes([buffer[4], buffer[5]]);
        
        // 读取PDU
        if length < 2 {
            continue;
        }
        
        let pdu_len = (length - 1) as usize; // 减去unit_id
        let n = stream.read(&mut buffer[6..6 + pdu_len]).await?;
        if n < pdu_len {
            continue;
        }
        
        let unit_id = buffer[6];
        let function_code = buffer[7];
        
        // 处理不同的功能码
        let response = match function_code {
            0x03 => handle_read_holding_registers(&buffer[8..6 + pdu_len], &registers).await?,
            0x04 => handle_read_input_registers(&buffer[8..6 + pdu_len], &registers).await?,
            0x01 => handle_read_coils(&buffer[8..6 + pdu_len], &coils).await?,
            0x05 => handle_write_single_coil(&buffer[8..6 + pdu_len], &coils).await?,
            0x06 => handle_write_single_register(&buffer[8..6 + pdu_len], &registers).await?,
            _ => {
                // 不支持的功能码
                vec![function_code | 0x80, 0x01] // 异常码: 非法功能
            }
        };
        
        // 构造响应MBAP头部
        let response_length = response.len() + 1; // +1 for unit_id
        let mbap = [
            (transaction_id >> 8) as u8, transaction_id as u8,
            0x00, 0x00, // Protocol ID
            (response_length >> 8) as u8, response_length as u8,
            unit_id,
        ];
        
        // 发送响应
        stream.write_all(&mbap).await?;
        stream.write_all(&response).await?;
    }
    
    Ok(())
}

/// 处理读保持寄存器 (0x03)
async fn handle_read_holding_registers(
    pdu: &[u8],
    registers: &Arc<RwLock<HashMap<u16, u16>>>,
) -> Result<Vec<u8>> {
    if pdu.len() < 4 {
        return Ok(vec![0x83, 0x02]); // 异常码: 非法数据地址
    }
    
    let start_addr = u16::from_be_bytes([pdu[0], pdu[1]]);
    let qty = u16::from_be_bytes([pdu[2], pdu[3]]);
    
    if qty == 0 || qty > 125 {
        return Ok(vec![0x83, 0x03]); // 异常码: 非法数据值
    }
    
    let registers_map = registers.read().await;
    let mut response = vec![0x03, (qty * 2) as u8]; // 功能码 + 字节数
    
    for i in 0..qty {
        let addr = start_addr + i;
        let value = registers_map.get(&addr).copied().unwrap_or(0);
        response.push((value >> 8) as u8);
        response.push(value as u8);
    }
    
    Ok(response)
}

/// 处理读输入寄存器 (0x04)
async fn handle_read_input_registers(
    pdu: &[u8],
    registers: &Arc<RwLock<HashMap<u16, u16>>>,
) -> Result<Vec<u8>> {
    if pdu.len() < 4 {
        return Ok(vec![0x84, 0x02]);
    }
    
    let start_addr = u16::from_be_bytes([pdu[0], pdu[1]]);
    let qty = u16::from_be_bytes([pdu[2], pdu[3]]);
    
    if qty == 0 || qty > 125 {
        return Ok(vec![0x84, 0x03]);
    }
    
    let registers_map = registers.read().await;
    let mut response = vec![0x04, (qty * 2) as u8];
    
    for i in 0..qty {
        let addr = start_addr + i;
        let value = registers_map.get(&addr).copied().unwrap_or(0);
        response.push((value >> 8) as u8);
        response.push(value as u8);
    }
    
    Ok(response)
}

/// 处理读线圈 (0x01)
async fn handle_read_coils(
    pdu: &[u8],
    coils: &Arc<RwLock<HashMap<u16, bool>>>,
) -> Result<Vec<u8>> {
    if pdu.len() < 4 {
        return Ok(vec![0x81, 0x02]);
    }
    
    let start_addr = u16::from_be_bytes([pdu[0], pdu[1]]);
    let qty = u16::from_be_bytes([pdu[2], pdu[3]]);
    
    if qty == 0 || qty > 2000 {
        return Ok(vec![0x81, 0x03]);
    }
    
    let coils_map = coils.read().await;
    let byte_count = (qty + 7) / 8;
    let mut response = vec![0x01, byte_count as u8];
    
    for byte_idx in 0..byte_count {
        let mut byte_value = 0u8;
        for bit_idx in 0..8 {
            let coil_addr = start_addr + byte_idx * 8 + bit_idx;
            if coil_addr < start_addr + qty {
                if coils_map.get(&coil_addr).copied().unwrap_or(false) {
                    byte_value |= 1 << bit_idx;
                }
            }
        }
        response.push(byte_value);
    }
    
    Ok(response)
}

/// 处理写单个线圈 (0x05)
async fn handle_write_single_coil(
    pdu: &[u8],
    coils: &Arc<RwLock<HashMap<u16, bool>>>,
) -> Result<Vec<u8>> {
    if pdu.len() < 4 {
        return Ok(vec![0x85, 0x02]);
    }
    
    let addr = u16::from_be_bytes([pdu[0], pdu[1]]);
    let value = u16::from_be_bytes([pdu[2], pdu[3]]);
    
    let coil_value = match value {
        0x0000 => false,
        0xFF00 => true,
        _ => return Ok(vec![0x85, 0x03]),
    };
    
    coils.write().await.insert(addr, coil_value);
    
    // 回显请求
    Ok(pdu.to_vec())
}

/// 处理写单个寄存器 (0x06)
async fn handle_write_single_register(
    pdu: &[u8],
    registers: &Arc<RwLock<HashMap<u16, u16>>>,
) -> Result<Vec<u8>> {
    if pdu.len() < 4 {
        return Ok(vec![0x86, 0x02]);
    }
    
    let addr = u16::from_be_bytes([pdu[0], pdu[1]]);
    let value = u16::from_be_bytes([pdu[2], pdu[3]]);
    
    registers.write().await.insert(addr, value);
    
    // 回显请求
    Ok(pdu.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_modbus::client::tcp::connect;
    
    #[tokio::test]
    async fn test_mock_plc_basic_operations() {
        let plc = MockPLCServer::new().await.unwrap();
        
        // 设置一些测试数据
        plc.set_holding_register(40001, 1500).await;
        plc.set_holding_register(40002, 500).await;
        
        // 验证数据设置成功
        assert_eq!(plc.get_holding_register(40001).await, Some(1500));
        assert_eq!(plc.get_holding_register(40002).await, Some(500));
        
        // 测试Modbus客户端连接
        let mut ctx = connect(plc.address()).await.unwrap();
        let result = ctx.read_holding_registers(40001, 2).await.unwrap();
        
        assert_eq!(result, vec![1500, 500]);
    }
}