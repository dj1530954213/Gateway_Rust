const http = require('http');
const fs = require('fs');

// 系统验证脚本
console.log('🚀 开始系统验证...\n');

const checks = [
  {
    name: '后端健康检查',
    test: () => checkHttp('http://localhost:8080/health')
  },
  {
    name: '后端设备API',
    test: () => checkHttp('http://localhost:8080/api/v1/devices')
  },
  {
    name: '前端界面',
    test: () => checkHttp('http://localhost:50020')
  },
  {
    name: 'Docker PostgreSQL',
    test: () => checkDockerContainer('gateway-postgres')
  }
];

function checkHttp(url) {
  return new Promise((resolve) => {
    const options = {
      timeout: 5000,
      headers: { 'User-Agent': 'System-Verifier' }
    };
    
    const req = http.get(url, options, (res) => {
      resolve({
        success: true,
        status: res.statusCode,
        message: `HTTP ${res.statusCode}`
      });
    });
    
    req.on('error', (err) => {
      resolve({
        success: false,
        message: `连接失败: ${err.message}`
      });
    });
    
    req.on('timeout', () => {
      req.destroy();
      resolve({
        success: false,
        message: '连接超时'
      });
    });
  });
}

function checkDockerContainer(containerName) {
  return new Promise((resolve) => {
    const { exec } = require('child_process');
    exec(`docker ps --format "table {{.Names}}" | findstr ${containerName}`, (error, stdout) => {
      if (error) {
        resolve({
          success: false,
          message: `Docker命令失败: ${error.message}`
        });
      } else if (stdout.trim().includes(containerName)) {
        resolve({
          success: true,
          message: '容器正在运行'
        });
      } else {
        resolve({
          success: false,
          message: '容器未运行'
        });
      }
    });
  });
}

async function runChecks() {
  const results = {};
  
  for (const check of checks) {
    console.log(`🔍 检查: ${check.name}...`);
    try {
      const result = await check.test();
      results[check.name] = result;
      
      if (result.success) {
        console.log(`✅ ${check.name}: ${result.message}`);
      } else {
        console.log(`❌ ${check.name}: ${result.message}`);
      }
    } catch (error) {
      results[check.name] = {
        success: false,
        message: `测试异常: ${error.message}`
      };
      console.log(`❌ ${check.name}: 测试异常: ${error.message}`);
    }
    console.log('');
  }
  
  // 生成报告
  const successCount = Object.values(results).filter(r => r.success).length;
  const totalCount = Object.keys(results).length;
  
  console.log('📊 系统状态总结:');
  console.log('='.repeat(40));
  
  for (const [name, result] of Object.entries(results)) {
    const status = result.success ? '✅ 正常' : '❌ 异常';
    console.log(`${name}: ${status} (${result.message})`);
  }
  
  console.log('='.repeat(40));
  console.log(`🎯 系统健康度: ${successCount}/${totalCount} (${Math.round(successCount/totalCount*100)}%)`);
  
  if (successCount === totalCount) {
    console.log('🎉 所有组件正常，系统准备就绪！');
  } else if (successCount >= totalCount * 0.75) {
    console.log('⚠️ 大部分组件正常，存在少量问题');
  } else {
    console.log('🚨 系统存在严重问题，需要立即修复');
  }
  
  // 保存结果到文件
  fs.writeFileSync('system-status.json', JSON.stringify({
    timestamp: new Date().toISOString(),
    results,
    health_score: successCount / totalCount
  }, null, 2));
  
  console.log('\n📄 详细结果已保存到 system-status.json');
}

runChecks().catch(console.error);