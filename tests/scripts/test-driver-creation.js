const playwright = require('playwright');

(async () => {
  console.log('🎯 开始测试驱动创建流程...');
  
  // 启动浏览器
  const browser = await playwright.chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  try {
    // 导航到前端页面
    console.log('📍 导航到驱动列表页面...');
    await page.goto('http://localhost:50020/drivers');
    await page.waitForTimeout(2000);
    
    // 检查页面是否加载成功
    console.log('✅ 检查页面加载状态...');
    const title = await page.title();
    console.log('页面标题:', title);
    
    // 点击新增驱动按钮
    console.log('🔘 点击新增驱动按钮...');
    try {
      // 等待按钮出现并点击
      await page.waitForSelector('button:has-text("新增驱动")', { timeout: 5000 });
      await page.click('button:has-text("新增驱动")');
      console.log('✅ 成功点击新增驱动按钮');
    } catch (error) {
      console.log('❌ 无法找到新增驱动按钮，尝试其他选择器...');
      await page.click('text=新增驱动');
    }
    
    await page.waitForTimeout(2000);
    console.log('📍 当前URL:', page.url());
    
    // 等待表单页面加载
    console.log('📝 等待驱动表单页面加载...');
    await page.waitForURL(/.*\/drivers\/(new|\w+)/, { timeout: 10000 });
    console.log('✅ 表单页面已加载');
    
    // 填写驱动基本信息
    console.log('📝 填写驱动基本信息...');
    
    // 驱动名称
    await page.fill('input[placeholder*="名称"]', 'Modbus TCP测试驱动');
    console.log('✅ 已填写驱动名称');
    
    // 驱动描述
    await page.fill('textarea[placeholder*="描述"]', '这是一个用于测试的Modbus TCP驱动配置');
    console.log('✅ 已填写驱动描述');
    
    // 等待一下让数据填充完成
    await page.waitForTimeout(1000);
    
    // 点击保存按钮
    console.log('💾 点击保存按钮...');
    try {
      await page.waitForSelector('button:has-text("创建驱动")', { timeout: 5000 });
      await page.click('button:has-text("创建驱动")');
      console.log('✅ 成功点击创建驱动按钮');
    } catch (error) {
      console.log('⚠️ 尝试点击其他保存按钮...');
      await page.click('button[type="submit"]');
    }
    
    // 等待保存完成
    await page.waitForTimeout(3000);
    
    // 检查是否有成功消息
    console.log('🔍 检查保存结果...');
    const successMessage = await page.textContent('.el-message--success, .success-message, [class*="success"]').catch(() => null);
    if (successMessage) {
      console.log('✅ 发现成功消息:', successMessage);
    }
    
    // 检查是否返回到列表页面
    console.log('📍 检查是否返回列表页面...');
    await page.waitForTimeout(2000);
    const currentUrl = page.url();
    console.log('当前URL:', currentUrl);
    
    if (currentUrl.includes('/drivers') && !currentUrl.includes('/new')) {
      console.log('✅ 已返回驱动列表页面');
      
      // 检查新创建的驱动是否出现在列表中
      console.log('🔍 检查新驱动是否出现在列表中...');
      await page.waitForTimeout(2000);
      
      // 查找包含"Modbus TCP测试驱动"的表格行
      const driverRow = await page.locator('text=Modbus TCP测试驱动').first();
      const isVisible = await driverRow.isVisible();
      
      if (isVisible) {
        console.log('🎉 SUCCESS: 新创建的驱动已出现在列表中！');
        console.log('✅ 驱动创建流程测试通过');
      } else {
        console.log('❌ FAILED: 新创建的驱动未出现在列表中');
        
        // 截图以供调试
        await page.screenshot({ path: 'driver-creation-failed.png' });
        console.log('📸 已保存失败截图: driver-creation-failed.png');
      }
    } else {
      console.log('❌ 未能返回到驱动列表页面');
    }
    
  } catch (error) {
    console.error('❌ 测试过程中发生错误:', error.message);
    
    // 截图以供调试
    await page.screenshot({ path: 'driver-creation-error.png' });
    console.log('📸 已保存错误截图: driver-creation-error.png');
    
    // 获取控制台错误
    const errors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    if (errors.length > 0) {
      console.log('🐛 浏览器控制台错误:');
      errors.forEach(err => console.log('  -', err));
    }
  } finally {
    await browser.close();
    console.log('🏁 测试完成');
  }
})();