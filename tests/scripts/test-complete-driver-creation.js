const playwright = require('playwright');

(async () => {
  console.log('🎯 完整测试驱动配置创建流程...');
  
  // 启动浏览器
  const browser = await playwright.chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  // 监听控制台错误
  const consoleErrors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') {
      consoleErrors.push(msg.text());
    }
  });
  
  // 监听网络请求
  const networkRequests = [];
  page.on('request', request => {
    if (request.url().includes('/api/v1/driver-configs')) {
      networkRequests.push({
        method: request.method(),
        url: request.url(),
        timestamp: new Date().toISOString()
      });
    }
  });
  
  page.on('response', response => {
    if (response.url().includes('/api/v1/driver-configs')) {
      console.log(`📡 API响应: ${response.status()} ${response.url()}`);
    }
  });
  
  try {
    // 1. 导航到驱动列表页面
    console.log('📍 步骤1: 导航到驱动列表页面...');
    await page.goto('http://localhost:50024/drivers');
    await page.waitForTimeout(2000);
    console.log('✅ 驱动列表页面已加载');
    
    // 2. 点击新增驱动按钮
    console.log('📍 步骤2: 点击新增驱动按钮...');
    await page.click('button:has-text("新增驱动")');
    await page.waitForTimeout(2000);
    
    // 验证导航到创建页面
    const createUrl = page.url();
    if (createUrl.includes('/drivers/create')) {
      console.log('✅ 成功导航到驱动创建页面');
    } else {
      throw new Error(`导航失败，当前URL: ${createUrl}`);
    }
    
    // 3. 填写驱动基本信息
    console.log('📍 步骤3: 填写驱动基本信息...');
    
    // 等待表单元素加载
    await page.waitForSelector('input', { timeout: 10000 });
    
    // 截图看看当前页面状态
    await page.screenshot({ path: 'form-page.png' });
    console.log('📸 已保存表单页面截图: form-page.png');
    
    // 尝试多种方式找到并填写驱动名称
    try {
      // 方法1: 通过placeholder查找
      const nameInputByPlaceholder = page.locator('input[placeholder*="名称"], input[placeholder*="驱动名称"]');
      if (await nameInputByPlaceholder.count() > 0) {
        await nameInputByPlaceholder.first().fill('Modbus TCP测试驱动配置');
        console.log('✅ 通过placeholder找到并填写了驱动名称');
      } else {
        // 方法2: 通过label查找
        const nameInputByLabel = page.locator('label:has-text("名称") + input, label:has-text("驱动名称") + input');
        if (await nameInputByLabel.count() > 0) {
          await nameInputByLabel.first().fill('Modbus TCP测试驱动配置');
          console.log('✅ 通过label找到并填写了驱动名称');
        } else {
          // 方法3: 通过索引查找第一个文本输入框
          const firstInput = page.locator('input[type="text"]').first();
          if (await firstInput.count() > 0) {
            await firstInput.fill('Modbus TCP测试驱动配置');
            console.log('✅ 通过索引找到第一个输入框并填写了驱动名称');
          } else {
            throw new Error('无法找到驱动名称输入框');
          }
        }
      }
    } catch (error) {
      console.log('⚠️ 填写驱动名称时遇到问题:', error.message);
      
      // 列出页面上的所有输入框
      const inputs = await page.locator('input').all();
      console.log(`📋 页面上找到 ${inputs.length} 个输入框:`);
      for (let i = 0; i < Math.min(inputs.length, 5); i++) {
        const placeholder = await inputs[i].getAttribute('placeholder').catch(() => '');
        const type = await inputs[i].getAttribute('type').catch(() => '');
        console.log(`  ${i + 1}. type="${type}" placeholder="${placeholder}"`);
      }
    }
    
    // 尝试填写描述
    try {
      const descTextarea = page.locator('textarea[placeholder*="描述"], textarea');
      if (await descTextarea.count() > 0) {
        await descTextarea.first().fill('这是一个用于测试的Modbus TCP驱动配置');
        console.log('✅ 已填写驱动描述');
      }
    } catch (error) {
      console.log('⚠️ 填写描述时遇到问题:', error.message);
    }
    
    await page.waitForTimeout(1000);
    
    // 4. 点击保存/创建按钮
    console.log('📍 步骤4: 点击创建驱动按钮...');
    
    try {
      // 尝试多种方式找到提交按钮
      const submitButtons = [
        'button:has-text("创建驱动")',
        'button:has-text("保存")',
        'button:has-text("提交")',
        'button[type="submit"]',
        'button:contains("创建")'
      ];
      
      let clicked = false;
      for (const selector of submitButtons) {
        try {
          const button = page.locator(selector);
          if (await button.count() > 0) {
            await button.first().click();
            console.log(`✅ 成功点击按钮: ${selector}`);
            clicked = true;
            break;
          }
        } catch (e) {
          continue;
        }
      }
      
      if (!clicked) {
        // 列出所有按钮
        const buttons = await page.locator('button').all();
        console.log(`📋 页面上找到 ${buttons.length} 个按钮:`);
        for (let i = 0; i < Math.min(buttons.length, 10); i++) {
          const text = await buttons[i].textContent().catch(() => '');
          console.log(`  ${i + 1}. "${text}"`);
        }
        
        throw new Error('无法找到提交按钮');
      }
      
    } catch (error) {
      console.log('❌ 点击提交按钮失败:', error.message);
    }
    
    // 5. 等待提交结果
    console.log('📍 步骤5: 等待提交结果...');
    await page.waitForTimeout(5000);
    
    // 检查是否有API请求
    if (networkRequests.length > 0) {
      console.log('📡 检测到API请求:');
      networkRequests.forEach(req => {
        console.log(`  ${req.method} ${req.url} at ${req.timestamp}`);
      });
    } else {
      console.log('⚠️ 未检测到API请求');
    }
    
    // 检查当前URL
    const finalUrl = page.url();
    console.log('📍 最终URL:', finalUrl);
    
    if (finalUrl.includes('/drivers') && !finalUrl.includes('/create')) {
      console.log('✅ 似乎已返回驱动列表页面');
    } else if (finalUrl.includes('/drivers/create')) {
      console.log('⚠️ 仍在创建页面，可能提交失败或正在处理');
    }
    
    // 检查是否有成功或错误消息
    try {
      const successMessage = await page.locator('.el-message--success, .success').first().textContent({ timeout: 2000 });
      if (successMessage) {
        console.log('✅ 发现成功消息:', successMessage);
      }
    } catch (e) {
      // 检查错误消息
      try {
        const errorMessage = await page.locator('.el-message--error, .error').first().textContent({ timeout: 2000 });
        if (errorMessage) {
          console.log('❌ 发现错误消息:', errorMessage);
        }
      } catch (e2) {
        console.log('📋 未发现明显的成功或错误消息');
      }
    }
    
    // 最终截图
    await page.screenshot({ path: 'final-state.png' });
    console.log('📸 已保存最终状态截图: final-state.png');
    
    // 6. 总结结果
    console.log('\n📊 测试总结:');
    console.log('✅ 前端导航: 成功');
    console.log('✅ 表单页面加载: 成功');
    console.log('⚠️ 表单填写: 部分成功（某些字段可能未找到）');
    console.log('⚠️ 表单提交: 尝试完成（结果待确认）');
    
    if (consoleErrors.length > 0) {
      console.log('\n🐛 控制台错误:');
      consoleErrors.forEach((error, i) => {
        console.log(`  ${i + 1}. ${error}`);
      });
    }
    
    console.log('\n🎉 主要问题已解决: "新增驱动"按钮现在正确导航到驱动配置创建页面！');
    
  } catch (error) {
    console.error('❌ 测试过程中发生错误:', error.message);
    
    // 错误截图
    await page.screenshot({ path: 'test-error-complete.png' });
    console.log('📸 已保存错误截图: test-error-complete.png');
  } finally {
    await browser.close();
    console.log('🏁 完整测试完成');
  }
})();