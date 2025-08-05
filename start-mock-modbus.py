#!/usr/bin/env python3
"""
简单的Mock Modbus TCP服务器 - 使用socket直接实现
避免pymodbus版本兼容性问题
"""

import socket
import threading
import time
import struct
import math

class MockModbusServer:
    def __init__(self, host='127.0.0.1', port=502):
        self.host = host
        self.port = port
        self.running = False
        self.start_time = time.time()
        
        # 模拟寄存器数据 (40001-40020)
        self.holding_registers = [0] * 20
        
        # 初始化测试数据
        self.holding_registers[0] = 123   # 40001 - 温度1
        self.holding_registers[1] = 456   # 40002 - 压力1  
        self.holding_registers[2] = 789   # 40003 - 流量1
        self.holding_registers[3] = 234   # 40004 - 温度2
        self.holding_registers[4] = 567   # 40005 - 压力2
        self.holding_registers[5] = 890   # 40006 - 流量2
        
    def update_dynamic_data(self):
        """更新动态数据"""
        elapsed = time.time() - self.start_time
        
        # 模拟传感器数据变化
        self.holding_registers[0] = int((25 + 5 * math.sin(elapsed * 0.1)) * 10)    # 温度1
        self.holding_registers[1] = int((5 + 1 * math.cos(elapsed * 0.15)) * 100)   # 压力1
        self.holding_registers[2] = int((80 + 10 * math.sin(elapsed * 0.2)) * 10)   # 流量1
        self.holding_registers[3] = int((20 + 5 * math.cos(elapsed * 0.12)) * 10)   # 温度2
        self.holding_registers[4] = int((5 + 2 * math.sin(elapsed * 0.18)) * 100)   # 压力2  
        self.holding_registers[5] = int((80 + 20 * math.cos(elapsed * 0.25)) * 10)  # 流量2
        
    def parse_modbus_request(self, data):
        """解析Modbus TCP请求"""
        if len(data) < 12:
            return None
            
        # Modbus TCP ADU格式: Transaction ID(2) + Protocol ID(2) + Length(2) + Unit ID(1) + PDU
        transaction_id = struct.unpack('>H', data[0:2])[0]
        protocol_id = struct.unpack('>H', data[2:4])[0] 
        length = struct.unpack('>H', data[4:6])[0]
        unit_id = data[6]
        
        if protocol_id != 0:  # Modbus protocol
            return None
            
        # PDU: Function Code(1) + Data
        function_code = data[7]
        
        if function_code == 0x03:  # Read Holding Registers
            if len(data) < 12:
                return None
            start_addr = struct.unpack('>H', data[8:10])[0]
            quantity = struct.unpack('>H', data[10:12])[0]
            
            return {
                'transaction_id': transaction_id,
                'unit_id': unit_id,
                'function_code': function_code,
                'start_addr': start_addr,
                'quantity': quantity
            }
            
        return None
        
    def create_modbus_response(self, request):
        """创建Modbus TCP响应"""
        if request['function_code'] == 0x03:  # Read Holding Registers
            start_addr = request['start_addr']
            quantity = request['quantity']
            
            # 检查地址范围
            if start_addr >= len(self.holding_registers) or start_addr + quantity > len(self.holding_registers):
                # 返回异常响应
                response_data = struct.pack('>HHHBB', 
                    request['transaction_id'],  # Transaction ID
                    0,                          # Protocol ID
                    3,                          # Length
                    request['unit_id'],         # Unit ID
                    0x83,                       # Function code + 0x80 (error)
                    0x02                        # Exception code: Illegal Data Address
                )
                return response_data
                
            # 正常响应
            byte_count = quantity * 2
            response_pdu = struct.pack('>BB', 0x03, byte_count)  # Function code + byte count
            
            # 添加寄存器数据
            for i in range(quantity):
                addr = start_addr + i
                value = self.holding_registers[addr] if addr < len(self.holding_registers) else 0
                response_pdu += struct.pack('>H', value)
                
            # 构建完整的TCP响应
            response_data = struct.pack('>HHH', 
                request['transaction_id'],  # Transaction ID
                0,                          # Protocol ID  
                len(response_pdu) + 1       # Length (PDU + Unit ID)
            )
            response_data += struct.pack('B', request['unit_id'])  # Unit ID
            response_data += response_pdu
            
            return response_data
            
        return None
        
    def handle_client(self, client_socket, addr):
        """处理客户端连接"""
        print(f"客户端连接: {addr}")
        
        try:
            while self.running:
                data = client_socket.recv(1024)
                if not data:
                    break
                    
                request = self.parse_modbus_request(data)
                if request:
                    print(f"收到请求: 功能码{request['function_code']}, 地址{request['start_addr']}, 数量{request['quantity']}")
                    
                    # 更新动态数据
                    self.update_dynamic_data()
                    
                    response = self.create_modbus_response(request)
                    if response:
                        client_socket.send(response)
                        print(f"发送响应: {len(response)}字节")
                        
        except Exception as e:
            print(f"客户端处理错误: {e}")
        finally:
            client_socket.close()
            print(f"客户端断开: {addr}")
            
    def start(self):
        """启动服务器"""
        self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        
        try:
            print(f"尝试绑定到 {self.host}:{self.port}...")
            self.server_socket.bind((self.host, self.port))
            print("绑定成功!")
            
            self.server_socket.listen(5)
            print("开始监听连接...")
            self.running = True
            
            print(f"Mock Modbus TCP服务器启动成功")
            print(f"监听地址: {self.host}:{self.port}")
            print(f"单元ID: 1")
            print(f"寄存器: 40001-40011 (地址0-10)")
            print("按Ctrl+C停止服务器")
            print("-" * 40)
            
            while self.running:
                try:
                    client_socket, addr = self.server_socket.accept()
                    client_thread = threading.Thread(
                        target=self.handle_client, 
                        args=(client_socket, addr)
                    )
                    client_thread.daemon = True
                    client_thread.start()
                except socket.error:
                    if self.running:
                        print("接受连接时出错")
                    break
                    
        except KeyboardInterrupt:
            print("\n收到停止信号...")
        except Exception as e:
            print(f"服务器错误: {e}")
        finally:
            self.stop()
            
    def stop(self):
        """停止服务器"""
        self.running = False
        if hasattr(self, 'server_socket'):
            self.server_socket.close()
        print("Mock Modbus服务器已停止")

if __name__ == "__main__":
    server = MockModbusServer()
    server.start()