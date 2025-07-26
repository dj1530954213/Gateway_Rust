/*!
# Modbus Slave Bridge

Modbus Slave桥接实现，将内部数据暴露为Modbus从站
*/

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};
// Modbus support temporarily disabled due to version conflicts
// use tokio_modbus::prelude::*;

use crate::bridge::*;
use crate::{BridgeError, Result};

/// Modbus桥接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusConfig {
    /// 基础桥接配置
    pub base: BridgeConfig,
    /// 从站ID
    pub slave_id: u8,
    /// 传输模式
    pub transport: ModbusTransport,
    /// 线圈起始地址
    pub coils_start: u16,
    /// 线圈数量
    pub coils_count: u16,
    /// 离散输入起始地址
    pub discrete_inputs_start: u16,
    /// 离散输入数量
    pub discrete_inputs_count: u16,
    /// 保持寄存器起始地址
    pub holding_registers_start: u16,
    /// 保持寄存器数量
    pub holding_registers_count: u16,
    /// 输入寄存器起始地址
    pub input_registers_start: u16,
    /// 输入寄存器数量
    pub input_registers_count: u16,
    /// 字节序
    pub byte_order: ByteOrder,
    /// 字顺序
    pub word_order: WordOrder,
}

/// Modbus传输模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModbusTransport {
    /// TCP传输
    Tcp,
    /// RTU传输（串口）
    Rtu,
    /// ASCII传输（串口）
    Ascii,
}

/// 字节序
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByteOrder {
    /// 大端序
    BigEndian,
    /// 小端序
    LittleEndian,
}

/// 字顺序
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WordOrder {
    /// 高字在前
    HighFirst,
    /// 低字在前
    LowFirst,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        let mut base = BridgeConfig::default();
        base.name = "modbus-slave".to_string();
        base.bridge_type = BridgeType::ModbusSlave;
        base.port = 502;
        
        Self {
            base,
            slave_id: 1,
            transport: ModbusTransport::Tcp,
            coils_start: 0,
            coils_count: 1000,
            discrete_inputs_start: 10000,
            discrete_inputs_count: 1000,
            holding_registers_start: 40000,
            holding_registers_count: 1000,
            input_registers_start: 30000,
            input_registers_count: 1000,
            byte_order: ByteOrder::BigEndian,
            word_order: WordOrder::HighFirst,
        }
    }
}

/// Modbus数据区域
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModbusRegion {
    /// 线圈（输出线圈）
    Coils,
    /// 离散输入
    DiscreteInputs,
    /// 保持寄存器
    HoldingRegisters,
    /// 输入寄存器
    InputRegisters,
}

/// Modbus数据点映射
#[derive(Debug, Clone)]
struct ModbusDataPoint {
    /// 内部数据点
    data_point: DataPoint,
    /// Modbus区域
    region: ModbusRegion,
    /// Modbus地址
    address: u16,
    /// 数据长度（对于寄存器）
    length: u16,
}

/// Modbus存储区
#[derive(Debug)]
struct ModbusStorage {
    /// 线圈存储
    coils: Vec<bool>,
    /// 离散输入存储
    discrete_inputs: Vec<bool>,
    /// 保持寄存器存储
    holding_registers: Vec<u16>,
    /// 输入寄存器存储
    input_registers: Vec<u16>,
}

impl ModbusStorage {
    fn new(config: &ModbusConfig) -> Self {
        Self {
            coils: vec![false; config.coils_count as usize],
            discrete_inputs: vec![false; config.discrete_inputs_count as usize],
            holding_registers: vec![0; config.holding_registers_count as usize],
            input_registers: vec![0; config.input_registers_count as usize],
        }
    }

    /// 读取线圈
    fn read_coils(&self, address: u16, count: u16) -> Result<Vec<bool>> {
        let start = address as usize;
        let end = start + count as usize;
        
        if end > self.coils.len() {
            return Err(BridgeError::modbus("Coil address out of range"));
        }
        
        Ok(self.coils[start..end].to_vec())
    }

