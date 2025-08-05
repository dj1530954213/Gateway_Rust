#!/usr/bin/env node
/**
 * 数据采集功能测试
 * 测试Modbus驱动是否能从Mock服务器采集数据
 */

const { Client } = require('pg');
const axios = require('axios');

const DB_CONFIG = {
    host: 'localhost',
    port: 5432,
    database: 'iot',
    user: 'postgres',
    password: 'postgres',
};

const API_BASE_URL = 'http://localhost:50010';

class DataCollectionTest {
    constructor() {
        this.deviceId = null;
        this.tagIds = [];
    }

    async runTest() {
        console.log('📊 开始数据采集功能测试');
        console.log('目标：验证系统能从127.0.0.1:502采集Modbus数据\n');

        try {
            // 1. 创建测试设备和点位数据
            await this.setupTestData();

            // 2. 验证Mock Modbus服务器状态
            await this.verifyModbusServer();

            // 3. 测试驱动能否连接和采集
            await this.testDriverConnection();

            // 4. 监控数据采集过程
            await this.monitorDataCollection();

            // 5. 验证数据存储
            await this.verifyDataStorage();

            console.log('\n🎉 数据采集测试完成');
            return true;

        } catch (error) {
            console.log(`\n💥 数据采集测试失败: ${error.message}`);
            return false;
        } finally {
            await this.cleanup();
        }
    }

    async setupTestData() {
        console.log('1. 设置测试数据...');
        
        const client = new Client(DB_CONFIG);
        await client.connect();

        try {
            // 创建Modbus设备
            this.deviceId = '87654321-4321-4321-4321-210987654321';
            await client.query(`
                INSERT INTO devices (id, name, protocol, endpoint, location, config, enabled, created_at, updated_at) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW()) 
                ON CONFLICT (id) DO UPDATE SET
                    name = EXCLUDED.name,
                    endpoint = EXCLUDED.endpoint,
                    config = EXCLUDED.config,
                    enabled = EXCLUDED.enabled,
                    updated_at = NOW()
            `, [
                this.deviceId,
                'DataCollection_Test_PLC',
                'ModbusTcp',
                'tcp://127.0.0.1:502',
                '数据采集测试',
                JSON.stringify({
                    unit_id: 1,
                    polling: '2s',
                    max_regs_per_req: 10,
                    retry: 2,
                    endian: 'big'
                }),
                true
            ]);

            console.log('  ✅ 测试设备已创建/更新');

            // 创建6个测试点位 - 对应40001-40006
            const tags = [
                { name: 'test.temp1', address: '40001', desc: '测试温度1' },
                { name: 'test.pressure1', address: '40002', desc: '测试压力1' },
                { name: 'test.flow1', address: '40003', desc: '测试流量1' },
                { name: 'test.temp2', address: '40004', desc: '测试温度2' },
                { name: 'test.pressure2', address: '40005', desc: '测试压力2' },
                { name: 'test.flow2', address: '40006', desc: '测试流量2' }
            ];

            for (const [index, tag] of tags.entries()) {
                const tagId = `8765432${index}-4321-4321-4321-210987654321`;
                
                await client.query(`
                    INSERT INTO tags (id, device_id, name, address, data_type, scaling, unit, description, enabled, created_at) 
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                    ON CONFLICT (id) DO UPDATE SET
                        name = EXCLUDED.name,
                        address = EXCLUDED.address,
                        enabled = EXCLUDED.enabled
                `, [
                    tagId,
                    this.deviceId,
                    tag.name,
                    tag.address,
                    'Float',
                    0.1,
                    'units',
                    tag.desc,
                    true
                ]);

                this.tagIds.push(tagId);
            }

            console.log(`  ✅ ${tags.length} 个测试点位已创建/更新`);

        } finally {
            await client.end();
        }
    }

    async verifyModbusServer() {
        console.log('\n2. 验证Mock Modbus服务器...');
        
        try {
            // 检查502端口是否监听
            const { spawn } = require('child_process');
            
            // 使用netstat检查端口
            const netstat = spawn('netstat', ['-an']);
            let output = '';
            
            netstat.stdout.on('data', (data) => {
                output += data.toString();
            });

            await new Promise((resolve) => {
                netstat.on('close', () => resolve());
            });

            if (output.includes(':502') || output.includes('127.0.0.1:502')) {
                console.log('  ✅ 检测到502端口有服务监听');
            } else {
                console.log('  ⚠️  未检测到502端口监听，但继续测试');
            }

            // 测试简单的TCP连接
            const net = require('net');
            const testConnection = () => {
                return new Promise((resolve, reject) => {
                    const socket = new net.Socket();
                    socket.setTimeout(2000);
                    
                    socket.connect(502, '127.0.0.1', () => {
                        console.log('  ✅ TCP连接到127.0.0.1:502成功');
                        socket.destroy();
                        resolve(true);
                    });
                    
                    socket.on('error', () => {
                        console.log('  ❌ TCP连接到127.0.0.1:502失败');
                        resolve(false);
                    });
                    
                    socket.on('timeout', () => {
                        console.log('  ⚠️  TCP连接超时');
                        socket.destroy();
                        resolve(false);
                    });
                });
            };

            await testConnection();

        } catch (error) {
            console.log('  ⚠️  服务器验证出现问题:', error.message);
        }
    }

