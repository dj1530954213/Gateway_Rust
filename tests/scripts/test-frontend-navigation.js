const playwright = require('playwright');

(async () => {
  console.log('🎯 测试前端驱动创建导航...');
  
  // 启动浏览器
  const browser = await playwright.chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  try {
    // 导航到前端页面 - 使用新的端口
    console.log('📍 导航到驱动列表页面...');
    await page.goto('http://localhost:50024/drivers');
    await page.waitForTimeout(3000);
    
    // 检查页面是否加载成功
    console.log('✅ 检查页面加载状态...');
    const title = await page.title();
    console.log('页面标题:', title);
    
    // 获取当前URL
    console.log('📍 当前URL:', page.url());
    
    // 点击新增驱动按钮
    console.log('🔘 点击新增驱动按钮...');
    try {
      // 等待按钮出现
      await page.waitForSelector('button:has-text("新增驱动")', { timeout: 10000 });
      console.log('✅ 找到新增驱动按钮');
      
      // 截图以供调试
      await page.screenshot({ path: 'before-click.png' });
      console.log('📸 已保存点击前截图: before-click.png');
      
      // 点击按钮
      await page.click('button:has-text("新增驱动")');
      console.log('✅ 成功点击新增驱动按钮');
      
      // 等待页面导航完成
      await page.waitForTimeout(3000);
      
      // 检查是否导航到了创建页面
      const currentUrl = page.url();
      console.log('📍 点击后的URL:', currentUrl);
      
      if (currentUrl.includes('/drivers/create')) {
        console.log('🎉 SUCCESS: 成功导航到驱动创建页面!');
        
        // 截图确认页面内容
        await page.screenshot({ path: 'create-page.png' });
        console.log('📸 已保存创建页面截图: create-page.png');
        
        // 检查页面标题
        const pageHeader = await page.textContent('h1').catch(() => null);
        if (pageHeader) {
          console.log('📋 页面标题:', pageHeader);
        }
        
        // 检查是否有表单元素
        const nameInput = await page.locator('input[placeholder*="名称"]').first();
        const isVisible = await nameInput.isVisible().catch(() => false);
        
        if (isVisible) {
          console.log('✅ 确认: 驱动配置表单已加载');
        } else {
          console.log('⚠️ 警告: 未找到驱动名称输入框');
        }
        
      } else {
        console.log('❌ FAILED: 没有导航到创建页面');
        console.log('期望包含: /drivers/create');
        console.log('实际URL:', currentUrl);
        
        // 截图以供调试
        await page.screenshot({ path: 'navigation-failed.png' });
        console.log('📸 已保存失败截图: navigation-failed.png');
      }
      
    } catch (error) {
      console.log('❌ 无法找到或点击新增驱动按钮:', error.message);
      
      // 截图以供调试
      await page.screenshot({ path: 'button-not-found.png' });
      console.log('📸 已保存错误截图: button-not-found.png');
      
      // 尝试列出页面上的所有按钮
      const buttons = await page.locator('button').all();
      console.log(`📋 页面上找到 ${buttons.length} 个按钮:`);
      for (let i = 0; i < Math.min(buttons.length, 10); i++) {
        const text = await buttons[i].textContent().catch(() => '');
        console.log(`  ${i + 1}. "${text}"`);
      }
    }
    
  } catch (error) {
    console.error('❌ 测试过程中发生错误:', error.message);
    
    // 截图以供调试
    await page.screenshot({ path: 'test-error.png' });
    console.log('📸 已保存错误截图: test-error.png');
  } finally {
    await browser.close();
    console.log('🏁 测试完成');
  }
})();