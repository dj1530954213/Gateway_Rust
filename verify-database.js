#!/usr/bin/env node
/**
 * 数据库数据验证脚本
 * 用于验证数据库中的实际数据情况
 */

const { Client } = require('pg');

const DB_CONFIG = {
    host: 'localhost',
    port: 5432,
    database: 'iot',
    user: 'postgres',
    password: 'postgres',
};

async function verifyDatabase() {
    console.log('🔍 开始验证数据库实际数据...');
    console.log('连接信息:');
    console.log(`  主机: ${DB_CONFIG.host}`);
    console.log(`  端口: ${DB_CONFIG.port}`);
    console.log(`  数据库: ${DB_CONFIG.database}`);
    console.log(`  用户: ${DB_CONFIG.user}\n`);

    const client = new Client(DB_CONFIG);
    
    try {
        await client.connect();
        console.log('✅ 数据库连接成功\n');
        
        // 1. 检查所有表
        console.log('📋 检查数据库表结构...');
        const tables = await client.query(`
            SELECT table_name, table_type 
            FROM information_schema.tables 
            WHERE table_schema = 'public'
            ORDER BY table_name
        `);
        console.log(`找到 ${tables.rows.length} 个表:`);
        tables.rows.forEach(table => {
            console.log(`  - ${table.table_name}`);
        });
        
        // 2. 检查设备表数据
        console.log('\n📋 设备表详细数据:');
        const devices = await client.query(`
            SELECT id, name, protocol, endpoint, enabled, created_at 
            FROM devices 
            ORDER BY created_at DESC
        `);
        console.log(`设备总数: ${devices.rows.length}`);
        
        if (devices.rows.length > 0) {
            devices.rows.forEach((device, i) => {
                console.log(`  ${i+1}. ${device.name}`);
                console.log(`     协议: ${device.protocol}`);
                console.log(`     端点: ${device.endpoint}`);
                console.log(`     状态: ${device.enabled ? '启用' : '禁用'}`);
                console.log(`     创建时间: ${device.created_at}`);
                console.log('');
            });
        } else {
            console.log('  ⚠️  设备表为空');
        }
        
        // 3. 检查标签表数据
        console.log('📋 标签表详细数据:');
        const tags = await client.query(`
            SELECT t.id, t.name, t.address, t.data_type, t.unit, d.name as device_name, t.created_at
            FROM tags t 
            LEFT JOIN devices d ON t.device_id = d.id 
            ORDER BY t.created_at DESC
        `);
        console.log(`标签总数: ${tags.rows.length}`);
        
        if (tags.rows.length > 0) {
            tags.rows.slice(0, 15).forEach((tag, i) => {
                console.log(`  ${i+1}. ${tag.name}`);
                console.log(`     地址: ${tag.address}`);
                console.log(`     类型: ${tag.data_type}`);
                console.log(`     单位: ${tag.unit}`);
                console.log(`     设备: ${tag.device_name}`);
                console.log(`     创建时间: ${tag.created_at}`);
                console.log('');
            });
            
            if (tags.rows.length > 15) {
                console.log(`  ... 还有 ${tags.rows.length - 15} 个标签`);
            }
        } else {
            console.log('  ⚠️  标签表为空');
        }
        
        // 4. 特别检查Modbus相关数据
        console.log('🔍 检查Modbus相关数据:');
        const modbusDevices = await client.query(`
            SELECT * FROM devices 
            WHERE protocol LIKE '%Modbus%' OR protocol LIKE '%modbus%'
            ORDER BY created_at DESC
        `);
        console.log(`Modbus设备数量: ${modbusDevices.rows.length}`);
        
        if (modbusDevices.rows.length > 0) {
            modbusDevices.rows.forEach((device, i) => {
                console.log(`  ${i+1}. ${device.name}`);
                console.log(`     协议: ${device.protocol}`);
                console.log(`     端点: ${device.endpoint}`);
                console.log(`     配置: ${JSON.stringify(device.config, null, 6)}`);
                console.log('');
            });
        } else {
            console.log('  ⚠️  没有找到Modbus设备');
        }
        
        // 5. 特别检查40001-40011地址范围的标签
        console.log('🔍 检查40001-40011地址范围的标签:');
        const addressTags = await client.query(`
            SELECT * FROM tags 
            WHERE address LIKE '%4000%' OR address LIKE '%4001%'
            ORDER BY address
        `);
        console.log(`相关地址标签数量: ${addressTags.rows.length}`);
        
        if (addressTags.rows.length > 0) {
            addressTags.rows.forEach((tag, i) => {
                console.log(`  ${i+1}. ${tag.name} - 地址: ${tag.address}`);
            });
        } else {
            console.log('  ⚠️  没有找到40001-40011地址范围的标签');
        }
        
        // 6. 数据库统计信息
        console.log('\n📊 数据库统计:');
        console.log(`  总设备数: ${devices.rows.length}`);
        console.log(`  总标签数: ${tags.rows.length}`);
        console.log(`  Modbus设备数: ${modbusDevices.rows.length}`);
        console.log(`  相关地址标签数: ${addressTags.rows.length}`);
        
        return {
            totalDevices: devices.rows.length,
            totalTags: tags.rows.length,
            modbusDevices: modbusDevices.rows.length,
            addressTags: addressTags.rows.length,
            devices: devices.rows,
            tags: tags.rows.slice(0, 10),
            modbusDevicesData: modbusDevices.rows
        };
        
    } catch (error) {
        console.log('❌ 数据库验证失败:', error.message);
        console.log('错误详情:', error);
        return null;
    } finally {
        await client.end();
        console.log('\n数据库连接已关闭');
    }
}

async function main() {
    const result = await verifyDatabase();
    
    if (result) {
        console.log('\n✅ 数据库验证完成');
        console.log('您可以使用DBeaver连接数据库查看详细数据');
        console.log('\nDBeaver连接信息:');
        console.log(`  主机: ${DB_CONFIG.host}`);
        console.log(`  端口: ${DB_CONFIG.port}`);
        console.log(`  数据库: ${DB_CONFIG.database}`);
        console.log(`  用户名: ${DB_CONFIG.user}`);
        console.log(`  密码: ${DB_CONFIG.password}`);
    } else {
        console.log('\n❌ 数据库验证失败');
    }
}

main();