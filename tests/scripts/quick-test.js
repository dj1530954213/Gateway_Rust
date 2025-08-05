/**
 * 快速CORS和WebSocket连接测试
 */

const { chromium } = require('playwright');

async function quickTest() {
    console.log('🔍 快速系统验证...');
    
    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext();
    const page = await context.newPage();
    
    const corsErrors = [];
    const websocketConnections = [];
    let successfulApiCalls = 0;
    
    // 监听控制台消息
    page.on('console', msg => {
        if (msg.type() === 'error') {
            if (msg.text().includes('CORS') || msg.text().includes('Access-Control')) {
                corsErrors.push(msg.text());
            }
        } else if (msg.type() === 'log') {
            if (msg.text().includes('WebSocket connected')) {
                websocketConnections.push(msg.text());
                console.log('✅ WebSocket连接成功');
            }
            if (msg.text().includes('API响应: 200')) {
                successfulApiCalls++;
            }
        }
    });
    
    try {
        console.log('访问前端页面...');
        await page.goto('http://localhost:50020', { waitUntil: 'networkidle', timeout: 10000 });
        
        // 等待页面初始化
        await page.waitForTimeout(5000);
        
        const title = await page.title();
        console.log(`页面标题: ${title}`);
        
        // 生成报告
        console.log('\n📊 测试结果:');
        console.log(`CORS错误数量: ${corsErrors.length}`);
        console.log(`WebSocket连接: ${websocketConnections.length > 0 ? '成功' : '失败'}`);
        console.log(`成功的API调用: ${successfulApiCalls}`);
        
        if (corsErrors.length === 0) {
            console.log('🎉 CORS问题已修复！');
        } else {
            console.log('❌ 仍有CORS问题:');
            corsErrors.slice(0, 3).forEach(error => console.log(`  - ${error}`));
        }
        
        // 测试驱动页面导航
        try {
            await page.click('text=驱动管理');
            await page.waitForTimeout(2000);
            console.log('✅ 驱动页面导航成功');
        } catch (e) {
            console.log('❌ 驱动页面导航失败');
        }
        
    } catch (error) {
        console.error('测试失败:', error.message);
    } finally {
        await browser.close();
        console.log('🏁 快速测试完成');
    }
}

quickTest().catch(console.error);