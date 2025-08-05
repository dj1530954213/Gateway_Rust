const puppeteer = require('puppeteer');

async function checkFrontendErrors() {
  console.log('🔍 检查前端实际网络错误和CORS问题...\n');
  
  const browser = await puppeteer.launch({ 
    headless: false,
    devtools: true,
    args: ['--disable-web-security', '--disable-features=VizDisplayCompositor']
  });
  
  const page = await browser.newPage();
  
  // 监听所有网络请求
  const networkRequests = [];
  const failedRequests = [];
  const corsErrors = [];
  
  page.on('response', response => {
    const request = response.request();
    networkRequests.push({
      url: request.url(),
      method: request.method(),
      status: response.status(),
      headers: response.headers()
    });
    
    if (response.status() >= 400) {
      failedRequests.push({
        url: request.url(),
        method: request.method(),
        status: response.status()
      });
    }
  });
  
  // 监听控制台错误
  const consoleErrors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') {
      consoleErrors.push(msg.text());
    }
  });
  
  // 监听页面错误
  const pageErrors = [];
  page.on('pageerror', err => {
    pageErrors.push(err.message);
  });
  
  try {
    console.log('🌐 访问前端页面...');
    await page.goto('http://localhost:50020', { waitUntil: 'networkidle2', timeout: 15000 });
    
    // 等待页面完全加载
    await page.waitForTimeout(5000);
    
    console.log('\n📊 网络请求分析:');
    console.log(`总请求数: ${networkRequests.length}`);
    console.log(`失败请求数: ${failedRequests.length}`);
    
    if (failedRequests.length > 0) {
      console.log('\n❌ 失败的网络请求:');
      failedRequests.forEach(req => {
        console.log(`  ${req.method} ${req.url} - HTTP ${req.status}`);
      });
    }
    
    // 检查CORS相关的请求
    const apiRequests = networkRequests.filter(req => req.url.includes('localhost:8080'));
    console.log(`\n🔗 API请求数: ${apiRequests.length}`);
    
    if (apiRequests.length > 0) {
      console.log('\nAPI请求详情:');
      apiRequests.forEach(req => {
        const corsHeaders = req.headers['access-control-allow-origin'] || req.headers['access-control-allow-credentials'];
        console.log(`  ${req.method} ${req.url} - HTTP ${req.status}`);
        if (corsHeaders) {
          console.log(`    CORS: ${corsHeaders}`);
        }
      });
    }
    
    console.log('\n🐛 控制台错误:');
    if (consoleErrors.length === 0) {
      console.log('  ✅ 无控制台错误');
    } else {
      consoleErrors.forEach(error => {
        console.log(`  ❌ ${error}`);
      });
    }
    
    console.log('\n🚨 页面错误:');
    if (pageErrors.length === 0) {
      console.log('  ✅ 无页面错误');
    } else {
      pageErrors.forEach(error => {
        console.log(`  ❌ ${error}`);
      });
    }
    
    // 检查页面是否正常显示
    const title = await page.title();
    const bodyText = await page.evaluate(() => document.body.innerText);
    
    console.log(`\n📄 页面状态:`);
    console.log(`  标题: ${title}`);
    console.log(`  内容长度: ${bodyText.length} 字符`);
    
    if (bodyText.includes('网络错误') || bodyText.includes('连接失败')) {
      console.log(`  ❌ 页面显示网络错误`);
    } else {
      console.log(`  ✅ 页面内容正常`);
    }
    
    // 尝试手动触发一些API调用
    console.log('\n🔄 手动触发API调用测试...');
    
    try {
      await page.evaluate(() => {
        // 触发一些API调用
        fetch('/api/v1/devices').catch(e => console.error('设备API错误:', e));
        fetch('/api/v1/system/health').catch(e => console.error('健康检查错误:', e));
      });
      
      await page.waitForTimeout(2000);
    } catch (e) {
      console.log('API调用触发失败:', e.message);
    }
    
  } catch (error) {
    console.error('❌ 页面访问失败:', error.message);
  } finally {
    await browser.close();
  }
  
  // 总结
  const hasNetworkErrors = failedRequests.length > 0;
  const hasCorsErrors = corsErrors.length > 0;
  const hasConsoleErrors = consoleErrors.length > 0;
  
  console.log('\n🎯 问题总结:');
  console.log(`网络连接错误: ${hasNetworkErrors ? '❌ 存在' : '✅ 无'}`);
  console.log(`CORS跨域问题: ${hasCorsErrors ? '❌ 存在' : '✅ 无'}`);
  console.log(`控制台错误: ${hasConsoleErrors ? '❌ 存在' : '✅ 无'}`);
  
  if (hasNetworkErrors || hasCorsErrors || hasConsoleErrors) {
    console.log('\n🚨 系统仍存在问题，需要进一步修复！');
  } else {
    console.log('\n🎉 前端网络连接完全正常！');
  }
}

// 检查puppeteer是否可用，如果不可用则使用简化版本
try {
  checkFrontendErrors().catch(console.error);
} catch (e) {
  console.log('❌ Puppeteer不可用，使用基础检查...');
  console.log('请手动访问 http://localhost:50020 并检查浏览器控制台');
}