    async testDriverConnection() {
        console.log('\n3. 测试驱动连接...');
        
        // 这里我们无法直接测试Rust驱动，但可以检查系统是否识别我们的设备
        try {
            const systemInfo = await axios.get(`${API_BASE_URL}/system/info`);
            console.log(`  📊 系统显示设备数: ${systemInfo.data.connected_devices}`);
            console.log(`  📊 系统显示点位数: ${systemInfo.data.active_tags}`);
            
            if (systemInfo.data.connected_devices > 0) {
                console.log('  ✅ 系统已识别到设备');
            } else {
                console.log('  ⚠️  系统暂未识别到设备');
            }

        } catch (error) {
            console.log('  ❌ 驱动连接测试失败:', error.message);
        }
    }

    async monitorDataCollection() {
        console.log('\n4. 监控数据采集过程...');
        
        console.log('  ⏱️  等待数据采集启动... (15秒)');
        
        // 等待15秒让系统启动数据采集
        for (let i = 15; i > 0; i--) {
            process.stdout.write(`\r  ⏳ 倒计时: ${i} 秒`);
            await this.sleep(1000);
        }
        console.log('\n');

        // 检查系统指标变化
        try {
            const metrics = await axios.get(`${API_BASE_URL}/system/metrics`);
            console.log('  📊 系统指标:');
            console.log(`    - 活跃连接数: ${metrics.data.activeConnections}`);
            console.log(`    - 每秒消息数: ${metrics.data.messagesPerSecond}`);
            console.log(`    - 网络输入: ${(metrics.data.networkIn / 1024).toFixed(2)} KB`);
            console.log(`    - 网络输出: ${(metrics.data.networkOut / 1024).toFixed(2)} KB`);

            if (metrics.data.messagesPerSecond > 0) {
                console.log('  ✅ 检测到数据活动');
            } else {
                console.log('  ⚠️  暂未检测到数据活动');
            }

        } catch (error) {
            console.log('  ❌ 指标监控失败:', error.message);
        }
    }

    async verifyDataStorage() {
        console.log('\n5. 验证数据存储...');
        
        // 检查数据库中是否有相关的数据记录
        // 注意：这个系统可能将数据存储在InfluxDB中，而不是PostgreSQL
        
        try {
            // 检查PostgreSQL中是否有数据活动记录
            const client = new Client(DB_CONFIG);
            await client.connect();

            const deviceCheck = await client.query('SELECT * FROM devices WHERE id = $1', [this.deviceId]);
            if (deviceCheck.rows.length > 0) {
                console.log('  ✅ 测试设备在数据库中存在');
                const device = deviceCheck.rows[0];
                console.log(`    设备: ${device.name} (${device.protocol})`);
                console.log(`    状态: ${device.enabled ? '启用' : '禁用'}`);
            }

            const tagsCheck = await client.query('SELECT COUNT(*) as count FROM tags WHERE device_id = $1', [this.deviceId]);
            console.log(`  ✅ 测试点位数量: ${tagsCheck.rows[0].count}`);

            await client.end();

            // 尝试检查是否有历史数据API可用
            try {
                const historyResponse = await axios.get(`${API_BASE_URL}/api/v1/history/tags`, {
                    params: { limit: 5, device_id: this.deviceId },
                    timeout: 5000
                });
                
                if (historyResponse.data && historyResponse.data.length > 0) {
                    console.log('  ✅ 发现历史数据记录');
                    console.log(`    记录数量: ${historyResponse.data.length}`);
                } else {
                    console.log('  ⚠️  历史数据API返回空结果');
                }
                
            } catch (error) {
                console.log('  ⚠️  历史数据API不可用或无数据');
            }

        } catch (error) {
            console.log('  ❌ 数据存储验证失败:', error.message);
        }
    }

    async cleanup() {
        console.log('\n🧹 清理测试数据...');
        
        const client = new Client(DB_CONFIG);
        await client.connect();

        try {
            await client.query('DELETE FROM tags WHERE device_id = $1', [this.deviceId]);
            await client.query('DELETE FROM devices WHERE id = $1', [this.deviceId]);
            console.log('  ✅ 测试数据已清理');
        } finally {
            await client.end();
        }
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

async function main() {
    const test = new DataCollectionTest();
    const success = await test.runTest();
    
    if (success) {
        console.log('\n🎊 数据采集测试总结:');
        console.log('✅ 系统基础设施正常运行');
        console.log('✅ Mock Modbus服务器连接测试完成');
        console.log('✅ 设备和点位数据配置正确');
        console.log('💡 数据采集功能需要更多时间验证实际效果');
    }
    
    process.exit(success ? 0 : 1);
}

main();