#!/usr/bin/env node
/**
 * 数据库连接测试脚本
 */

const axios = require('axios');

const API_BASE_URL = 'http://localhost:50010';

async function testDatabase() {
    console.log('测试数据库连接和API...');
    
    try {
        // 1. 测试健康检查
        console.log('\n1. 健康检查...');
        const healthResponse = await axios.get(`${API_BASE_URL}/health`);
        console.log('✅ 健康检查成功');
        console.log('服务状态:', JSON.stringify(healthResponse.data, null, 2));
        
        // 2. 测试系统健康状态
        console.log('\n2. 系统健康状态...');
        const systemHealthResponse = await axios.get(`${API_BASE_URL}/system/health`);
        console.log('✅ 系统健康状态获取成功');
        console.log('系统状态:', JSON.stringify(systemHealthResponse.data, null, 2));
        
        // 3. 测试设备API - 详细错误信息
        console.log('\n3. 设备API测试...');
        try {
            const devicesResponse = await axios.get(`${API_BASE_URL}/api/v1/devices`);
            console.log('✅ 设备API成功');
            console.log('设备列表:', JSON.stringify(devicesResponse.data, null, 2));
        } catch (error) {
            console.log('❌ 设备API失败');
            console.log('状态码:', error.response?.status);
            console.log('错误信息:', error.response?.data);
            console.log('错误详情:', error.message);
        }
        
        // 4. 测试标签API
        console.log('\n4. 标签API测试...');
        try {
            const tagsResponse = await axios.get(`${API_BASE_URL}/api/v1/tags`);
            console.log('✅ 标签API成功');
            console.log('标签列表:', JSON.stringify(tagsResponse.data, null, 2));
        } catch (error) {
            console.log('❌ 标签API失败');
            console.log('状态码:', error.response?.status);
            console.log('错误信息:', error.response?.data);
        }
        
        // 5. 测试创建简单设备
        console.log('\n5. 创建测试设备...');
        const testDevice = {
            name: 'test-device-' + Date.now(),
            description: '测试设备',
            endpoint: 'tcp://127.0.0.1:502',
            protocol: 'ModbusTcp',
            config: {
                unit_id: 1,
                timeout: 5000
            },
            enabled: true
        };
        
        try {
            const createResponse = await axios.post(`${API_BASE_URL}/api/v1/devices`, testDevice, {
                headers: { 'Content-Type': 'application/json' }
            });
            console.log('✅ 设备创建成功');
            console.log('创建的设备:', JSON.stringify(createResponse.data, null, 2));
            
            // 删除测试设备
            const deviceId = createResponse.data.id;
            await axios.delete(`${API_BASE_URL}/api/v1/devices/${deviceId}`);
            console.log('✅ 测试设备已删除');
            
        } catch (error) {
            console.log('❌ 设备创建失败');
            console.log('状态码:', error.response?.status);
            console.log('错误信息:', JSON.stringify(error.response?.data, null, 2));
            
            if (error.response?.status === 500) {
                console.log('\n💡 可能的问题:');
                console.log('- 数据库连接问题');
                console.log('- 数据库迁移未完成');
                console.log('- 数据库表不存在');
                console.log('- API代码中有运行时错误');
            }
        }
        
    } catch (error) {
        console.log('❌ 测试过程中出现异常:', error.message);
    }
}

testDatabase();