    /// 写入线圈
    fn write_coils(&mut self, address: u16, values: &[bool]) -> Result<()> {
        let start = address as usize;
        let end = start + values.len();
        
        if end > self.coils.len() {
            return Err(BridgeError::modbus("Coil address out of range"));
        }
        
        self.coils[start..end].copy_from_slice(values);
        Ok(())
    }

    /// 读取离散输入
    fn read_discrete_inputs(&self, address: u16, count: u16) -> Result<Vec<bool>> {
        let start = address as usize;
        let end = start + count as usize;
        
        if end > self.discrete_inputs.len() {
            return Err(BridgeError::modbus("Discrete input address out of range"));
        }
        
        Ok(self.discrete_inputs[start..end].to_vec())
    }

    /// 写入离散输入
    fn write_discrete_inputs(&mut self, address: u16, values: &[bool]) -> Result<()> {
        let start = address as usize;
        let end = start + values.len();
        
        if end > self.discrete_inputs.len() {
            return Err(BridgeError::modbus("Discrete input address out of range"));
        }
        
        self.discrete_inputs[start..end].copy_from_slice(values);
        Ok(())
    }

    /// 读取保持寄存器
    fn read_holding_registers(&self, address: u16, count: u16) -> Result<Vec<u16>> {
        let start = address as usize;
        let end = start + count as usize;
        
        if end > self.holding_registers.len() {
            return Err(BridgeError::modbus("Holding register address out of range"));
        }
        
        Ok(self.holding_registers[start..end].to_vec())
    }

    /// 写入保持寄存器
    fn write_holding_registers(&mut self, address: u16, values: &[u16]) -> Result<()> {
        let start = address as usize;
        let end = start + values.len();
        
        if end > self.holding_registers.len() {
            return Err(BridgeError::modbus("Holding register address out of range"));
        }
        
        self.holding_registers[start..end].copy_from_slice(values);
        Ok(())
    }

    /// 读取输入寄存器
    fn read_input_registers(&self, address: u16, count: u16) -> Result<Vec<u16>> {
        let start = address as usize;
        let end = start + count as usize;
        
        if end > self.input_registers.len() {
            return Err(BridgeError::modbus("Input register address out of range"));
        }
        
        Ok(self.input_registers[start..end].to_vec())
    }

    /// 写入输入寄存器
    fn write_input_registers(&mut self, address: u16, values: &[u16]) -> Result<()> {
        let start = address as usize;
        let end = start + values.len();
        
        if end > self.input_registers.len() {
            return Err(BridgeError::modbus("Input register address out of range"));
        }
        
        self.input_registers[start..end].copy_from_slice(values);
        Ok(())
    }
}

