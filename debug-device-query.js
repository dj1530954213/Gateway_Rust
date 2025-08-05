#!/usr/bin/env node
/**
 * 调试设备查询问题
 * 模拟Rust代码的查询逻辑
 */

const { Client } = require('pg');

const DB_CONFIG = {
    host: 'localhost',
    port: 5432,
    database: 'iot',
    user: 'postgres',
    password: 'postgres',
};

async function debugDeviceQuery() {
    console.log('🔍 调试设备查询问题...\n');

    const client = new Client(DB_CONFIG);
    
    try {
        await client.connect();
        
        // 1. 检查枚举类型定义
        console.log('1. 检查数据库枚举类型定义...');
        const enumTypes = await client.query(`
            SELECT enumlabel, enumsortorder 
            FROM pg_enum 
            WHERE enumtypid = (SELECT oid FROM pg_type WHERE typname = 'protocol_kind')
            ORDER BY enumsortorder
        `);
        
        console.log('protocol_kind枚举值:');
        enumTypes.rows.forEach((row, i) => {
            console.log(`  ${i+1}. ${row.enumlabel}`);
        });
        
        // 2. 直接查询设备数据
        console.log('\n2. 直接查询设备数据...');
        const devices = await client.query(`
            SELECT id, name, protocol, endpoint, enabled, created_at 
            FROM devices 
            ORDER BY created_at DESC
        `);
        
        console.log(`找到 ${devices.rows.length} 个设备:`);
        devices.rows.forEach((device, i) => {
            console.log(`  ${i+1}. ${device.name}`);
            console.log(`     协议: ${device.protocol}`);
            console.log(`     端点: ${device.endpoint}`);
            console.log(`     状态: ${device.enabled ? '启用' : '禁用'}`);
            console.log(`     ID: ${device.id}`);
            console.log('');
        });
        
        // 3. 测试协议过滤查询
        console.log('3. 测试协议过滤查询...');
        try {
            const modbusDevices = await client.query(`
                SELECT id, name, protocol, endpoint, enabled 
                FROM devices 
                WHERE protocol = 'ModbusTcp'
            `);
            
            console.log(`ModbusTcp设备数量: ${modbusDevices.rows.length}`);
            modbusDevices.rows.forEach((device, i) => {
                console.log(`  ${i+1}. ${device.name} - ${device.protocol}`);
            });
        } catch (error) {
            console.log('❌ 协议过滤查询失败:', error.message);
        }
        
        // 4. 测试计数查询
        console.log('\n4. 测试计数查询...');
        try {
            const countResult = await client.query('SELECT COUNT(*) as count FROM devices');
            console.log(`设备总数: ${countResult.rows[0].count}`);
            
            const modbusCount = await client.query(`
                SELECT COUNT(*) as count FROM devices WHERE protocol = 'ModbusTcp'
            `);
            console.log(`ModbusTcp设备数: ${modbusCount.rows[0].count}`);
            
        } catch (error) {
            console.log('❌ 计数查询失败:', error.message);
        }
        
        // 5. 检查是否有权限或连接问题
        console.log('\n5. 检查数据库权限...');
        const userInfo = await client.query('SELECT current_user, current_database()');
        console.log(`当前用户: ${userInfo.rows[0].current_user}`);
        console.log(`当前数据库: ${userInfo.rows[0].current_database}`);
        
        // 6. 检查表权限
        const tablePrivileges = await client.query(`
            SELECT privilege_type 
            FROM information_schema.role_table_grants 
            WHERE table_name = 'devices' AND grantee = current_user
        `);
        console.log('devices表权限:', tablePrivileges.rows.map(r => r.privilege_type).join(', '));
        
        console.log('\n✅ 数据库查询调试完成');
        
        return {
            enumTypes: enumTypes.rows,
            deviceCount: devices.rows.length,
            devices: devices.rows,
            permissions: tablePrivileges.rows
        };
        
    } catch (error) {
        console.log('❌ 调试失败:', error.message);
        console.log('错误详情:', error);
        return null;
    } finally {
        await client.end();
    }
}

async function main() {
    const result = await debugDeviceQuery();
    
    if (result) {
        console.log('\n💡 分析结果:');
        if (result.deviceCount > 0) {
            console.log('✅ 数据库中有设备数据');
            console.log('✅ 直接SQL查询正常工作');
            console.log('💭 问题可能在于:');
            console.log('   1. Rust sqlx的类型映射');
            console.log('   2. API路由或中间件问题');
            console.log('   3. 数据库连接池配置');
        } else {
            console.log('⚠️  数据库中没有设备数据');
        }
    } else {
        console.log('💥 无法完成调试');
    }
}

main();