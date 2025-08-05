#!/usr/bin/env node
const { Client } = require('pg');

async function checkSchema() {
  const client = new Client({
    host: 'localhost',
    port: 5432,
    database: 'iot',
    user: 'postgres',
    password: 'postgres'
  });
  
  try {
    await client.connect();
    
    console.log('📋 devices表结构:');
    const deviceSchema = await client.query(`
      SELECT column_name, data_type, is_nullable
      FROM information_schema.columns 
      WHERE table_name = 'devices' 
      ORDER BY ordinal_position
    `);
    deviceSchema.rows.forEach(col => {
      console.log(`  - ${col.column_name}: ${col.data_type} (${col.is_nullable})`);
    });
    
    console.log('\n📋 tags表结构:');
    const tagSchema = await client.query(`
      SELECT column_name, data_type, is_nullable
      FROM information_schema.columns 
      WHERE table_name = 'tags' 
      ORDER BY ordinal_position
    `);
    tagSchema.rows.forEach(col => {
      console.log(`  - ${col.column_name}: ${col.data_type} (${col.is_nullable})`);
    });
    
    console.log('\n📊 实际数据数量:');
    const deviceCount = await client.query('SELECT COUNT(*) as count FROM devices');
    const tagCount = await client.query('SELECT COUNT(*) as count FROM tags');
    console.log(`  设备数量: ${deviceCount.rows[0].count}`);
    console.log(`  标签数量: ${tagCount.rows[0].count}`);
    
  } catch (error) {
    console.log('❌ 错误:', error.message);
  } finally {
    await client.end();
  }
}

checkSchema();