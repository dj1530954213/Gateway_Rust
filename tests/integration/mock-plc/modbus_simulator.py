#!/usr/bin/env python3
"""
Modbus TCP服务器模拟器 - 用于集成测试

模拟一个完整的PLC设备，支持：
- 读写保持寄存器
- 读写输入寄存器  
- 读写线圈
- 读写离散输入
- 动态数据生成
- 故障注入
"""

import time
import math
import random
import logging
import threading
from typing import Dict, Any
from socketserver import TCPServer
from pymodbus.server.sync import StartTcpServer
from pymodbus.device import ModbusDeviceIdentification
from pymodbus.datastore import ModbusSequentialDataBlock, ModbusSlaveContext, ModbusServerContext
from pymodbus.transaction import ModbusRtuFramer, ModbusSocketFramer

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class DynamicDataGenerator:
    """动态数据生成器"""
    
    def __init__(self):
        self.start_time = time.time()
        self.data_patterns = {
            'sine_wave': self._sine_wave,
            'random_walk': self._random_walk,
            'step_function': self._step_function,
            'linear_ramp': self._linear_ramp,
        }
        self.random_state = {}
    
    def _sine_wave(self, address: int, amplitude: float = 1000.0, period: float = 60.0) -> int:
        """正弦波数据"""
        t = time.time() - self.start_time
        value = amplitude * math.sin(2 * math.pi * t / period) + amplitude
        return int(value)
    
    def _random_walk(self, address: int, base: float = 500.0, step: float = 50.0) -> int:
        """随机游走数据"""
        if address not in self.random_state:
            self.random_state[address] = base
        
        change = random.uniform(-step, step)
        self.random_state[address] += change
        
        # 限制范围
        self.random_state[address] = max(0, min(65535, self.random_state[address]))
        return int(self.random_state[address])
    
    def _step_function(self, address: int, low: float = 100.0, high: float = 900.0, period: float = 30.0) -> int:
        """阶跃函数数据"""
        t = time.time() - self.start_time
        cycle_pos = (t % period) / period
        return int(high if cycle_pos > 0.5 else low)
    
    def _linear_ramp(self, address: int, min_val: float = 0.0, max_val: float = 1000.0, period: float = 120.0) -> int:
        """线性斜坡数据"""
        t = time.time() - self.start_time
        cycle_pos = (t % period) / period
        value = min_val + (max_val - min_val) * cycle_pos
        return int(value)

class MockPLCDataStore:
    """模拟PLC数据存储"""
    
    def __init__(self):
        self.generator = DynamicDataGenerator()
        self.static_data = {}
        self.fault_injection = {
            'enabled': False,
            'error_rate': 0.0,
            'response_delay': 0.0,
        }
        
        # 初始化测试数据
        self._initialize_test_data()
    
    def _initialize_test_data(self):
        """初始化测试数据模式"""
        # 温度传感器 (40001-40010) - 正弦波
        for addr in range(40001, 40011):
            self.static_data[addr] = {
                'pattern': 'sine_wave',
                'params': {'amplitude': 50, 'period': 60}
            }
        
        # 压力传感器 (40011-40020) - 随机游走
        for addr in range(40011, 40021):
            self.static_data[addr] = {
                'pattern': 'random_walk',
                'params': {'base': 750, 'step': 30}
            }
        
        # 流量传感器 (40021-40030) - 阶跃函数
        for addr in range(40021, 40031):
            self.static_data[addr] = {
                'pattern': 'step_function',
                'params': {'low': 200, 'high': 800, 'period': 45}
            }
        
        # 液位传感器 (40031-40040) - 线性斜坡
        for addr in range(40031, 40041):
            self.static_data[addr] = {
                'pattern': 'linear_ramp',
                'params': {'min_val': 100, 'max_val': 900, 'period': 90}
            }
    
    def get_dynamic_value(self, address: int) -> int:
        """获取动态生成的值"""
        if address in self.static_data:
            pattern_name = self.static_data[address]['pattern']
            params = self.static_data[address]['params']
            pattern_func = self.generator.data_patterns[pattern_name]
            return pattern_func(address, **params)
        else:
            # 默认返回地址本身作为值
            return address % 65536
    
    def inject_fault(self, error_rate: float = 0.1, response_delay: float = 0.0):
        """注入故障"""
        self.fault_injection = {
            'enabled': True,
            'error_rate': error_rate,
            'response_delay': response_delay,
        }
        logger.info(f"Fault injection enabled: error_rate={error_rate}, delay={response_delay}s")
    
    def clear_faults(self):
        """清除故障注入"""
        self.fault_injection['enabled'] = False
        logger.info("Fault injection disabled")
    
    def should_inject_error(self) -> bool:
        """判断是否应该注入错误"""
        if not self.fault_injection['enabled']:
            return False
        return random.random() < self.fault_injection['error_rate']
    
    def get_response_delay(self) -> float:
        """获取响应延迟"""
        if not self.fault_injection['enabled']:
            return 0.0
        return self.fault_injection['response_delay']

