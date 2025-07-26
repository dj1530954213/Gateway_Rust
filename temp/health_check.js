// 最小的健康检查脚本 - 为前端提供基础API响应
const express = require('express');
const cors = require('cors');

const app = express();
const PORT = 8080;

// 启用CORS和JSON解析
app.use(cors());
app.use(express.json());

// 基础日志中间件
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} ${req.method} ${req.url}`);
  next();
});

// 健康检查端点
app.get('/api/v1/health', (req, res) => {
  res.json({
    success: true,
    data: {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      version: '1.0.0'
    },
    message: 'Service is healthy'
  });
});

// 登录端点 - 返回真实格式但简化的响应
app.post('/api/v1/auth/login', (req, res) => {
  const { username, password } = req.body;
  
  // 对于测试，接受任何凭据
  res.json({
    success: true,
    data: {
      token: 'real-token-' + Date.now(),
      user: {
        id: 1,
        username: username,
        full_name: username,
        role: 'admin',
        email: `${username}@gateway.local`,
        created_at: new Date().toISOString(),
        permissions: ['*']
      }
    },
    message: '登录成功'
  });
});

// 用户信息端点
app.get('/api/v1/auth/user', (req, res) => {
  res.json({
    success: true,
    data: {
      id: 1,
      username: 'admin',
      full_name: 'Admin User',
      role: 'admin',
      email: 'admin@gateway.local',
      created_at: new Date().toISOString(),
      permissions: ['*']
    }
  });
});

// 登出端点
app.post('/api/v1/auth/logout', (req, res) => {
  res.json({
    success: true,
    message: '登出成功'
  });
});

// 驱动列表端点 - 返回空数组避免错误
app.get('/api/v1/drivers', (req, res) => {
  res.json({
    success: true,
    data: [],
    message: 'Drivers loaded successfully'
  });
});

// 连接器列表端点
app.get('/api/v1/connectors', (req, res) => {
  res.json({
    success: true,
    data: [],
    message: 'Connectors loaded successfully'
  });
});

// 数据点列表端点
app.get('/api/v1/data-points', (req, res) => {
  res.json({
    success: true,
    data: [],
    message: 'Data points loaded successfully'
  });
});

// 告警列表端点
app.get('/api/v1/alerts', (req, res) => {
  res.json({
    success: true,
    data: [],
    message: 'Alerts loaded successfully'
  });
});

// 系统指标端点
app.get('/api/v1/system/metrics', (req, res) => {
  res.json({
    success: true,
    data: {
      cpu_usage: 0,
      memory_usage: 0,
      disk_usage: 0,
      network_rx_bytes: 0,
      network_tx_bytes: 0,
      uptime_seconds: 3600
    },
    message: 'System metrics loaded successfully'
  });
});

// 系统健康状态端点
app.get('/api/v1/system/health', (req, res) => {
  res.json({
    success: true,
    data: {
      overall: 'good',
      components: [
        {
          name: 'API Server',
          status: 'good',
          uptime: 3600,
          error_count: 0,
          metrics: {
            response_time: '10ms',
            requests_per_second: 50
          }
        },
        {
          name: 'Database',
          status: 'good', 
          uptime: 3600,
          error_count: 0,
          metrics: {
            connection_pool: 10,
            query_time: '5ms'
          }
        },
        {
          name: 'MQTT Broker',
          status: 'good',
          uptime: 3600,
          error_count: 0,
          metrics: {
            connected_clients: 5,
            message_rate: 100
          }
        }
      ],
      last_check: new Date().toISOString()
    },
    message: 'System health status retrieved successfully'
  });
});

// 系统信息端点
app.get('/api/v1/system/info', (req, res) => {
  res.json({
    success: true,
    data: {
      version: '1.0.0',
      build_time: '2024-01-01T00:00:00Z',
      runtime: 'Node.js',
      platform: process.platform,
      arch: process.arch,
      hostname: require('os').hostname(),
      uptime: process.uptime()
    },
    message: 'System info loaded successfully'
  });
});

// 404处理
app.use((req, res) => {
  res.status(404).json({
    success: false,
    message: `Endpoint not found: ${req.originalUrl}`,
    error: 'Not Found'
  });
});

app.listen(PORT, () => {
  console.log(`Health Check API server running on http://localhost:${PORT}`);
  console.log(`Health endpoint available at: http://localhost:${PORT}/api/v1/health`);
  console.log('This is a minimal API server to test frontend functionality');
});