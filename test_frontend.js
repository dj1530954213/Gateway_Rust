// 测试前端页面渲染
const http = require('http');

const testPages = [
  '/',
  '/dashboard', 
  '/realtime',
  '/drivers',
  '/connectors',
  '/data-points',
  '/alerts',
  '/analytics',
  '/system',
  '/monitoring'
];

async function testPage(path) {
  return new Promise((resolve) => {
    const options = {
      hostname: 'localhost',
      port: 20016,
      path: path,
      method: 'GET'
    };

    const req = http.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        console.log(`✅ ${path}: Status ${res.statusCode}`);
        resolve({ path, status: res.statusCode, hasContent: data.length > 1000 });
      });
    });

    req.on('error', (e) => {
      console.error(`❌ ${path}: Error - ${e.message}`);
      resolve({ path, status: 'error', error: e.message });
    });

    req.setTimeout(5000, () => {
      console.error(`⏰ ${path}: Timeout`);
      req.destroy();
      resolve({ path, status: 'timeout' });
    });

    req.end();
  });
}

async function runTests() {
  console.log('🚀 Testing frontend pages...\n');
  
  for (const path of testPages) {
    await testPage(path);
    await new Promise(resolve => setTimeout(resolve, 100)); // Small delay
  }
  
  console.log('\n✅ Frontend page tests completed');
}

runTests();