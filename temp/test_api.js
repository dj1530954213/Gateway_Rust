// 简单的API测试脚本
const http = require('http');

// 测试API健康检查
const options = {
  hostname: 'localhost',
  port: 20009,
  path: '/api/v1/health',
  method: 'GET'
};

const req = http.request(options, (res) => {
  console.log(`✅ API Status Code: ${res.statusCode}`);
  
  let data = '';
  res.on('data', (chunk) => {
    data += chunk;
  });
  
  res.on('end', () => {
    console.log('✅ API Response:', data);
  });
});

req.on('error', (e) => {
  console.error(`❌ API Error: ${e.message}`);
});

req.end();

// 测试前端是否可访问
const frontendOptions = {
  hostname: 'localhost',
  port: 20016,
  path: '/',
  method: 'GET'
};

const frontendReq = http.request(frontendOptions, (res) => {
  console.log(`✅ Frontend Status Code: ${res.statusCode}`);
  console.log('✅ Frontend accessible');
});

frontendReq.on('error', (e) => {
  console.error(`❌ Frontend Error: ${e.message}`);
});

frontendReq.end();