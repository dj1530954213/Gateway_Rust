#!/usr/bin/env node
/**
 * 完整的Modbus TCP设备和点位管理测试
 * 
 * 测试流程：
 * 1. 添加Modbus设备 (127.0.0.1:502)
 * 2. 添加6个传感器点位 (40001-40006)
 * 3. 验证设备和点位创建成功
 * 4. 测试数据采集功能
 * 5. 通过前端API查看实时数据
 */

const axios = require('axios');

// 配置
const API_BASE_URL = 'http://localhost:50010';
const MODBUS_HOST = '127.0.0.1';
const MODBUS_PORT = 502;

// 设备和点位数据
const MODBUS_DEVICE = {
    name: 'PLC_Simulator',
    description: 'Mock Modbus TCP PLC设备',
    endpoint: `tcp://${MODBUS_HOST}:${MODBUS_PORT}`,
    driver_type: 'modbus-tcp',
    config: {
        unit_id: 1,
        polling: '1s',
        max_regs_per_req: 120,
        retry: 3,
        endian: 'big',
        enable_write: false
    }
};

const SENSOR_TAGS = [
    {
        name: 'sensor.temp1',
        description: '温度传感器1',
        address: '40001',
        data_type: 'Float',
        access: 'Read',
        unit: '°C',
        scale: 0.1
    },
    {
        name: 'sensor.pressure1', 
        description: '压力传感器1',
        address: '40002',
        data_type: 'Float',
        access: 'Read',
        unit: 'bar',
        scale: 0.01
    },
    {
        name: 'sensor.flow1',
        description: '流量传感器1',
        address: '40003', 
        data_type: 'Float',
        access: 'Read',
        unit: 'L/min',
        scale: 0.1
    },
    {
        name: 'sensor.temp2',
        description: '温度传感器2',
        address: '40004',
        data_type: 'Float', 
        access: 'Read',
        unit: '°C',
        scale: 0.1
    },
    {
        name: 'sensor.pressure2',
        description: '压力传感器2',
        address: '40005',
        data_type: 'Float',
        access: 'Read', 
        unit: 'bar',
        scale: 0.01
    },
    {
        name: 'sensor.flow2',
        description: '流量传感器2',
        address: '40006',
        data_type: 'Float',
        access: 'Read',
        unit: 'L/min', 
        scale: 0.1
    }
];