/// Modbus Slave桥接实现
pub struct ModbusBridge {
    config: ModbusConfig,
    state: Arc<RwLock<BridgeState>>,
    stats: Arc<RwLock<BridgeStats>>,
    data_points: Arc<RwLock<HashMap<String, ModbusDataPoint>>>,
    storage: Arc<Mutex<ModbusStorage>>,
    shutdown_sender: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl ModbusBridge {
    /// 创建新的Modbus桥接
    pub fn new(config: ModbusConfig) -> Result<Self> {
        let storage = ModbusStorage::new(&config);
        
        Ok(Self {
            config,
            state: Arc::new(RwLock::new(BridgeState::Stopped)),
            stats: Arc::new(RwLock::new(BridgeStats::default())),
            data_points: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(Mutex::new(storage)),
            shutdown_sender: Arc::new(Mutex::new(None)),
        })
    }

    /// 确定数据点的Modbus映射
    fn determine_modbus_mapping(&self, data_point: &DataPoint) -> Result<(ModbusRegion, u16, u16)> {
        // 根据数据类型和访问权限确定映射
        match (&data_point.data_type, &data_point.access) {
            (DataType::Boolean, AccessLevel::ReadWrite) => {
                // 布尔值读写 -> 线圈
                let address = self.find_available_address(ModbusRegion::Coils, 1)?;
                Ok((ModbusRegion::Coils, address, 1))
            }
            (DataType::Boolean, AccessLevel::ReadOnly) => {
                // 布尔值只读 -> 离散输入
                let address = self.find_available_address(ModbusRegion::DiscreteInputs, 1)?;
                Ok((ModbusRegion::DiscreteInputs, address, 1))
            }
            (_, AccessLevel::ReadWrite) => {
                // 数值读写 -> 保持寄存器
                let length = self.get_register_length(&data_point.data_type);
                let address = self.find_available_address(ModbusRegion::HoldingRegisters, length)?;
                Ok((ModbusRegion::HoldingRegisters, address, length))
            }
            (_, AccessLevel::ReadOnly) => {
                // 数值只读 -> 输入寄存器
                let length = self.get_register_length(&data_point.data_type);
                let address = self.find_available_address(ModbusRegion::InputRegisters, length)?;
                Ok((ModbusRegion::InputRegisters, address, length))
            }
            (_, AccessLevel::WriteOnly) => {
                // 数值只写 -> 保持寄存器
                let length = self.get_register_length(&data_point.data_type);
                let address = self.find_available_address(ModbusRegion::HoldingRegisters, length)?;
                Ok((ModbusRegion::HoldingRegisters, address, length))
            }
        }
    }

    /// 获取数据类型对应的寄存器长度
    fn get_register_length(&self, data_type: &DataType) -> u16 {
        match data_type {
            DataType::Boolean => 1,
            DataType::Int16 | DataType::UInt16 => 1,
            DataType::Int32 | DataType::UInt32 | DataType::Float => 2,
            DataType::Int64 | DataType::UInt64 | DataType::Double => 4,
            DataType::String | DataType::ByteArray => 10, // 默认10个寄存器
            DataType::DateTime => 4,
        }
    }

    /// 查找可用的地址
    fn find_available_address(&self, region: ModbusRegion, length: u16) -> Result<u16> {
        let data_points = self.data_points.read().unwrap();
        
        let (start, count) = match region {
            ModbusRegion::Coils => (self.config.coils_start, self.config.coils_count),
            ModbusRegion::DiscreteInputs => (self.config.discrete_inputs_start, self.config.discrete_inputs_count),
            ModbusRegion::HoldingRegisters => (self.config.holding_registers_start, self.config.holding_registers_count),
            ModbusRegion::InputRegisters => (self.config.input_registers_start, self.config.input_registers_count),
        };

        // 简单的线性搜索，实际实现中可以优化
        for addr in start..(start + count - length + 1) {
            let mut available = true;
            
            for dp in data_points.values() {
                if dp.region == region {
                    let dp_end = dp.address + dp.length;
                    let check_end = addr + length;
                    
                    // 检查地址范围是否重叠
                    if !(check_end <= dp.address || addr >= dp_end) {
                        available = false;
                        break;
                    }
                }
            }
            
            if available {
                return Ok(addr);
            }
        }
        
        Err(BridgeError::modbus("No available address in region"))
    }

    /// 将内部数据值转换为Modbus存储
    async fn data_value_to_modbus(&self, value: &DataValue, mapping: &ModbusDataPoint) -> Result<()> {
        let mut storage = self.storage.lock().await;
        
        match &mapping.region {
            ModbusRegion::Coils => {
                if let DataValue::Boolean(b) = value {
                    storage.write_coils(mapping.address, &[*b])?;
                } else {
                    return Err(BridgeError::modbus("Invalid value type for coil"));
                }
            }
            ModbusRegion::DiscreteInputs => {
                if let DataValue::Boolean(b) = value {
                    storage.write_discrete_inputs(mapping.address, &[*b])?;
                } else {
                    return Err(BridgeError::modbus("Invalid value type for discrete input"));
                }
            }
            ModbusRegion::HoldingRegisters | ModbusRegion::InputRegisters => {
                let registers = self.value_to_registers(value, mapping.length)?;
                
                match mapping.region {
                    ModbusRegion::HoldingRegisters => {
                        storage.write_holding_registers(mapping.address, &registers)?;
                    }
                    ModbusRegion::InputRegisters => {
                        storage.write_input_registers(mapping.address, &registers)?;
                    }
                    _ => unreachable!(),
                }
            }
        }
        
        Ok(())
    }

    /// 将数据值转换为寄存器数组
    fn value_to_registers(&self, value: &DataValue, length: u16) -> Result<Vec<u16>> {
        let mut registers = vec![0u16; length as usize];
        
        match value {
            DataValue::Int16(v) => {
                registers[0] = *v as u16;
            }
            DataValue::UInt16(v) => {
                registers[0] = *v;
            }
            DataValue::Int32(v) => {
                let bytes = v.to_be_bytes();
                registers[0] = u16::from_be_bytes([bytes[0], bytes[1]]);
                registers[1] = u16::from_be_bytes([bytes[2], bytes[3]]);
            }
            DataValue::UInt32(v) => {
                let bytes = v.to_be_bytes();
                registers[0] = u16::from_be_bytes([bytes[0], bytes[1]]);
                registers[1] = u16::from_be_bytes([bytes[2], bytes[3]]);
            }
            DataValue::Float(v) => {
                let bytes = v.to_be_bytes();
                registers[0] = u16::from_be_bytes([bytes[0], bytes[1]]);
                registers[1] = u16::from_be_bytes([bytes[2], bytes[3]]);
            }
            DataValue::Int64(v) => {
                let bytes = v.to_be_bytes();
                registers[0] = u16::from_be_bytes([bytes[0], bytes[1]]);
                registers[1] = u16::from_be_bytes([bytes[2], bytes[3]]);
                registers[2] = u16::from_be_bytes([bytes[4], bytes[5]]);
                registers[3] = u16::from_be_bytes([bytes[6], bytes[7]]);
            }
            DataValue::UInt64(v) => {
                let bytes = v.to_be_bytes();
                registers[0] = u16::from_be_bytes([bytes[0], bytes[1]]);
                registers[1] = u16::from_be_bytes([bytes[2], bytes[3]]);
                registers[2] = u16::from_be_bytes([bytes[4], bytes[5]]);
                registers[3] = u16::from_be_bytes([bytes[6], bytes[7]]);
            }
            DataValue::Double(v) => {
                let bytes = v.to_be_bytes();
                registers[0] = u16::from_be_bytes([bytes[0], bytes[1]]);
                registers[1] = u16::from_be_bytes([bytes[2], bytes[3]]);
                registers[2] = u16::from_be_bytes([bytes[4], bytes[5]]);
                registers[3] = u16::from_be_bytes([bytes[6], bytes[7]]);
            }
            DataValue::String(s) => {
                let bytes = s.as_bytes();
                for (i, chunk) in bytes.chunks(2).enumerate() {
                    if i >= registers.len() {
                        break;
                    }
                    
                    let high = chunk.get(0).copied().unwrap_or(0);
                    let low = chunk.get(1).copied().unwrap_or(0);
                    registers[i] = u16::from_be_bytes([high, low]);
                }
            }
            DataValue::ByteArray(bytes) => {
                for (i, chunk) in bytes.chunks(2).enumerate() {
                    if i >= registers.len() {
                        break;
                    }
                    
                    let high = chunk.get(0).copied().unwrap_or(0);
                    let low = chunk.get(1).copied().unwrap_or(0);
                    registers[i] = u16::from_be_bytes([high, low]);
                }
            }
            DataValue::DateTime(dt) => {
                let timestamp = dt.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                let bytes = timestamp.to_be_bytes();
                registers[0] = u16::from_be_bytes([bytes[0], bytes[1]]);
                registers[1] = u16::from_be_bytes([bytes[2], bytes[3]]);
                registers[2] = u16::from_be_bytes([bytes[4], bytes[5]]);
                registers[3] = u16::from_be_bytes([bytes[6], bytes[7]]);
            }
            _ => {
                return Err(BridgeError::modbus("Unsupported value type"));
            }
        }
        
        Ok(registers)
    }

    /// 从寄存器数组转换为数据值
    fn registers_to_value(&self, registers: &[u16], data_type: &DataType) -> Result<DataValue> {
        match data_type {
            DataType::Int16 => {
                if registers.is_empty() {
                    return Err(BridgeError::modbus("Not enough registers for Int16"));
                }
                Ok(DataValue::Int16(registers[0] as i16))
            }
            DataType::UInt16 => {
                if registers.is_empty() {
                    return Err(BridgeError::modbus("Not enough registers for UInt16"));
                }
                Ok(DataValue::UInt16(registers[0]))
            }
            DataType::Int32 => {
                if registers.len() < 2 {
                    return Err(BridgeError::modbus("Not enough registers for Int32"));
                }
                let bytes = [
                    (registers[0] >> 8) as u8, (registers[0] & 0xFF) as u8,
                    (registers[1] >> 8) as u8, (registers[1] & 0xFF) as u8,
                ];
                Ok(DataValue::Int32(i32::from_be_bytes(bytes)))
            }
            DataType::UInt32 => {
                if registers.len() < 2 {
                    return Err(BridgeError::modbus("Not enough registers for UInt32"));
                }
                let bytes = [
                    (registers[0] >> 8) as u8, (registers[0] & 0xFF) as u8,
                    (registers[1] >> 8) as u8, (registers[1] & 0xFF) as u8,
                ];
                Ok(DataValue::UInt32(u32::from_be_bytes(bytes)))
            }
            DataType::Float => {
                if registers.len() < 2 {
                    return Err(BridgeError::modbus("Not enough registers for Float"));
                }
                let bytes = [
                    (registers[0] >> 8) as u8, (registers[0] & 0xFF) as u8,
                    (registers[1] >> 8) as u8, (registers[1] & 0xFF) as u8,
                ];
                Ok(DataValue::Float(f32::from_be_bytes(bytes)))
            }
            _ => {
                Err(BridgeError::modbus("Unsupported data type for register conversion"))
            }
        }
    }

    /// 启动Modbus TCP服务器
    async fn start_tcp_server(&self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.base.bind_address, self.config.base.port)
            .parse()
            .map_err(|_| BridgeError::config("Invalid bind address"))?;

        let listener = TcpListener::bind(addr).await
            .map_err(|e| BridgeError::modbus(format!("Failed to bind TCP listener: {}", e)))?;

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        {
            let mut sender = self.shutdown_sender.lock().await;
            *sender = Some(shutdown_tx);
        }

        let storage = self.storage.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            tracing::info!("Modbus TCP server listening on {}", addr);

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, client_addr)) => {
                                tracing::debug!("New Modbus client connected: {}", client_addr);
                                
                                // 更新统计信息
                                {
                                    let mut stats = stats.write().unwrap();
                                    stats.connections += 1;
                                    stats.last_activity = Some(SystemTime::now());
                                }

                                // 在实际实现中，这里应该处理Modbus协议
                                // 由于tokio-modbus主要是客户端库，这里简化实现
                                tokio::spawn(async move {
                                    // 处理客户端连接
                                    tracing::debug!("Handling Modbus client: {}", client_addr);
                                    // TODO: 实现Modbus协议处理
                                });
                            }
                            Err(e) => {
                                tracing::error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Modbus TCP server shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl ProtocolBridge for ModbusBridge {
    fn config(&self) -> &BridgeConfig {
        &self.config.base
    }

    async fn state(&self) -> BridgeState {
        self.state.read().unwrap().clone()
    }

    async fn stats(&self) -> BridgeStats {
        self.stats.read().unwrap().clone()
    }

    async fn start(&self) -> Result<()> {
        {
            let mut state = self.state.write().unwrap();
            if *state == BridgeState::Running {
                return Ok(());
            }
            *state = BridgeState::Starting;
        }

        // 启动相应的传输层
        match self.config.transport {
            ModbusTransport::Tcp => {
                self.start_tcp_server().await?;
            }
            ModbusTransport::Rtu | ModbusTransport::Ascii => {
                // TODO: 实现串口支持
                return Err(BridgeError::modbus("Serial transport not yet implemented"));
            }
        }

        // 更新状态
        {
            let mut state = self.state.write().unwrap();
            *state = BridgeState::Running;
        }

        {
            let mut stats = self.stats.write().unwrap();
            stats.start_time = Some(SystemTime::now());
        }

        tracing::info!("Modbus bridge started on {}:{}", 
            self.config.base.bind_address, 
            self.config.base.port
        );
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        {
            let mut state = self.state.write().unwrap();
            if *state == BridgeState::Stopped {
                return Ok(());
            }
            *state = BridgeState::Stopping;
        }

        // 发送关闭信号
        if let Some(sender) = self.shutdown_sender.lock().await.as_ref() {
            let _ = sender.send(());
        }

        // 更新状态
        {
            let mut state = self.state.write().unwrap();
            *state = BridgeState::Stopped;
        }

        tracing::info!("Modbus bridge stopped");
        Ok(())
    }

    async fn add_data_point(&self, data_point: DataPoint) -> Result<()> {
        // 确定Modbus映射
        let (region, address, length) = self.determine_modbus_mapping(&data_point)?;

        let modbus_data_point = ModbusDataPoint {
            data_point: data_point.clone(),
            region,
            address,
            length,
        };

        // 如果有初始值，写入存储
        if let Some(ref value) = data_point.value {
            self.data_value_to_modbus(value, &modbus_data_point).await?;
        }

        // 存储映射
        {
            let mut data_points = self.data_points.write().unwrap();
            data_points.insert(data_point.id.clone(), modbus_data_point);
        }

        tracing::debug!("Added Modbus data point: {} -> {:?}:{}", 
            data_point.id, region, address);
        Ok(())
    }

    async fn remove_data_point(&self, data_point_id: &str) -> Result<()> {
        let mut data_points = self.data_points.write().unwrap();
        if data_points.remove(data_point_id).is_some() {
            tracing::debug!("Removed Modbus data point: {}", data_point_id);
            Ok(())
        } else {
            Err(BridgeError::BridgeNotFound(format!("Data point not found: {}", data_point_id)))
        }
    }

    async fn get_data_point(&self, data_point_id: &str) -> Result<Option<DataPoint>> {
        let data_points = self.data_points.read().unwrap();
        Ok(data_points.get(data_point_id).map(|dp| dp.data_point.clone()))
    }

    async fn list_data_points(&self) -> Result<Vec<DataPoint>> {
        let data_points = self.data_points.read().unwrap();
        Ok(data_points.values().map(|dp| dp.data_point.clone()).collect())
    }

    async fn read_value(&self, data_point_id: &str) -> Result<Option<DataValue>> {
        let data_points = self.data_points.read().unwrap();
        Ok(data_points.get(data_point_id).and_then(|dp| dp.data_point.value.clone()))
    }

    async fn write_value(&self, data_point_id: &str, value: DataValue) -> Result<()> {
        let mapping = {
            let data_points = self.data_points.read().unwrap();
            data_points.get(data_point_id).cloned()
                .ok_or_else(|| BridgeError::BridgeNotFound(format!("Data point not found: {}", data_point_id)))?
        };

        // 写入Modbus存储
        self.data_value_to_modbus(&value, &mapping).await?;

        // 更新数据点值
        {
            let mut data_points = self.data_points.write().unwrap();
            if let Some(dp) = data_points.get_mut(data_point_id) {
                dp.data_point.value = Some(value);
                dp.data_point.last_updated = Some(SystemTime::now());
            }
        }

        // 更新统计信息
        {
            let mut stats = self.stats.write().unwrap();
            stats.last_activity = Some(SystemTime::now());
        }

        Ok(())
    }

    async fn read_multiple(&self, data_point_ids: &[String]) -> Result<HashMap<String, Option<DataValue>>> {
        let mut result = HashMap::new();
        for id in data_point_ids {
            let value = self.read_value(id).await?;
            result.insert(id.clone(), value);
        }
        Ok(result)
    }

    async fn write_multiple(&self, values: HashMap<String, DataValue>) -> Result<()> {
        for (id, value) in values {
            self.write_value(&id, value).await?;
        }
        Ok(())
    }

    async fn subscribe(&self, _data_point_ids: &[String]) -> Result<String> {
        // Modbus通常是轮询协议，不支持真正的订阅
        // 这里返回一个虚拟的订阅ID
        let subscription_id = uuid::Uuid::new_v4().to_string();
        tracing::debug!("Created Modbus subscription: {}", subscription_id);
        Ok(subscription_id)
    }

    async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        tracing::debug!("Cancelled Modbus subscription: {}", subscription_id);
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(matches!(self.state().await, BridgeState::Running))
    }

    async fn info(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut info = HashMap::new();
        info.insert("bridge_type".to_string(), serde_json::Value::String("modbus-slave".to_string()));
        info.insert("slave_id".to_string(), serde_json::Value::Number(self.config.slave_id.into()));
        info.insert("transport".to_string(), serde_json::Value::String(format!("{:?}", self.config.transport)));
        info.insert("data_points_count".to_string(), serde_json::Value::Number(self.data_points.read().unwrap().len().into()));
        
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_modbus_bridge_creation() {
        let config = ModbusConfig::default();
        let bridge = ModbusBridge::new(config).unwrap();
        
        assert_eq!(bridge.state().await, BridgeState::Stopped);
        assert!(bridge.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_data_point_mapping() {
        let bridge = ModbusBridge::new(ModbusConfig::default()).unwrap();

        let data_point = DataPoint {
            id: "test.bool.1".to_string(),
            name: "Test Boolean".to_string(),
            data_type: DataType::Boolean,
            access: AccessLevel::ReadWrite,
            value: Some(DataValue::Boolean(true)),
            last_updated: Some(SystemTime::now()),
            quality: Quality::Good,
        };

        let (region, address, length) = bridge.determine_modbus_mapping(&data_point).unwrap();
        assert_eq!(region, ModbusRegion::Coils);
        assert_eq!(length, 1);
    }

    #[test]
    fn test_value_conversion() {
        let bridge = ModbusBridge::new(ModbusConfig::default()).unwrap();
        
        let value = DataValue::Int32(0x12345678);
        let registers = bridge.value_to_registers(&value, 2).unwrap();
        assert_eq!(registers.len(), 2);
        
        let converted = bridge.registers_to_value(&registers, &DataType::Int32).unwrap();
        match converted {
            DataValue::Int32(v) => assert_eq!(v, 0x12345678),
            _ => panic!("Unexpected value type"),
        }
    }

    #[test]
    fn test_modbus_storage() {
        let config = ModbusConfig::default();
        let mut storage = ModbusStorage::new(&config);
        
        // 测试线圈操作
        storage.write_coils(0, &[true, false, true]).unwrap();
        let coils = storage.read_coils(0, 3).unwrap();
        assert_eq!(coils, vec![true, false, true]);
        
        // 测试寄存器操作
        storage.write_holding_registers(0, &[0x1234, 0x5678]).unwrap();
        let registers = storage.read_holding_registers(0, 2).unwrap();
        assert_eq!(registers, vec![0x1234, 0x5678]);
    }
}