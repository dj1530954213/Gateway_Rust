#!/usr/bin/env node
/**
 * 创建真实的ModbusTCP设备和标签 - 不删除数据
 */

const { Client } = require('pg');

const DB_CONFIG = {
    host: 'localhost',
    port: 5432,
    database: 'iot',
    user: 'postgres',
    password: 'postgres',
};

async function createRealModbusDevice() {
    console.log('🔧 开始创建真实的ModbusTCP设备和标签');
    console.log('⚠️  注意：此次创建的数据将永久保存，不会删除！\n');

    const client = new Client(DB_CONFIG);
    
    try {
        await client.connect();
        console.log('✅ 数据库连接成功');
        
        // 1. 创建ModbusTCP设备
        const deviceId = 'f47ac10b-58cc-4372-a567-0e02b2c3d479'; // 固定UUID
        const deviceName = 'ModbusTCP_PLC_Real';
        
        console.log('\n📝 创建ModbusTCP设备...');
        console.log(`设备ID: ${deviceId}`);
        console.log(`设备名称: ${deviceName}`);
        
        const deviceResult = await client.query(`
            INSERT INTO devices (id, name, protocol, endpoint, location, config, enabled, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW()) 
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                endpoint = EXCLUDED.endpoint,
                config = EXCLUDED.config,
                enabled = EXCLUDED.enabled,
                updated_at = NOW()
            RETURNING id, name, protocol, endpoint, enabled
        `, [
            deviceId,
            deviceName,
            'ModbusTcp',
            'tcp://127.0.0.1:502',
            '生产车间A',
            JSON.stringify({
                unit_id: 1,
                polling_interval: '2s',
                max_registers_per_request: 10,
                retry_count: 3,
                timeout: '5s',
                byte_order: 'big_endian'
            }),
            true
        ]);

        console.log('✅ ModbusTCP设备创建成功!');
        console.log('设备信息:', deviceResult.rows[0]);
        
        // 2. 创建6个浮点型标签 (40001-40006)
        console.log('\n📝 创建6个浮点型标签...');
        
        const tags = [
            { name: 'sensor.temperature1', address: '40001', description: '车间温度传感器1', unit: '°C', scaling: 0.1 },
            { name: 'sensor.pressure1', address: '40002', description: '液压系统压力1', unit: 'bar', scaling: 0.01 },
            { name: 'sensor.flow1', address: '40003', description: '冷却液流量1', unit: 'L/min', scaling: 0.1 },
            { name: 'sensor.temperature2', address: '40004', description: '车间温度传感器2', unit: '°C', scaling: 0.1 },
            { name: 'sensor.pressure2', address: '40005', description: '液压系统压力2', unit: 'bar', scaling: 0.01 },
            { name: 'sensor.flow2', address: '40006', description: '冷却液流量2', unit: 'L/min', scaling: 0.1 }
        ];

        const createdTags = [];
        
        for (const [index, tag] of tags.entries()) {
            const tagId = `f47ac10${index}-58cc-4372-a567-0e02b2c3d479`; // 基于设备ID的变体
            
            const tagResult = await client.query(`
                INSERT INTO tags (id, device_id, name, address, data_type, scaling, unit, description, enabled, created_at) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                ON CONFLICT (id) DO UPDATE SET
                    name = EXCLUDED.name,
                    address = EXCLUDED.address,
                    data_type = EXCLUDED.data_type,
                    scaling = EXCLUDED.scaling,
                    unit = EXCLUDED.unit,
                    description = EXCLUDED.description,
                    enabled = EXCLUDED.enabled
                RETURNING id, name, address, data_type, unit, scaling
            `, [
                tagId,
                deviceId,
                tag.name,
                tag.address,
                'Float',
                tag.scaling,
                tag.unit,
                tag.description,
                true
            ]);

            createdTags.push(tagResult.rows[0]);
            console.log(`✅ 标签创建成功: ${tag.name} (地址:${tag.address})`);
        }
        
        // 3. 验证创建结果
        console.log('\n🔍 验证创建结果...');
        
        const deviceCheck = await client.query('SELECT COUNT(*) as count FROM devices WHERE id = $1', [deviceId]);
        const tagsCheck = await client.query('SELECT COUNT(*) as count FROM tags WHERE device_id = $1', [deviceId]);
        
        console.log(`✅ 设备验证: ${deviceCheck.rows[0].count} 个设备`);
        console.log(`✅ 标签验证: ${tagsCheck.rows[0].count} 个标签`);
        
        // 4. 显示最终数据库状态
        console.log('\n📊 当前数据库状态:');
        const totalDevices = await client.query('SELECT COUNT(*) as count FROM devices');
        const totalTags = await client.query('SELECT COUNT(*) as count FROM tags');
        
        console.log(`总设备数: ${totalDevices.rows[0].count}`);
        console.log(`总标签数: ${totalTags.rows[0].count}`);
        
        console.log('\n🎯 创建完成！请在DBeaver中刷新查看:');
        console.log('1. devices表应该有1个新设备');
        console.log('2. tags表应该有6个新标签');
        console.log('3. 所有数据都是持久化的，不会被删除');
        
        return {
            deviceId: deviceId,
            deviceName: deviceName,
            tagCount: createdTags.length,
            tags: createdTags
        };
        
    } catch (error) {
        console.log('❌ 创建失败:', error.message);
        console.log('错误详情:', error);
        return null;
    } finally {
        await client.end();
        console.log('\n数据库连接已关闭');
    }
}

async function main() {
    const result = await createRealModbusDevice();
    
    if (result) {
        console.log('\n🎊 SUCCESS! 真实ModbusTCP设备创建成功');
        console.log(`设备: ${result.deviceName} (${result.deviceId})`);
        console.log(`标签数量: ${result.tagCount}`);
        console.log('\n现在您可以在DBeaver中看到这些真实数据！');
    } else {
        console.log('\n💥 FAILED! 设备创建失败');
    }
}

main();