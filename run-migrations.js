#!/usr/bin/env node
/**
 * 手动运行数据库迁移
 */

const { Client } = require('pg');
const fs = require('fs');
const path = require('path');

// 数据库连接配置
const DB_CONFIG = {
    host: 'localhost',
    port: 5432,
    database: 'iot',
    user: 'postgres',
    password: 'postgres',
};

const MIGRATIONS_DIR = './schema/migrations';

async function runMigrations() {
    console.log('开始运行数据库迁移...');
    
    const client = new Client(DB_CONFIG);
    
    try {
        // 连接数据库
        console.log('连接数据库...');
        await client.connect();
        console.log('✅ 数据库连接成功');
        
        // 检查当前数据库状态
        console.log('\n检查现有表...');
        const tablesResult = await client.query(`
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public'
        `);
        
        const existingTables = tablesResult.rows.map(row => row.table_name);
        console.log('现有表:', existingTables);
        
        // 读取迁移文件
        console.log('\n读取迁移文件...');
        const migrationFiles = fs.readdirSync(MIGRATIONS_DIR)
            .filter(file => file.endsWith('.sql'))
            .sort();
        
        console.log('找到迁移文件:', migrationFiles);
        
        // 运行迁移
        for (const file of migrationFiles) {
            console.log(`\n运行迁移: ${file}`);
            
            const filePath = path.join(MIGRATIONS_DIR, file);
            const sql = fs.readFileSync(filePath, 'utf8');
            
            try {
                await client.query(sql);
                console.log(`  ✅ ${file} 执行成功`);
            } catch (error) {
                console.log(`  ❌ ${file} 执行失败: ${error.message}`);
                
                // 如果是"already exists"错误，则继续
                if (error.message.includes('already exists')) {
                    console.log(`  ℹ️  ${file} 已存在，跳过`);
                } else {
                    console.log(`  详细错误:`, error.message);
                }
            }
        }
        
        // 再次检查表结构
        console.log('\n迁移后的表结构:');
        const finalTablesResult = await client.query(`
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public'
        `);
        
        const finalTables = finalTablesResult.rows.map(row => row.table_name);
        console.log('所有表:', finalTables);
        
        // 检查关键表
        const requiredTables = ['devices', 'tags'];
        const missingTables = requiredTables.filter(table => !finalTables.includes(table));
        
        if (missingTables.length === 0) {
            console.log('✅ 所有必要的表都已创建');
            
            // 检查表结构
            for (const table of requiredTables) {
                console.log(`\n${table}表结构:`);
                const columnsResult = await client.query(`
                    SELECT column_name, data_type, is_nullable 
                    FROM information_schema.columns 
                    WHERE table_name = $1 AND table_schema = 'public'
                    ORDER BY ordinal_position
                `, [table]);
                
                columnsResult.rows.forEach(col => {
                    console.log(`  - ${col.column_name}: ${col.data_type} (nullable: ${col.is_nullable})`);
                });
            }
            
        } else {
            console.log('❌ 缺少必要的表:', missingTables);
        }
        
        console.log('\n🎉 数据库迁移完成');
        
    } catch (error) {
        console.error('❌ 迁移过程中出现错误:', error.message);
        console.error('详细错误:', error);
    } finally {
        await client.end();
        console.log('数据库连接已关闭');
    }
}

if (require.main === module) {
    runMigrations();
}