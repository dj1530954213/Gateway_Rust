#!/usr/bin/env node
/**
 * 简单的设备API测试
 */

const axios = require('axios');

const API_BASE_URL = 'http://localhost:50010';

async function testDeviceAPI() {
    console.log('测试设备API...');
    
    const axiosConfig = {
        timeout: 10000,
        headers: {
            'Content-Type': 'application/json',
            'Accept': 'application/json'
        }
    };
    
    try {
        // 1. 测试获取设备列表
        console.log('\n1. 获取设备列表...');
        const listResponse = await axios.get(`${API_BASE_URL}/api/v1/devices`, axiosConfig);
        console.log('✅ 设备列表获取成功');
        console.log('设备数量:', listResponse.data.items ? listResponse.data.items.length : listResponse.data.length);
        
        // 2. 创建测试设备
        console.log('\n2. 创建Modbus设备...');
        const testDevice = {
            name: 'modbus_test_' + Date.now(),
            description: 'Modbus测试设备',
            endpoint: 'tcp://127.0.0.1:502',
            protocol: 'ModbusTcp',
            location: '测试车间',
            config: {
                unit_id: 1,
                polling: '1s',
                max_regs_per_req: 120,
                retry: 3,
                endian: 'big',
                enable_write: false
            },
            enabled: true
        };
        
        const createResponse = await axios.post(`${API_BASE_URL}/api/v1/devices`, testDevice, axiosConfig);
        console.log('✅ 设备创建成功');
        console.log('设备ID:', createResponse.data.id);
        
        const deviceId = createResponse.data.id;
        
        // 3. 获取设备详情
        console.log('\n3. 获取设备详情...');
        const getResponse = await axios.get(`${API_BASE_URL}/api/v1/devices/${deviceId}`, axiosConfig);
        console.log('✅ 设备详情获取成功');
        console.log('设备名称:', getResponse.data.name);
        console.log('设备协议:', getResponse.data.protocol);
        
        // 4. 再次获取设备列表，确认设备已添加
        console.log('\n4. 再次获取设备列表...');
        const listResponse2 = await axios.get(`${API_BASE_URL}/api/v1/devices`, axiosConfig);
        console.log('✅ 设备列表更新成功');
        console.log('设备数量:', listResponse2.data.items ? listResponse2.data.items.length : listResponse2.data.length);
        
        console.log('\n🎉 设备API测试全部通过！');
        console.log('创建的设备ID:', deviceId);
        
        return { success: true, deviceId };
        
    } catch (error) {
        console.log('❌ 设备API测试失败');
        console.log('状态码:', error.response?.status);
        console.log('错误信息:', JSON.stringify(error.response?.data, null, 2));
        console.log('错误详情:', error.message);
        
        return { success: false, error: error.message };
    }
}

if (require.main === module) {
    testDeviceAPI().then(result => {
        process.exit(result.success ? 0 : 1);
    });
}