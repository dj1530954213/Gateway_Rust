#!/usr/bin/env python3
"""
Modbus TCP客户端测试脚本
用于验证127.0.0.1:502的连接
"""

import asyncio
from pymodbus.client import AsyncModbusTcpClient

async def test_modbus_connection():
    print("测试Modbus TCP连接...")
    print("目标: 127.0.0.1:502")
    
    client = AsyncModbusTcpClient(host='127.0.0.1', port=502)
    
    try:
        # 尝试连接
        print("尝试连接...")
        connected = await client.connect()
        
        if not connected:
            print("❌ 连接失败")
            return False
            
        print("✅ 连接成功")
        
        # 尝试读取保持寄存器40001-40006 (地址0-5)
        print("读取保持寄存器 40001-40006...")
        result = await client.read_holding_registers(0, 6, slave=1)
        
        if result.isError():
            print(f"❌ 读取失败: {result}")
            return False
            
        print("✅ 读取成功:")
        for i, value in enumerate(result.registers):
            print(f"  40{i+1:03d}: {value}")
            
        return True
        
    except Exception as e:
        print(f"❌ 异常: {e}")
        return False
        
    finally:
        client.close()
        print("连接已关闭")

if __name__ == "__main__":
    success = asyncio.run(test_modbus_connection())
    exit(0 if success else 1)