#!/usr/bin/env node
/**
 * 直接数据库连接测试 - 模拟API服务的数据库操作
 */

const { Client } = require('pg');

// 数据库连接配置 - 和API服务相同
const DB_CONFIG = {
    host: 'localhost',
    port: 5432,
    database: 'iot',
    user: 'postgres',
    password: 'postgres',
};

async function testDatabaseOperations() {
    console.log('测试数据库操作...');
    console.log('连接配置:', DB_CONFIG);
    
    const client = new Client(DB_CONFIG);
    
    try {
        // 1. 连接数据库
        console.log('\n1. 连接数据库...');
        await client.connect();
        console.log('✅ 数据库连接成功');
        
        // 2. 测试基本查询
        console.log('\n2. 测试基本查询...');
        const versionResult = await client.query('SELECT version()');
        console.log('✅ PostgreSQL版本:', versionResult.rows[0].version.substring(0, 50) + '...');
        
        // 3. 检查表结构
        console.log('\n3. 检查表结构...');
        const tablesResult = await client.query(`
            SELECT table_name, table_type 
            FROM information_schema.tables 
            WHERE table_schema = 'public'
            ORDER BY table_name
        `);
        console.log('✅ 找到表:', tablesResult.rows.length, '个');
        tablesResult.rows.forEach(row => {
            console.log(`  - ${row.table_name} (${row.table_type})`);
        });
        
        // 4. 测试设备表查询
        console.log('\n4. 测试设备表查询...');
        const devicesCountResult = await client.query('SELECT COUNT(*) as count FROM devices');
        console.log('✅ 设备表查询成功，设备数量:', devicesCountResult.rows[0].count);
        
        const devicesResult = await client.query('SELECT id, name, protocol, enabled FROM devices LIMIT 5');
        console.log('✅ 设备列表查询成功，返回', devicesResult.rows.length, '条记录');
        devicesResult.rows.forEach(device => {
            console.log(`  - ${device.name} (${device.protocol}) - ${device.enabled ? '启用' : '禁用'}`);
        });
        
        // 5. 测试标签表查询
        console.log('\n5. 测试标签表查询...');
        const tagsCountResult = await client.query('SELECT COUNT(*) as count FROM tags');
        console.log('✅ 标签表查询成功，标签数量:', tagsCountResult.rows[0].count);
        
        // 6. 测试创建设备
        console.log('\n6. 测试创建设备...');
        const deviceId = 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee'; // 固定UUID用于测试
        const deviceName = 'test_device_' + Date.now();
        
        const insertResult = await client.query(`
            INSERT INTO devices (id, name, protocol, endpoint, enabled, config) 
            VALUES ($1, $2, $3, $4, $5, $6) 
            RETURNING id, name
        `, [deviceId, deviceName, 'ModbusTcp', 'tcp://127.0.0.1:502', true, JSON.stringify({unit_id: 1})]);
        
        console.log('✅ 设备创建成功:', insertResult.rows[0]);
        
        // 7. 验证设备创建
        console.log('\n7. 验证设备创建...');
        const verifyResult = await client.query('SELECT * FROM devices WHERE id = $1', [deviceId]);
        console.log('✅ 设备验证成功:', {
            id: verifyResult.rows[0].id,
            name: verifyResult.rows[0].name,
            protocol: verifyResult.rows[0].protocol,
            enabled: verifyResult.rows[0].enabled
        });
        
        // 8. 清理测试数据
        console.log('\n8. 清理测试数据...');
        await client.query('DELETE FROM devices WHERE id = $1', [deviceId]);
        console.log('✅ 测试设备已删除');
        
        console.log('\n🎉 所有数据库操作测试通过！');
        console.log('数据库连接和操作都正常，问题可能在API服务的实现中');
        
        return true;
        
    } catch (error) {
        console.log('❌ 数据库操作失败:', error.message);
        console.log('错误详情:', error);
        return false;
    } finally {
        await client.end();
        console.log('数据库连接已关闭');
    }
}

async function main() {
    const success = await testDatabaseOperations();
    
    if (success) {
        console.log('\n💡 建议检查事项:');
        console.log('1. API服务的数据库连接字符串是否正确');
        console.log('2. API服务是否有正确的数据库权限');
        console.log('3. API服务的错误日志中是否有具体错误信息');
        console.log('4. Rust代码中的数据库查询是否有语法错误');
    }
    
    process.exit(success ? 0 : 1);
}

main();