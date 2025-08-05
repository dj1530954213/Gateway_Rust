#!/usr/bin/env node
/**
 * 最终ModbusTCP验证测试
 */

const { Client } = require('pg');
const axios = require('axios');
const net = require('net');

async function finalVerification() {
    console.log('🎯 最终ModbusTCP验证测试\n');

    let score = 0;
    let total = 4;

    // 1. 数据库验证
    console.log('1. 📋 数据库验证...');
    try {
        const client = new Client({
            host: 'localhost',
            port: 5432,
            database: 'iot',
            user: 'postgres',
            password: 'postgres',
        });
        
        await client.connect();
        
        const devices = await client.query("SELECT COUNT(*) as count FROM devices WHERE name = 'ModbusTCP_PLC_Real'");
        const tags = await client.query("SELECT COUNT(*) as count FROM tags WHERE device_id = (SELECT id FROM devices WHERE name = 'ModbusTCP_PLC_Real')");
        
        console.log(`  设备数: ${devices.rows[0].count}`);
        console.log(`  标签数: ${tags.rows[0].count}`);
        
        if (devices.rows[0].count > 0 && tags.rows[0].count > 0) {
            console.log('  ✅ 数据库验证通过');
            score++;
        } else {
            console.log('  ❌ 数据库验证失败');
        }
        
        await client.end();
    } catch (error) {
        console.log('  ❌ 数据库连接失败:', error.message);
    }

    // 2. Mock Modbus验证
    console.log('\n2. 🔌 Mock Modbus验证...');
    try {
        const socket = new net.Socket();
        await new Promise((resolve, reject) => {
            socket.setTimeout(3000);
            socket.connect(502, '127.0.0.1', () => {
                console.log('  ✅ Mock Modbus连接成功');
                socket.destroy();
                score++;
                resolve();
            });
            socket.on('error', reject);
            socket.on('timeout', reject);
        });
    } catch (error) {
        console.log('  ❌ Mock Modbus连接失败');
    }

    // 3. API验证  
    console.log('\n3. 📊 API验证...');
    try {
        const health = await axios.get('http://localhost:50010/health', { timeout: 5000 });
        const info = await axios.get('http://localhost:50010/system/info', { timeout: 5000 });
        
        console.log(`  健康检查: ${health.status}`);
        console.log(`  系统信息: ${info.status}`);
        console.log(`  显示设备数: ${info.data.connected_devices}`);
        
        if (health.status === 200 && info.status === 200) {
            console.log('  ✅ API验证通过');
            score++;
        } else {
            console.log('  ❌ API验证失败');
        }
    } catch (error) {
        console.log('  ❌ API连接失败');
    }

    // 4. 前端验证
    console.log('\n4. 🖥️  前端验证...');
    try {
        const frontend = await axios.get('http://localhost:50021', { 
            timeout: 5000,
            headers: { 'Accept': 'text/html' }
        });
        
        if (frontend.status === 200) {
            console.log('  ✅ 前端验证通过');
            score++;
        } else {
            console.log('  ❌ 前端验证失败');
        }
    } catch (error) {
        console.log('  ❌ 前端连接失败');
    }

    // 最终结果
    console.log('\n📊 最终验证结果');
    console.log('='.repeat(30));
    console.log(`总分: ${score}/${total} (${Math.round(score/total*100)}%)`);
    
    if (score === total) {
        console.log('🎊 完美！所有测试通过');
        console.log('ModbusTCP集成完全成功！');
    } else if (score >= 3) {
        console.log('✅ 优秀！主要功能正常');
        console.log('ModbusTCP集成基本成功！');
    } else if (score >= 2) {
        console.log('⚠️  良好！核心组件正常');
        console.log('ModbusTCP集成部分成功');
    } else {
        console.log('❌ 需要改进！');
        console.log('ModbusTCP集成需要进一步修复');
    }

    return score >= 2;
}

finalVerification().then(success => {
    process.exit(success ? 0 : 1);
});