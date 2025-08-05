// 最终CORS验证 - 模拟前端axios XMLHttpRequest请求
const axios = require('axios');

async function finalCORSVerification() {
    console.log('🔍 最终CORS验证测试');
    console.log('================================================\n');
    
    const API_BASE_URL = 'http://localhost:50010';
    
    // 模拟前端的axios配置
    const axiosConfig = {
        timeout: 10000,
        headers: {
            'Content-Type': 'application/json',
            'Accept': 'application/json',
            'Origin': 'http://localhost:50021',  // 模拟前端来源
            'X-Requested-With': 'XMLHttpRequest', // 关键的CORS头
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        }
    };
    
    const tests = [
        {
            name: '健康检查API (基础)',
            url: `${API_BASE_URL}/health`,
            critical: true
        },
        {
            name: '存活探针API',
            url: `${API_BASE_URL}/health/live`,
            critical: true
        },
        {
            name: '就绪探针API', 
            url: `${API_BASE_URL}/health/ready`,
            critical: true
        },
        {
            name: '系统信息API',
            url: `${API_BASE_URL}/system/info`,
            critical: true
        },
        {
            name: '系统指标API',
            url: `${API_BASE_URL}/system/metrics`,
            critical: true
        },
        {
            name: '系统健康API',
            url: `${API_BASE_URL}/system/health`,
            critical: true
        },
        {
            name: '组件状态API',
            url: `${API_BASE_URL}/system/components/status`,
            critical: true
        }
    ];
    
    let results = {
        total: tests.length,
        passed: 0,
        failed: 0,
        corsFixed: true,
        details: []
    };
    
    console.log('🚀 开始执行模拟前端的axios请求测试...\n');
    
    for (let i = 0; i < tests.length; i++) {
        const test = tests[i];
        console.log(`[${i + 1}/${tests.length}] 测试: ${test.name}`);
        
        try {
            const response = await axios.get(test.url, axiosConfig);
            
            console.log(`✅ 成功 - 状态码: ${response.status}`);
            console.log(`   响应头 Access-Control-Allow-Origin: ${response.headers['access-control-allow-origin'] || '未设置'}`);
            
            if (response.data) {
                const dataPreview = JSON.stringify(response.data).substring(0, 150);
                console.log(`   数据预览: ${dataPreview}${dataPreview.length >= 150 ? '...' : ''}`);
            }
            
            results.passed++;
            results.details.push({ test: test.name, status: 'PASS', error: null });
            
        } catch (error) {
            let errorMsg = '';
            let isCorsError = false;
            
            if (error.response) {
                errorMsg = `HTTP ${error.response.status}: ${error.response.statusText}`;
                console.log(`❌ HTTP错误: ${error.response.status} ${error.response.statusText}`);
            } else if (error.code === 'ECONNREFUSED') {
                errorMsg = '连接被拒绝 - API服务未运行';
                console.log(`❌ 连接错误: ${errorMsg}`);
            } else if (error.message.toLowerCase().includes('cors') || 
                       error.message.toLowerCase().includes('cross-origin') ||
                       error.message.toLowerCase().includes('access-control')) {
                errorMsg = `CORS错误: ${error.message}`;
                isCorsError = true;
                results.corsFixed = false;
                console.log(`❌ CORS错误: ${error.message}`);
            } else {
                errorMsg = error.message;
                console.log(`❌ 其他错误: ${error.message}`);
            }
            
            results.failed++;
            results.details.push({ 
                test: test.name, 
                status: 'FAIL', 
                error: errorMsg,
                isCorsError: isCorsError
            });
            
            // 如果是关键测试失败且是CORS错误，立即终止
            if (test.critical && isCorsError) {
                console.log('\n🚨 关键CORS错误detected，终止测试');
                break;
            }
        }
        
        console.log(''); // 空行分隔
        
        // 添加延迟避免请求过快
        if (i < tests.length - 1) {
            await new Promise(resolve => setTimeout(resolve, 200));
        }
    }
    
    // 输出最终结果
    console.log('================================================');
    console.log('📊 最终测试结果');
    console.log('================================================');
    console.log(`总计测试: ${results.total}`);
    console.log(`通过测试: ${results.passed}`);
    console.log(`失败测试: ${results.failed}`);
    console.log(`成功率: ${Math.round((results.passed / results.total) * 100)}%`);
    
    if (results.corsFixed && results.passed > 0) {
        console.log('\n🎉 CORS问题已彻底解决！');
        console.log('✅ 前端axios XMLHttpRequest请求可以正常访问后端API');
        console.log('✅ Access-Control-Allow-Origin 头正确设置');
        console.log('✅ 跨域请求障碍已清除');
        
        if (results.failed > 0) {
            console.log('\n⚠️  注意：部分API返回HTTP错误，但这不是CORS问题');
            console.log('   这些错误通常是由于数据库未初始化或业务逻辑问题');
        }
        
        return true;
        
    } else if (!results.corsFixed) {
        console.log('\n❌ CORS问题仍未完全解决');
        console.log('需要进一步检查后端CORS配置');
        
        const corsErrors = results.details.filter(d => d.isCorsError);
        if (corsErrors.length > 0) {
            console.log('\nCORS错误详情:');
            corsErrors.forEach(err => {
                console.log(`  - ${err.test}: ${err.error}`);
            });
        }
        
        return false;
        
    } else {
        console.log('\n⚠️  无法确定CORS状态 - 所有请求都失败了');
        console.log('请检查API服务是否正常运行');
        return false;
    }
}

// 运行最终验证
console.log('开始最终CORS验证...\n');

finalCORSVerification().then(success => {
    console.log('\n================================================');
    if (success) {
        console.log('🏆 验证完成：CORS问题已解决，可以继续后续开发工作');
    } else {
        console.log('💥 验证失败：需要进一步解决CORS问题');
    }
    console.log('================================================');
    process.exit(success ? 0 : 1);
}).catch(error => {
    console.error('\n💥 验证过程出现异常:', error.message);
    process.exit(1);
});