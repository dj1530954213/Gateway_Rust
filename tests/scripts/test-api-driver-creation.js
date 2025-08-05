const axios = require('axios');

async function testDriverConfigAPI() {
  console.log('🎯 开始测试驱动配置API...');
  
  const baseURL = 'http://localhost:8080';
  
  try {
    // 1. 测试健康检查
    console.log('📍 测试健康检查...');
    const healthResponse = await axios.get(`${baseURL}/health`);
    console.log('✅ 健康检查通过:', healthResponse.data);
    
    // 2. 测试创建驱动配置
    console.log('📝 测试创建驱动配置...');
    const createDriverConfig = {
      name: 'Modbus TCP测试驱动',
      description: '这是一个用于测试的Modbus TCP驱动配置',
      protocol: 'modbus_tcp',
      connection_type: 'ethernet',
      enabled: true,
      config: {
        host: '192.168.1.100',
        port: 502,
        unit_id: 1
      },
      scan_interval: 1000,
      timeout: 5000,
      max_concurrent: 10,
      batch_size: 100,
      max_retries: 3,
      retry_interval: 1000,
      exponential_backoff: true,
      max_retry_interval: 10000,
      log_level: 'info',
      enable_request_log: false,
      enable_response_log: false,
      max_log_size: 10,
      enable_ssl: false,
      verify_certificate: true
    };
    
    const createResponse = await axios.post(`${baseURL}/api/v1/driver-configs`, createDriverConfig, {
      headers: {
        'Content-Type': 'application/json'
      }
    });
    
    console.log('✅ 驱动配置创建成功!');
    console.log('📋 创建的驱动配置ID:', createResponse.data.driver_config.id);
    console.log('📋 驱动配置名称:', createResponse.data.driver_config.name);
    
    const driverId = createResponse.data.driver_config.id;
    
    // 3. 测试查询驱动配置列表  
    console.log('📋 测试查询驱动配置列表...');
    const listResponse = await axios.get(`${baseURL}/api/v1/driver-configs`);
    console.log('✅ 驱动配置列表查询成功!');
    console.log('📊 总数:', listResponse.data.total);
    console.log('📊 驱动配置数量:', listResponse.data.driver_configs.length);
    
    // 检查我们创建的驱动是否在列表中
    const createdDriver = listResponse.data.driver_configs.find(driver => driver.id === driverId);
    if (createdDriver) {
      console.log('🎉 SUCCESS: 新创建的驱动配置已出现在列表中!');
      console.log('📋 驱动配置详情:', {
        id: createdDriver.id,
        name: createdDriver.name,
        protocol: createdDriver.protocol,
        enabled: createdDriver.enabled
      });
    } else {
      console.log('❌ FAILED: 新创建的驱动配置未出现在列表中');
      return;
    }
    
    // 4. 测试获取单个驱动配置详情
    console.log('🔍 测试获取驱动配置详情...');
    const detailResponse = await axios.get(`${baseURL}/api/v1/driver-configs/${driverId}`);
    console.log('✅ 驱动配置详情查询成功!');
    console.log('📋 详情:', {
      name: detailResponse.data.driver_config.name,
      protocol: detailResponse.data.driver_config.protocol,
      config: detailResponse.data.driver_config.config
    });
    
    // 5. 测试更新驱动配置
    console.log('✏️ 测试更新驱动配置...');
    const updateRequest = {
      description: '已更新的Modbus TCP驱动配置描述',
      scan_interval: 2000
    };
    
    const updateResponse = await axios.put(`${baseURL}/api/v1/driver-configs/${driverId}`, updateRequest, {
      headers: {
        'Content-Type': 'application/json'
      }
    });
    
    console.log('✅ 驱动配置更新成功!');
    console.log('📋 更新后的描述:', updateResponse.data.driver_config.description);
    console.log('📋 更新后的扫描间隔:', updateResponse.data.driver_config.scan_interval);
    
    // 6. 清理 - 删除测试驱动配置
    console.log('🗑️ 清理测试数据...');
    await axios.delete(`${baseURL}/api/v1/driver-configs/${driverId}`);
    console.log('✅ 测试驱动配置已删除');
    
    console.log('🎉 所有测试通过! 驱动配置API工作正常!');
    
  } catch (error) {
    console.error('❌ 测试失败:', error.message);
    if (error.response) {
      console.error('📄 响应状态:', error.response.status);
      console.error('📄 响应数据:', error.response.data);
    }
    if (error.request) {
      console.error('📡 请求详情:', error.request.path);
    }
  }
}

// 运行测试
testDriverConfigAPI();