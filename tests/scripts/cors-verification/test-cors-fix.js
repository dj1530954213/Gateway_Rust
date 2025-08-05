// 测试CORS修复是否生效
const axios = require('axios');

const API_BASE_URL = 'http://localhost:50010';

async function testCORS() {
    console.log('🔍 测试CORS配置是否生效...\n');
    
    const tests = [
        {
            name: '健康检查API',
            url: `${API_BASE_URL}/health`,
            method: 'GET'
        },
        {
            name: '系统信息API',
            url: `${API_BASE_URL}/system/info`,
            method: 'GET'
        },
        {
            name: '设备列表API',
            url: `${API_BASE_URL}/api/v1/devices`,
            method: 'GET'
        },
        {
            name: '标签列表API',
            url: `${API_BASE_URL}/api/v1/tags`,
            method: 'GET'
        }
    ];
    
    let passedTests = 0;
    let totalTests = tests.length;
    
    for (const test of tests) {
        try {
            console.log(`⏳ 测试: ${test.name}`);
            
            const response = await axios({
                method: test.method,
                url: test.url,
                timeout: 5000,
                headers: {
                    'Content-Type': 'application/json',
                    'Origin': 'http://localhost:50021',
                    'X-Requested-With': 'XMLHttpRequest'
                }
            });
            
            console.log(`✅ ${test.name} - 状态码: ${response.status}`);
            if (response.data) {
                console.log(`   响应数据: ${JSON.stringify(response.data).substring(0, 100)}...`);
            }
            passedTests++;
            
        } catch (error) {
            if (error.response) {
                console.log(`❌ ${test.name} - HTTP错误: ${error.response.status} ${error.response.statusText}`);
            } else if (error.code === 'ECONNREFUSED') {
                console.log(`❌ ${test.name} - 连接被拒绝，API服务未启动`);
            } else {
                console.log(`❌ ${test.name} - CORS错误: ${error.message}`);
            }
        }
        console.log('');
    }
    
    console.log(`📊 测试结果: ${passedTests}/${totalTests} 通过`);
    
    if (passedTests === totalTests) {
        console.log('🎉 所有API测试通过，CORS问题已解决！');
        return true;
    } else {
        console.log('⚠️  仍存在CORS或连接问题');
        return false;
    }
}

// 运行测试
testCORS().then(success => {
    process.exit(success ? 0 : 1);
}).catch(error => {
    console.error('测试执行失败:', error.message);
    process.exit(1);
});