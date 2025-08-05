#!/usr/bin/env python3
"""
测试Modbus TCP连接到127.0.0.1:502
验证数据读取是否正常
"""

import socket
import struct
import time
import sys

def create_modbus_request(unit_id=1, function_code=3, start_address=0, quantity=1):
    """创建Modbus TCP读保持寄存器请求"""
    # Modbus TCP MBAP Header
    transaction_id = 0x0001
    protocol_id = 0x0000  
    length = 6  # 后续字节数
    
    # Modbus PDU
    pdu = struct.pack('>BBH', function_code, start_address, quantity)
    
    # 完整请求
    mbap = struct.pack('>HHHB', transaction_id, protocol_id, length, unit_id)
    return mbap + pdu

def parse_modbus_response(response):
    """解析Modbus TCP响应"""
    if len(response) < 9:
        return None, "响应太短"
    
    # 解析MBAP头
    trans_id, proto_id, length, unit_id = struct.unpack('>HHHB', response[:7])
    
    # 解析PDU
    func_code = response[7]
    
    if func_code >= 0x80:
        error_code = response[8]
        return None, f"Modbus错误: 功能码{func_code}, 错误码{error_code}"
    
    if func_code == 3:  # 读保持寄存器
        byte_count = response[8]
        if len(response) < 9 + byte_count:
            return None, "数据不完整"
        
        data = response[9:9+byte_count]
        # 解析为16位寄存器值
        values = []
        for i in range(0, len(data), 2):
            if i + 1 < len(data):
                value = struct.unpack('>H', data[i:i+2])[0]
                values.append(value)
        
        return values, "成功"
    
    return None, f"未知功能码: {func_code}"

def test_modbus_connection():
    """测试Modbus TCP连接"""
    host = "127.0.0.1"
    port = 502
    
    print(f"[TEST] Modbus TCP连接测试: {host}:{port}")
    
    try:
        # 创建TCP连接
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5.0)
        sock.connect((host, port))
        print("[OK] TCP连接成功")
        
        # 测试读取保持寄存器 40001-40005
        test_addresses = [0, 1, 2, 3, 4]  # Modbus地址0-4 (对应40001-40005)
        
        for addr in test_addresses:
            print(f"\n[READ] 读取寄存器 4000{addr+1} (地址{addr})...")
            
            # 创建请求
            request = create_modbus_request(unit_id=1, function_code=3, start_address=addr, quantity=1)
            
            # 发送请求
            sock.send(request)
            
            # 接收响应
            response = sock.recv(1024)
            
            # 解析响应
            values, message = parse_modbus_response(response)
            
            if values is not None:
                print(f"[SUCCESS] 读取成功: 寄存器4000{addr+1} = {values[0]} (0x{values[0]:04X})")
            else:
                print(f"[ERROR] 读取失败: {message}")
        
        sock.close()
        print("\n[DONE] Modbus TCP测试完成")
        return True
        
    except socket.timeout:
        print("[ERROR] 连接超时")
        return False
    except ConnectionRefusedError:
        print("[ERROR] 连接被拒绝")
        return False
    except Exception as e:
        print(f"[ERROR] 连接错误: {e}")
        return False

if __name__ == "__main__":
    success = test_modbus_connection()
    sys.exit(0 if success else 1)