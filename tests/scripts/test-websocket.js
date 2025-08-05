const WebSocket = require('ws');

console.log('🔍 测试WebSocket连接...\n');

function testWebSocket(url, timeout = 5000) {
  return new Promise((resolve) => {
    console.log(`尝试连接: ${url}`);
    
    const ws = new WebSocket(url);
    const startTime = Date.now();
    
    let resolved = false;
    
    const resolveOnce = (result) => {
      if (!resolved) {
        resolved = true;
        if (ws.readyState === WebSocket.OPEN) {
          ws.close();
        }
        resolve(result);
      }
    };
    
    ws.on('open', () => {
      const time = Date.now() - startTime;
      console.log(`✅ WebSocket连接成功 (${time}ms)`);
      resolveOnce({ success: true, time, message: '连接成功' });
    });
    
    ws.on('error', (error) => {
      const time = Date.now() - startTime;
      console.log(`❌ WebSocket连接失败 (${time}ms): ${error.message}`);
      resolveOnce({ success: false, time, message: error.message });
    });
    
    ws.on('close', (code, reason) => {
      if (!resolved) {
        const time = Date.now() - startTime;
        console.log(`🔌 WebSocket连接关闭 (${time}ms): ${code} ${reason}`);
        resolveOnce({ success: false, time, message: `连接关闭: ${code}` });
      }
    });
    
    // 超时处理
    setTimeout(() => {
      if (!resolved) {
        console.log(`⏱️ WebSocket连接超时 (${timeout}ms)`);
        resolveOnce({ success: false, time: timeout, message: '连接超时' });
      }
    }, timeout);
  });
}

async function testAllWebSocketPaths() {
  const testUrls = [
    'ws://localhost:8080/ws',
    'ws://localhost:8080/ws/telemetry',
    'ws://localhost:8080/telemetry',
  ];
  
  console.log('测试不同的WebSocket路径:\n');
  
  for (const url of testUrls) {
    const result = await testWebSocket(url);
    console.log('');
  }
  
  console.log('🔍 检查WebSocket端点响应...\n');
  
  // 使用HTTP检查WebSocket端点
  const http = require('http');
  
  for (const path of ['/ws', '/ws/telemetry']) {
    const url = `http://localhost:8080${path}`;
    
    try {
      const result = await new Promise((resolve, reject) => {
        const req = http.get(url, { timeout: 3000 }, (res) => {
          resolve({
            status: res.statusCode,
            headers: res.headers
          });
        });
        req.on('error', reject);
        req.on('timeout', () => reject(new Error('Timeout')));
      });
      
      console.log(`HTTP ${path}: ${result.status}`);
      if (result.headers.upgrade) {
        console.log(`  支持升级到: ${result.headers.upgrade}`);
      }
    } catch (error) {
      console.log(`HTTP ${path}: 错误 - ${error.message}`);
    }
  }
}

// 安装ws模块或使用内置方法
try {
  testAllWebSocketPaths().catch(console.error);
} catch (e) {
  console.log('❌ 需要安装ws模块: npm install ws');
  console.log('使用基本HTTP测试...');
  
  // 基本HTTP测试WebSocket端点
  const http = require('http');
  
  ['/ws', '/ws/telemetry'].forEach(path => {
    const req = http.get(`http://localhost:8080${path}`, (res) => {
      console.log(`${path}: HTTP ${res.statusCode}`);
    });
    req.on('error', (err) => {
      console.log(`${path}: 错误 - ${err.message}`);
    });
  });
}