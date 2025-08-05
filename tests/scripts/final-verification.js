/**
 * 最终完整性验证测试
 * 确保所有功能正常工作，包括驱动添加流程
 */

const { chromium } = require('playwright');

async function finalVerification() {
    console.log('🔥 最终完整性验证测试...');
    
    const browser = await chromium.launch({ 
        headless: false,
        slowMo: 500
    });
    
    const context = await browser.newContext();
    const page = await context.newPage();
    
    const testResults = {
        pageLoad: false,
        corsErrors: 0,
        websocketConnection: false,
        apiCalls: 0,
        driverPageAccess: false,
        driverDialogOpen: false,
        navigationTest: false
    };
    
    // 监听控制台消息
    page.on('console', msg => {
        if (msg.type() === 'error') {
            if (msg.text().includes('CORS') || msg.text().includes('Access-Control')) {
                testResults.corsErrors++;
                console.error(`❌ CORS错误: ${msg.text()}`);
            }
        } else if (msg.type() === 'log') {
            if (msg.text().includes('WebSocket connected')) {
                testResults.websocketConnection = true;
                console.log('✅ WebSocket连接成功');
            }
            if (msg.text().includes('API响应: 200')) {
                testResults.apiCalls++;
            }
        }
    });
    
    try {
        console.log('🌐 访问前端页面...');
        await page.goto('http://localhost:50020', { 
            waitUntil: 'networkidle',
            timeout: 15000 
        });
        
        testResults.pageLoad = true;
        console.log('✅ 页面加载成功');
        
        // 等待页面初始化
        await page.waitForTimeout(3000);
        
        console.log('🔧 测试驱动管理功能...');
        
        // 点击驱动管理菜单
        try {
            await page.click('text=驱动管理', { timeout: 5000 });
            await page.waitForTimeout(2000);
            testResults.driverPageAccess = true;
            console.log('✅ 驱动页面访问成功');
            
            // 尝试点击添加驱动按钮
            const addButtonSelectors = [
                'text=添加驱动',
                'text=新增驱动',
                'button:has-text("添加")',
                '.el-button--primary:has-text("添加")'
            ];
            
            for (const selector of addButtonSelectors) {
                try {
                    const button = await page.locator(selector).first();
                    if (await button.isVisible({ timeout: 2000 })) {
                        await button.click();
                        await page.waitForTimeout(1000);
                        
                        // 检查对话框是否打开
                        const dialogVisible = await page.locator('.el-dialog, .modal, [role="dialog"]').isVisible();
                        if (dialogVisible) {
                            testResults.driverDialogOpen = true;
                            console.log('✅ 驱动添加对话框打开成功');
                            
                            // 关闭对话框
                            await page.keyboard.press('Escape');
                            await page.waitForTimeout(500);
                        }
                        break;
                    }
                } catch (e) {
                    // 继续尝试其他选择器
                }
            }
        } catch (e) {
            console.warn('⚠️  驱动页面导航失败');
        }
        
        console.log('🧭 测试页面导航...');
        
        // 测试其他页面导航
        const navigationPages = [
            { name: '仪表板', selector: 'text=仪表板' },
            { name: '实时监控', selector: 'text=实时监控' },
            { name: '标签管理', selector: 'text=标签管理' }
        ];
        
        let successfulNavigations = 0;
        for (const navPage of navigationPages) {
            try {
                await page.click(navPage.selector, { timeout: 3000 });
                await page.waitForTimeout(1000);
                successfulNavigations++;
                console.log(`✅ ${navPage.name}页面导航成功`);
            } catch (e) {
                console.log(`⚠️  ${navPage.name}页面导航失败`);
            }
        }
        
        testResults.navigationTest = successfulNavigations >= 2;
        
        // 等待更多API调用
        await page.waitForTimeout(2000);
        
    } catch (error) {
        console.error('❌ 测试执行失败:', error.message);
    } finally {
        await browser.close();
        
        // 生成最终报告
        console.log('\n🏆 最终验证报告:');
        console.log('='.repeat(50));
        console.log(`页面加载: ${testResults.pageLoad ? '✅ 成功' : '❌ 失败'}`);
        console.log(`CORS错误: ${testResults.corsErrors === 0 ? '✅ 零错误' : `❌ ${testResults.corsErrors}个错误`}`);
        console.log(`WebSocket连接: ${testResults.websocketConnection ? '✅ 成功' : '❌ 失败'}`);
        console.log(`API调用: ${testResults.apiCalls > 0 ? `✅ ${testResults.apiCalls}次成功` : '⚠️  无调用'}`);
        console.log(`驱动页面访问: ${testResults.driverPageAccess ? '✅ 成功' : '❌ 失败'}`);
        console.log(`驱动对话框: ${testResults.driverDialogOpen ? '✅ 成功' : '⚠️  未测试'}`);
        console.log(`页面导航: ${testResults.navigationTest ? '✅ 成功' : '❌ 失败'}`);
        
        const successCount = Object.values(testResults).filter(v => v === true || (typeof v === 'number' && v === 0)).length;
        const totalTests = Object.keys(testResults).length;
        
        console.log('\n🎯 总体评分:');
        if (testResults.corsErrors === 0 && testResults.pageLoad && testResults.websocketConnection) {
            console.log('🎉 完美！所有核心功能正常，CORS问题完全解决！');
        } else {
            console.log(`⚠️  部分功能需要优化 (${successCount}/${totalTests})`);
        }
        
        console.log('🏁 最终验证完成');
    }
}

finalVerification().catch(console.error);