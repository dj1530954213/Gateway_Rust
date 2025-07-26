// 简单的模拟API服务器用于测试前端
const express = require('express');
const cors = require('cors');
const app = express();

app.use(cors());
app.use(express.json());

// 模拟登录端点
app.post('/api/v1/auth/login', (req, res) => {
  console.log('Login request:', req.body);
  
  // 简单验证
  if (req.body.username && req.body.password) {
    res.json({
      success: true,
      data: {
        token: 'mock-jwt-token-12345',
        user: {
          id: 1,
          username: req.body.username,
          email: 'user@example.com',
          role: 'admin'
        }
      },
      message: 'Login successful'
    });
  } else {
    res.status(400).json({
      success: false,
      message: 'Username and password required'
    });
  }
});

// 模拟用户信息获取
app.get('/api/v1/auth/user', (req, res) => {
  const authHeader = req.headers.authorization;
  
  if (authHeader && authHeader.startsWith('Bearer ')) {
    res.json({
      success: true,
      data: {
        id: 1,
        username: 'admin',
        email: 'admin@example.com',
        role: 'admin',
        full_name: '系统管理员'
      }
    });
  } else {
    res.status(401).json({
      success: false,
      message: 'Unauthorized'
    });
  }
});

// 模拟退出登录
app.post('/api/v1/auth/logout', (req, res) => {
  res.json({
    success: true,
    message: 'Logout successful'
  });
});

// 模拟健康检查
app.get('/api/v1/health', (req, res) => {
  res.json({
    success: true,
    data: {
      status: 'healthy',
      timestamp: new Date().toISOString()
    }
  });
});

// 模拟系统信息
app.get('/api/v1/system/info', (req, res) => {
  res.json({
    success: true,
    data: {
      version: '1.0.0',
      name: 'Edge Gateway',
      uptime: '2 hours 15 minutes',
      status: 'running',
      environment: 'docker'
    }
  });
});

// 模拟系统指标
app.get('/api/v1/system/metrics', (req, res) => {
  res.json({
    success: true,
    data: {
      cpuUsage: Math.floor(Math.random() * 30) + 20, // 20-50%
      memoryUsage: Math.floor(Math.random() * 25) + 40, // 40-65%
      diskUsage: Math.floor(Math.random() * 20) + 30, // 30-50%
      networkLatency: Math.floor(Math.random() * 50) + 10, // 10-60ms
      uptime: 8130, // seconds
      activeConnections: Math.floor(Math.random() * 15) + 5, // 5-20
      messagesPerSecond: Math.floor(Math.random() * 100) + 50 // 50-150
    }
  });
});

// 模拟驱动列表
app.get('/api/v1/drivers', (req, res) => {
  res.json({
    success: true,
    data: [
      {
        id: '1',
        name: 'Production Line PLC',
        type: 'modbus-tcp',
        status: 'running',
        endpoint: 'tcp://192.168.1.100:502',
        lastUpdate: new Date().toISOString(),
        dataPoints: 25
      },
      {
        id: '2', 
        name: 'Packaging Line Controller',
        type: 'modbus-tcp',
        status: 'running',
        endpoint: 'tcp://192.168.1.101:502',
        lastUpdate: new Date(Date.now() - 5000).toISOString(),
        dataPoints: 18
      },
      {
        id: '3',
        name: 'Power Meter Station 1',
        type: 'modbus-rtu',
        status: 'stopped',
        endpoint: 'COM3:9600',
        lastUpdate: new Date(Date.now() - 300000).toISOString(),
        dataPoints: 12
      }
    ]
  });
});

// 模拟连接器列表
app.get('/api/v1/connectors', (req, res) => {
  res.json({
    success: true,
    data: [
      {
        id: '1',
        name: 'MQTT Cloud Connector',
        type: 'mqtt5',
        status: 'connected',
        endpoint: 'tcp://broker.emqx.io:1883',
        lastActivity: new Date().toISOString()
      },
      {
        id: '2',
        name: 'HTTP API Connector', 
        type: 'http',
        status: 'disconnected',
        endpoint: 'https://api.example.com/webhook',
        lastActivity: new Date(Date.now() - 600000).toISOString()
      }
    ]
  });
});

// 模拟数据点
app.get('/api/v1/data-points', (req, res) => {
  const dataPoints = [];
  const pointNames = ['temperature', 'pressure', 'flow_rate', 'speed', 'vibration'];
  
  for (let i = 1; i <= 20; i++) {
    const pointName = pointNames[Math.floor(Math.random() * pointNames.length)];
    dataPoints.push({
      id: `dp_${i}`,
      name: `${pointName}_${String(i).padStart(3, '0')}`,
      address: `40${String(i).padStart(3, '0')}`,
      dataType: 'float32',
      value: (Math.random() * 100).toFixed(2),
      unit: pointName === 'temperature' ? '°C' : pointName === 'pressure' ? 'bar' : 'RPM',
      timestamp: new Date(Date.now() - Math.random() * 60000).toISOString(),
      quality: Math.random() > 0.1 ? 'good' : 'bad'
    });
  }
  
  res.json({
    success: true,
    data: dataPoints
  });
});

// 模拟告警
app.get('/api/v1/alerts', (req, res) => {
  res.json({
    success: true,
    data: [
      {
        id: '1',
        message: '生产线温度超过安全阈值 (85°C)',
        level: 'critical',
        source: 'Driver: Production Line PLC',
        created_at: new Date(Date.now() - 120000).toISOString(),
        acknowledged: false
      },
      {
        id: '2',
        message: 'MQTT连接器连接中断',
        level: 'warning',
        source: 'Connector: MQTT Cloud',
        created_at: new Date(Date.now() - 300000).toISOString(),
        acknowledged: true
      },
      {
        id: '3',
        message: '系统内存使用率达到80%',
        level: 'warning',
        source: 'System Monitor',
        created_at: new Date(Date.now() - 600000).toISOString(),
        acknowledged: false
      }
    ]
  });
});

// 模拟实时数据（用于图表）
app.get('/api/v1/data-points/:id/history', (req, res) => {
  const points = [];
  const now = Date.now();
  
  // 生成过去1小时的数据点
  for (let i = 60; i >= 0; i--) {
    points.push({
      timestamp: new Date(now - i * 60000).toISOString(),
      value: (Math.sin(i * 0.1) * 20 + 50 + Math.random() * 10).toFixed(2)
    });
  }
  
  res.json({
    success: true,
    data: points
  });
});

const PORT = 8080;
app.listen(PORT, () => {
  console.log(`Mock API server running on http://localhost:${PORT}`);
  console.log('Available endpoints:');
  console.log('- POST /api/v1/auth/login');
  console.log('- GET /api/v1/health');
  console.log('- GET /api/v1/system/info');
  console.log('- GET /api/v1/drivers');
});