class DynamicModbusDataBlock(ModbusSequentialDataBlock):
    """支持动态数据的Modbus数据块"""
    
    def __init__(self, address, values, data_store: MockPLCDataStore):
        super().__init__(address, values)
        self.data_store = data_store
        self.last_update = time.time()
        self.update_interval = 1.0  # 每秒更新一次动态数据
    
    def getValues(self, address, count=1):
        """获取值，支持动态生成"""
        # 检查是否需要更新动态数据
        current_time = time.time()
        if current_time - self.last_update > self.update_interval:
            self._update_dynamic_values()
            self.last_update = current_time
        
        # 故障注入 - 响应延迟
        delay = self.data_store.get_response_delay()
        if delay > 0:
            time.sleep(delay)
        
        # 故障注入 - 错误响应
        if self.data_store.should_inject_error():
            logger.warning(f"Injecting error for address {address}")
            raise Exception("Simulated Modbus error")
        
        return super().getValues(address, count)
    
    def _update_dynamic_values(self):
        """更新动态值"""
        for addr in range(self.address, self.address + len(self.values)):
            dynamic_value = self.data_store.get_dynamic_value(addr)
            self.values[addr - self.address] = dynamic_value

def setup_server_context(data_store: MockPLCDataStore) -> ModbusServerContext:
    """设置服务器上下文"""
    
    # 保持寄存器 (Function Code 3 & 6 & 16)
    holding_registers = DynamicModbusDataBlock(40001, [0] * 100, data_store)
    
    # 输入寄存器 (Function Code 4)
    input_registers = DynamicModbusDataBlock(30001, [0] * 100, data_store)
    
    # 线圈 (Function Code 1 & 5 & 15)
    coils = ModbusSequentialDataBlock(1, [False] * 100)
    
    # 离散输入 (Function Code 2)
    discrete_inputs = ModbusSequentialDataBlock(10001, [False] * 100)
    
    # 创建从站上下文
    slave_context = ModbusSlaveContext(
        di=discrete_inputs,  # 离散输入
        co=coils,           # 线圈
        hr=holding_registers, # 保持寄存器
        ir=input_registers   # 输入寄存器
    )
    
    # 创建服务器上下文 (支持多个从站)
    context = ModbusServerContext(slaves={1: slave_context}, single=False)
    
    return context

def setup_device_identification() -> ModbusDeviceIdentification:
    """设置设备标识"""
    identity = ModbusDeviceIdentification()
    identity.VendorName = 'Gateway Test'
    identity.ProductCode = 'MOCK-PLC'
    identity.VendorUrl = 'https://github.com/edge-gateway'
    identity.ProductName = 'Mock PLC Simulator'
    identity.ModelName = 'Test PLC v1.0'
    identity.MajorMinorRevision = '1.0.0'
    return identity

def control_server(data_store: MockPLCDataStore):
    """控制服务器 - 接收故障注入命令"""
    import http.server
    import socketserver
    import json
    from urllib.parse import urlparse, parse_qs
    
    class ControlHandler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            """处理GET请求 - 获取状态"""
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            
            status = {
                'fault_injection': data_store.fault_injection,
                'timestamp': time.time(),
                'uptime': time.time() - data_store.generator.start_time
            }
            
            self.wfile.write(json.dumps(status).encode())
        
        def do_POST(self):
            """处理POST请求 - 故障注入控制"""
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            
            try:
                data = json.loads(post_data.decode('utf-8'))
                command = data.get('command')
                
                if command == 'inject_fault':
                    error_rate = data.get('error_rate', 0.1)
                    delay = data.get('response_delay', 0.0)
                    data_store.inject_fault(error_rate, delay)
                    response = {'status': 'success', 'message': 'Fault injection enabled'}
                
                elif command == 'clear_faults':
                    data_store.clear_faults()
                    response = {'status': 'success', 'message': 'Fault injection disabled'}
                
                else:
                    response = {'status': 'error', 'message': f'Unknown command: {command}'}
                
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                self.wfile.write(json.dumps(response).encode())
                
            except Exception as e:
                self.send_response(400)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                error_response = {'status': 'error', 'message': str(e)}
                self.wfile.write(json.dumps(error_response).encode())
        
        def log_message(self, format, *args):
            # 禁用HTTP日志输出
            pass
    
    with socketserver.TCPServer(("", 8080), ControlHandler) as httpd:
        logger.info("Control server started on port 8080")
        httpd.serve_forever()

def main():
    """主函数"""
    logger.info("Starting Mock PLC Simulator...")
    
    # 创建数据存储
    data_store = MockPLCDataStore()
    
    # 设置服务器上下文
    context = setup_server_context(data_store)
    
    # 设置设备标识
    identity = setup_device_identification()
    
    # 启动控制服务器线程
    control_thread = threading.Thread(
        target=control_server,
        args=(data_store,),
        daemon=True
    )
    control_thread.start()
    
    logger.info("Mock PLC ready - Modbus TCP on port 502, Control API on port 8080")
    
    # 启动Modbus TCP服务器
    try:
        StartTcpServer(
            context,
            identity=identity,
            address=("0.0.0.0", 502),
            framer=ModbusSocketFramer
        )
    except KeyboardInterrupt:
        logger.info("Shutting down Mock PLC Simulator...")
    except Exception as e:
        logger.error(f"Server error: {e}")

if __name__ == "__main__":
    main()