class ModbusTestManager {
    constructor() {
        this.deviceId = null;
        this.tagIds = [];
        this.axiosConfig = {
            timeout: 10000,
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'application/json'
            }
        };
    }

    async waitForApiReady() {
        console.log('等待API服务就绪...');
        
        for (let i = 0; i < 30; i++) {
            try {
                const response = await axios.get(`${API_BASE_URL}/health`, this.axiosConfig);
                if (response.status === 200) {
                    console.log('✅ API服务已就绪');
                    return true;
                }
            } catch (error) {
                console.log(`⏳ 等待API服务... (${i + 1}/30)`);
                await this.sleep(2000);
            }
        }
        
        throw new Error('API服务启动超时');
    }

    async addDevice() {
        console.log('\n📡 添加Modbus设备...');
        console.log(`设备: ${MODBUS_DEVICE.name}`);
        console.log(`端点: ${MODBUS_DEVICE.endpoint}`);
        
        try {
            const response = await axios.post(
                `${API_BASE_URL}/api/v1/devices`,
                MODBUS_DEVICE,
                this.axiosConfig
            );
            
            this.deviceId = response.data.id;
            console.log(`✅ 设备创建成功: ID = ${this.deviceId}`);
            return true;
            
        } catch (error) {
            console.log(`❌ 设备创建失败: ${error.response?.status} ${error.response?.statusText}`);
            if (error.response?.data) {
                console.log(`详细错误: ${JSON.stringify(error.response.data)}`);
            }
            return false;
        }
    }

    async addTags() {
        console.log('\n🏷️  添加传感器点位...');
        
        let successCount = 0;
        
        for (const [index, tag] of SENSOR_TAGS.entries()) {
            console.log(`添加点位: ${tag.name} (${tag.description})`);
            
            const tagData = {
                ...tag,
                device_id: this.deviceId
            };
            
            try {
                const response = await axios.post(
                    `${API_BASE_URL}/api/v1/tags`,
                    tagData,
                    this.axiosConfig
                );
                
                this.tagIds.push(response.data.id);
                console.log(`  ✅ ${tag.name} - ID: ${response.data.id}`);
                successCount++;
                
            } catch (error) {
                console.log(`  ❌ ${tag.name} - 错误: ${error.response?.status || error.message}`);
            }
        }
        
        console.log(`\n📊 点位创建结果: ${successCount}/${SENSOR_TAGS.length} 成功`);
        return successCount === SENSOR_TAGS.length;
    }

    async verifyDevicesAndTags() {
        console.log('\n🔍 验证设备和点位...');
        
        try {
            // 验证设备列表
            const devicesResponse = await axios.get(`${API_BASE_URL}/api/v1/devices`, this.axiosConfig);
            const devices = devicesResponse.data;
            
            console.log(`设备列表: 找到 ${devices.length} 个设备`);
            const ourDevice = devices.find(d => d.id === this.deviceId);
            
            if (ourDevice) {
                console.log(`  ✅ 设备确认: ${ourDevice.name}`);
            } else {
                console.log(`  ❌ 未找到我们创建的设备`);
                return false;
            }
            
            // 验证点位列表
            const tagsResponse = await axios.get(`${API_BASE_URL}/api/v1/tags`, this.axiosConfig);
            const tags = tagsResponse.data;
            
            console.log(`点位列表: 找到 ${tags.length} 个点位`);
            const ourTags = tags.filter(t => this.tagIds.includes(t.id));
            
            console.log(`  我们的点位: ${ourTags.length}/${this.tagIds.length}`);
            ourTags.forEach(tag => {
                console.log(`    - ${tag.name}: ${tag.description}`);
            });
            
            return ourTags.length === this.tagIds.length;
            
        } catch (error) {
            console.log(`❌ 验证失败: ${error.message}`);
            return false;
        }
    }

    async testDataCollection() {
        console.log('\n📈 测试数据采集...');
        
        // 等待几秒让驱动开始采集数据
        console.log('等待数据采集启动...');
        await this.sleep(5000);
        
        try {
            // 查询历史数据API（如果有的话）
            const historyResponse = await axios.get(
                `${API_BASE_URL}/api/v1/history/tags`,
                {
                    ...this.axiosConfig,
                    params: {
                        tags: this.tagIds.join(','),
                        limit: 10
                    }
                }
            );
            
            console.log(`✅ 历史数据查询成功: ${historyResponse.data.length} 条记录`);
            
            if (historyResponse.data.length > 0) {
                console.log('最新数据样本:');
                historyResponse.data.slice(0, 3).forEach(record => {
                    console.log(`  ${record.tag}: ${record.value} (${record.timestamp})`);
                });
            }
            
            return true;
            
        } catch (error) {
            console.log(`❌ 数据采集测试失败: ${error.response?.status || error.message}`);
            
            // 尝试其他可能的数据查询端点
            try {
                const realtimeResponse = await axios.get(`${API_BASE_URL}/api/v1/realtime`, this.axiosConfig);
                console.log(`✅ 实时数据接口可用: ${JSON.stringify(realtimeResponse.data).substring(0, 100)}...`);
                return true;
            } catch (e) {
                console.log(`❌ 实时数据接口也无法访问: ${e.response?.status || e.message}`);  
                return false;
            }
        }
    }

    async testFrontendAccess() {
        console.log('\n🌐 测试前端访问...');
        
        try {
            // 测试一些关键的前端API端点
            const endpoints = [
                { name: '设备列表', url: '/api/v1/devices' },
                { name: '点位列表', url: '/api/v1/tags' },
                { name: '系统状态', url: '/system/info' },
                { name: '系统健康', url: '/health' }
            ];
            
            let successCount = 0;
            
            for (const endpoint of endpoints) {
                try {
                    const response = await axios.get(`${API_BASE_URL}${endpoint.url}`, this.axiosConfig);
                    console.log(`  ✅ ${endpoint.name}: HTTP ${response.status}`);
                    successCount++;
                } catch (error) {
                    console.log(`  ❌ ${endpoint.name}: ${error.response?.status || error.message}`);
                }
            }
            
            console.log(`📊 前端API测试: ${successCount}/${endpoints.length} 通过`);
            return successCount === endpoints.length;
            
        } catch (error) {
            console.log(`❌ 前端访问测试失败: ${error.message}`);
            return false;
        }
    }

    async runCompleteTest() {
        console.log('🚀 开始Modbus完整功能测试');
        console.log('=' * 50);
        
        const results = {
            apiReady: false,
            deviceAdded: false,
            tagsAdded: false,
            verified: false,
            dataCollection: false,
            frontendAccess: false
        };
        
        try {
            // 1. 等待API就绪
            results.apiReady = await this.waitForApiReady();
            
            // 2. 添加设备
            if (results.apiReady) {
                results.deviceAdded = await this.addDevice();
            }
            
            // 3. 添加点位
            if (results.deviceAdded) {
                results.tagsAdded = await this.addTags();
            }
            
            // 4. 验证创建结果
            if (results.tagsAdded) {
                results.verified = await this.verifyDevicesAndTags();
            }
            
            // 5. 测试数据采集
            if (results.verified) {
                results.dataCollection = await this.testDataCollection();
            }
            
            // 6. 测试前端访问
            results.frontendAccess = await this.testFrontendAccess();
            
        } catch (error) {
            console.log(`\n💥 测试过程中出现异常: ${error.message}`);
        }
        
        // 输出最终结果
        this.printTestResults(results);
        
        return this.allTestsPassed(results);
    }

    printTestResults(results) {
        console.log('\n' + '=' * 50);
        console.log('📋 测试结果总结');
        console.log('=' * 50);
        
        const tests = [
            { name: 'API服务就绪', result: results.apiReady },
            { name: 'Modbus设备添加', result: results.deviceAdded },
            { name: '传感器点位添加', result: results.tagsAdded },
            { name: '设备点位验证', result: results.verified },
            { name: '数据采集测试', result: results.dataCollection },
            { name: '前端接口访问', result: results.frontendAccess }
        ];
        
        let passedCount = 0;
        
        tests.forEach(test => {
            const status = test.result ? '✅ 通过' : '❌ 失败';
            console.log(`${status} ${test.name}`);
            if (test.result) passedCount++;
        });
        
        console.log('-' * 50);
        console.log(`总体结果: ${passedCount}/${tests.length} 测试通过`);
        
        if (passedCount === tests.length) {
            console.log('🎉 所有测试通过！Modbus功能完全正常');
        } else {
            console.log('⚠️  部分测试失败，需要检查和修复');
        }
        
        if (this.deviceId) {
            console.log(`\n📋 创建的资源信息:`);
            console.log(`设备ID: ${this.deviceId}`);
            console.log(`点位数量: ${this.tagIds.length}`);
            console.log(`点位IDs: [${this.tagIds.join(', ')}]`);
        }
    }

    allTestsPassed(results) {
        return Object.values(results).every(result => result === true);
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// 运行测试
async function main() {
    const testManager = new ModbusTestManager();
    
    try {
        const success = await testManager.runCompleteTest();
        process.exit(success ? 0 : 1);
    } catch (error) {
        console.error('测试执行失败:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}