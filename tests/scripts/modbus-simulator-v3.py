#!/usr/bin/env python3
"""
Modbus TCP 模拟器 (pymodbus 3.x版本)
用于测试Gateway Rust项目的Modbus驱动功能
"""

import asyncio
import time
import math
import threading
from pymodbus.server import StartAsyncTcpServer
from pymodbus.device import ModbusDeviceIdentification
from pymodbus.datastore import ModbusSequentialDataBlock, ModbusSlaveContext, ModbusServerContext

class ModbusSimulator:
    def __init__(self):
        self.context = None
        self.server_task = None
        self.update_task = None
        self.start_time = time.time()
        
    def create_modbus_context(self):
        """创建Modbus数据存储上下文"""
        
        # 创建数据块，支持40001-40011地址（Modbus地址从1开始，但内部存储从0开始）
        # 40001-40011 对应内部地址 0-10
        holding_registers = ModbusSequentialDataBlock(0, [0] * 20)  # 创建更多空间以防越界
        
        # 初始化一些测试数据
        # 40001-40011 (地址0-10) 存储6个浮点数
        initial_values = [
            123,    # 40001 - 温度传感器1 (12.3°C * 10)
            456,    # 40002 - 压力传感器1 (4.56 bar * 100) 
            789,    # 40003 - 流量传感器1 (78.9 L/min * 10)
            234,    # 40004 - 温度传感器2 (23.4°C * 10)
            567,    # 40005 - 压力传感器2 (5.67 bar * 100)
            890,    # 40006 - 流量传感器2 (89.0 L/min * 10)
            100,    # 40007 - 备用1
            200,    # 40008 - 备用2  
            300,    # 40009 - 备用3
            400,    # 40010 - 备用4
            500,    # 40011 - 备用5
        ]
        
        for i, value in enumerate(initial_values):
            if i < 20:  # 确保不超出范围
                holding_registers.setValues(i, [value])
        
        # 创建从站上下文
        slave_context = ModbusSlaveContext(
            di=None,  # 离散输入 (读线圈)
            co=None,  # 线圈 (读写线圈)
            hr=holding_registers,  # 保持寄存器 (读写)
            ir=None   # 输入寄存器 (只读)
        )
        
        # 创建服务器上下文，单元ID=1
        self.context = ModbusServerContext(slaves={1: slave_context}, single=False)
        return self.context

    async def update_values_continuously(self):
        """持续更新模拟数据"""
        print("开始数据更新任务...")
        
        while True:
            try:
                current_time = time.time()
                elapsed = current_time - self.start_time
                
                # 模拟动态数据变化
                slave_context = self.context[1]  # 获取单元ID=1的上下文
                
                # 40001 - 温度传感器1: 20-30°C 正弦波动
                temp1 = int((25 + 5 * math.sin(elapsed * 0.1)) * 10)
                slave_context.setValues(3, 0, [temp1])  # 功能码3，地址0 (40001)
                
                # 40002 - 压力传感器1: 4-6 bar 余弦波动
                pressure1 = int((5 + 1 * math.cos(elapsed * 0.15)) * 100)
                slave_context.setValues(3, 1, [pressure1])  # 地址1 (40002)
                
                # 40003 - 流量传感器1: 70-90 L/min 线性变化
                flow1 = int((80 + 10 * math.sin(elapsed * 0.2)) * 10)
                slave_context.setValues(3, 2, [flow1])  # 地址2 (40003)
                
                # 40004 - 温度传感器2: 15-25°C 
                temp2 = int((20 + 5 * math.cos(elapsed * 0.12)) * 10)
                slave_context.setValues(3, 3, [temp2])  # 地址3 (40004)
                
                # 40005 - 压力传感器2: 3-7 bar
                pressure2 = int((5 + 2 * math.sin(elapsed * 0.18)) * 100)
                slave_context.setValues(3, 4, [pressure2])  # 地址4 (40005)
                
                # 40006 - 流量传感器2: 60-100 L/min  
                flow2 = int((80 + 20 * math.cos(elapsed * 0.25)) * 10)
                slave_context.setValues(3, 5, [flow2])  # 地址5 (40006)
                
                # 每5秒打印一次当前值
                if int(elapsed) % 5 == 0 and elapsed - int(elapsed) < 0.1:
                    print(f"[{int(elapsed)}s] 模拟数据更新:")
                    print(f"  40001 温度1: {temp1/10:.1f}°C")
                    print(f"  40002 压力1: {pressure1/100:.2f} bar") 
                    print(f"  40003 流量1: {flow1/10:.1f} L/min")
                    print(f"  40004 温度2: {temp2/10:.1f}°C")
                    print(f"  40005 压力2: {pressure2/100:.2f} bar")
                    print(f"  40006 流量2: {flow2/10:.1f} L/min")
                    print()
                
            except Exception as e:
                print(f"数据更新错误: {e}")
            
            await asyncio.sleep(0.1)  # 100ms更新间隔

    async def start_server(self):
        """启动Modbus TCP服务器"""
        print("🚀 启动Modbus TCP模拟器")
        print("=" * 50)
        print("服务器配置:")
        print("  地址: 127.0.0.1:502")
        print("  单元ID: 1")
        print("  寄存器地址: 40001-40011")
        print("  数据类型: 6个浮点型传感器数据")
        print("=" * 50)
        
        # 创建Modbus上下文
        context = self.create_modbus_context()
        
        # 设备标识信息
        identity = ModbusDeviceIdentification()
        identity.VendorName = 'Gateway Rust Test'
        identity.ProductCode = 'ModbusSim'
        identity.VendorUrl = 'https://github.com/gateway-rust'
        identity.ProductName = 'Modbus TCP Simulator'
        identity.ModelName = 'Test Simulator v1.0'
        identity.MajorMinorRevision = '1.0.0'
        
        print("✅ 服务器上下文已创建")
        
        # 启动数据更新任务
        self.update_task = asyncio.create_task(self.update_values_continuously())
        print("✅ 数据更新任务已启动")
        print()
        print("📡 正在启动Modbus TCP服务器...")
        print("   按 Ctrl+C 停止服务器")
        print()
        
        try:
            # 启动异步TCP服务器
            await StartAsyncTcpServer(
                context=context,
                identity=identity,
                address=('127.0.0.1', 502),
                allow_reuse_address=True,
            )
        except Exception as e:
            print(f"❌ 服务器启动失败: {e}")
            print("   请确保端口502未被占用，或者以管理员权限运行")
            raise

async def main():
    simulator = ModbusSimulator()
    
    try:
        await simulator.start_server()
    except KeyboardInterrupt:
        print("\n🛑 接收到停止信号，正在关闭服务器...")
    except Exception as e:
        print(f"❌ 服务器错误: {e}")
    finally:
        if simulator.update_task:
            simulator.update_task.cancel()
        print("👋 Modbus TCP模拟器已停止")

if __name__ == "__main__":
    asyncio.run(main())