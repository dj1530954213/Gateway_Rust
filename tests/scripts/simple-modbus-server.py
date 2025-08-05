#!/usr/bin/env python3
"""
简单的Modbus TCP服务器
使用最基本的pymodbus 3.x API
"""

import asyncio
import time
import math
from pymodbus.datastore import ModbusSequentialDataBlock, ModbusDeviceContext, ModbusServerContext
from pymodbus.server import StartAsyncTcpServer

class SimpleModbusServer:
    def __init__(self):
        self.start_time = time.time()
        self.context = None
        
    def setup_datastore(self):
        """设置数据存储"""
        # 创建保持寄存器，地址0-19 (对应40001-40020)
        holding_block = ModbusSequentialDataBlock(0, [0] * 20)
        
        # 初始化测试数据
        initial_data = [
            123,   # 40001
            456,   # 40002  
            789,   # 40003
            234,   # 40004
            567,   # 40005
            890,   # 40006
            111,   # 40007
            222,   # 40008
            333,   # 40009  
            444,   # 40010
            555,   # 40011
        ]
        
        for i, val in enumerate(initial_data):
            holding_block.setValues(i, [val])
            
        # 创建设备上下文
        device_context = ModbusDeviceContext(
            di=None,
            co=None, 
            hr=holding_block,
            ir=None
        )
        
        # 创建服务器上下文 (单元ID=1)
        self.context = ModbusServerContext(devices={1: device_context}, single=False)
        return self.context
        
    async def update_data(self):
        """持续更新数据"""
        while True:
            try:
                elapsed = time.time() - self.start_time
                slave = self.context[1]
                
                # 动态更新数据
                temp1 = int((25 + 5 * math.sin(elapsed * 0.1)) * 10)
                pressure1 = int((5 + 1 * math.cos(elapsed * 0.15)) * 100) 
                flow1 = int((80 + 10 * math.sin(elapsed * 0.2)) * 10)
                
                slave.setValues(3, 0, [temp1])    # 40001
                slave.setValues(3, 1, [pressure1]) # 40002
                slave.setValues(3, 2, [flow1])    # 40003
                
                # 每10秒打印状态
                if int(elapsed) % 10 == 0 and elapsed - int(elapsed) < 0.5:
                    print(f"[{int(elapsed)}s] 数据更新: 40001={temp1}, 40002={pressure1}, 40003={flow1}")
                    
            except Exception as e:
                print(f"更新数据时出错: {e}")
                
            await asyncio.sleep(0.5)

async def run_server():
    print("启动简单Modbus TCP服务器")
    print("地址: 127.0.0.1:502")
    print("单元ID: 1")
    print("寄存器: 40001-40011")
    print("按Ctrl+C停止")
    print("-" * 40)
    
    server = SimpleModbusServer()
    context = server.setup_datastore()
    
    # 启动数据更新任务
    update_task = asyncio.create_task(server.update_data())
    
    try:
        # 启动服务器
        await StartAsyncTcpServer(
            context=context,
            address=('127.0.0.1', 502)
        )
    except KeyboardInterrupt:
        print("\n停止服务器...")
        update_task.cancel()
    except Exception as e:
        print(f"服务器错误: {e}")
        update_task.cancel()

if __name__ == "__main__":
    asyncio.run(run_server())