#!/usr/bin/env node
/**
 * API调试脚本 - 逐步测试每个端点
 */

const axios = require('axios');

const API_BASE_URL = 'http://localhost:50010';

async function debugAPI() {
    console.log('开始API调试...\n');
    
    const axiosConfig = {
        timeout: 5000,
        headers: { 'Accept': 'application/json' }
    };
    
    const endpoints = [
        { name: '健康检查', url: '/health', method: 'GET' },
        { name: '存活探针', url: '/health/live', method: 'GET' },
        { name: '就绪探针', url: '/health/ready', method: 'GET' },
        { name: '系统信息', url: '/system/info', method: 'GET' },
        { name: '系统指标', url: '/system/metrics', method: 'GET' },
        { name: '系统健康', url: '/system/health', method: 'GET' },
        { name: '组件状态', url: '/system/components/status', method: 'GET' },
        { name: '设备列表', url: '/api/v1/devices', method: 'GET' },
        { name: '标签列表', url: '/api/v1/tags', method: 'GET' },
    ];
    
    let workingEndpoints = 0;
    let totalEndpoints = endpoints.length;
    
    for (const endpoint of endpoints) {
        try {
            console.log(`测试: ${endpoint.name} (${endpoint.method} ${endpoint.url})`);
            
            const response = await axios({
                method: endpoint.method,
                url: `${API_BASE_URL}${endpoint.url}`,
                ...axiosConfig
            });
            
            console.log(`  ✅ 成功 - 状态: ${response.status}`);
            if (response.data && typeof response.data === 'object') {
                const keys = Object.keys(response.data);
                console.log(`  📄 响应字段: [${keys.slice(0, 5).join(', ')}${keys.length > 5 ? '...' : ''}]`);
            }
            workingEndpoints++;
            
        } catch (error) {
            console.log(`  ❌ 失败 - 状态: ${error.response?.status || 'NO_RESPONSE'}`);
            if (error.response?.data) {
                console.log(`  📄 错误: ${JSON.stringify(error.response.data).substring(0, 100)}`);
            } else {
                console.log(`  📄 错误: ${error.message}`);
            }
        }
        console.log('');
    }
    
    console.log('========================================');
    console.log(`API调试结果: ${workingEndpoints}/${totalEndpoints} 端点工作正常`);
    
    if (workingEndpoints > 0) {
        console.log('\n工作正常的端点可以用于测试功能');
        
        // 如果基础端点工作，尝试创建最简单的设备
        if (workingEndpoints >= 3) {
            console.log('\n尝试创建最简单的测试设备...');
            await createSimpleDevice();
        }
    } else {
        console.log('\n所有端点都失败了，需要检查API服务状态');
    }
}

async function createSimpleDevice() {
    const simpleDevice = {
        name: 'simple_test',
        protocol: 'ModbusTcp',
        enabled: true
    };
    
    console.log('设备数据:', JSON.stringify(simpleDevice, null, 2));
    
    try {
        const response = await axios.post(
            `${API_BASE_URL}/api/v1/devices`,
            simpleDevice,
            {
                timeout: 10000,
                headers: { 
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                }
            }
        );
        
        console.log('✅ 简单设备创建成功!');
        console.log('设备ID:', response.data.id);
        
        // 立即删除测试设备
        await axios.delete(`${API_BASE_URL}/api/v1/devices/${response.data.id}`);
        console.log('✅ 测试设备已删除');
        
    } catch (error) {
        console.log('❌ 简单设备创建失败');
        console.log('状态码:', error.response?.status);
        console.log('错误信息:', JSON.stringify(error.response?.data, null, 2));
    }
}

debugAPI();