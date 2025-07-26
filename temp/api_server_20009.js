// API服务器 - 端口20009，对应Docker中的edge-gateway服务
const express = require('express');
const cors = require('cors');

const app = express();
const PORT = 20009;

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
          name: 'EMQX MQTT Broker',
          status: 'good', 
          uptime: 3600,
          error_count: 0,
          metrics: {
            connected_clients: 5,
            message_rate: 100,
            port: 20002
          }
        },
        {
          name: 'InfluxDB',
          status: 'good',
          uptime: 3600,
          error_count: 0,
          metrics: {
            database_size: '100MB',
            port: 20012
          }
        },
        {
          name: 'Grafana Dashboard',
          status: 'good',
          uptime: 3600,
          error_count: 0,
          metrics: {
            active_dashboards: 3,
            port: 20008
          }
        },
        {
          name: 'Prometheus Monitoring',
          status: 'good',
          uptime: 3600,
          error_count: 0,
          metrics: {
            metrics_collected: 1000,
            port: 20007
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
      runtime: 'Rust + Node.js',
      platform: process.platform,
      arch: process.arch,
      hostname: require('os').hostname(),
      uptime: process.uptime(),
      ports: {
        api: 20009,
        web: 20010,
        metrics: 20011,
        mqtt: 20002,
        grafana: 20008,
        prometheus: 20007,
        influxdb: 20012,
        plc_simulator: 20001,
        frontend: 20013
      }
    },
    message: 'System info loaded successfully'
  });
});

// 系统指标端点
app.get('/api/v1/system/metrics', (req, res) => {
  res.json({
    success: true,
    data: {
      cpu_usage: Math.random() * 100,
      memory_usage: Math.random() * 100,
      disk_usage: Math.random() * 100,
      network_rx_bytes: Math.floor(Math.random() * 1000000),
      network_tx_bytes: Math.floor(Math.random() * 1000000),
      uptime_seconds: 3600 + Math.floor(Math.random() * 1000),
      timestamp: new Date().toISOString()
    },
    message: 'System metrics loaded successfully'
  });
});

// 登录端点
app.post('/api/v1/auth/login', (req, res) => {
  const { username, password } = req.body;
  
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

// 驱动列表端点
app.get('/api/v1/drivers', (req, res) => {
  res.json({
    success: true,
    data: {
      items: [
        {
          id: '1',
          name: 'Modbus TCP Driver',
          type: 'modbus_tcp',
          status: 'running',
          connection: {
            host: 'localhost',
            port: 20001
          },
          data_points: 10,
          last_update: new Date().toISOString()
        }
      ],
      total: 1
    },
    message: 'Drivers loaded successfully'
  });
});

// 连接器列表端点
app.get('/api/v1/connectors', (req, res) => {
  res.json({
    success: true,
    data: {
      items: [
        {
          id: '1',
          name: 'MQTT Publisher',
          type: 'mqtt',
          status: 'connected',
          config: {
            broker_url: `mqtt://localhost:${20002}`,
            topic_prefix: 'iot/gateway'
          },
          messages_sent: 1500,
          last_activity: new Date().toISOString()
        }
      ],
      total: 1
    },
    message: 'Connectors loaded successfully'
  });
});

// 数据点列表端点
app.get('/api/v1/data-points', (req, res) => {
  res.json({
    success: true,
    data: {
      items: [
        {
          id: '1',
          name: 'Temperature_1',
          address: '40001',
          data_type: 'float',
          unit: '°C',
          current_value: 23.5,
          quality: 'good',
          last_update: new Date().toISOString()
        },
        {
          id: '2',
          name: 'Pressure_1',
          address: '40002',
          data_type: 'float',
          unit: 'bar',
          current_value: 1.2,
          quality: 'good',
          last_update: new Date().toISOString()
        }
      ],
      total: 2
    },
    message: 'Data points loaded successfully'
  });
});

// 告警列表端点
app.get('/api/v1/alerts', (req, res) => {
  res.json({
    success: true,
    data: {
      items: [],
      total: 0
    },
    message: 'Alerts loaded successfully'
  });
});

// 实时数据端点
app.get('/api/v1/realtime/current', (req, res) => {
  res.json({
    success: true,
    data: [
      {
        point_id: '1',
        value: 23.5 + (Math.random() - 0.5) * 2,
        quality: 'good',
        timestamp: new Date().toISOString()
      },
      {
        point_id: '2',
        value: 1.2 + (Math.random() - 0.5) * 0.2,
        quality: 'good',
        timestamp: new Date().toISOString()
      }
    ],
    message: 'Current values loaded successfully'
  });
});

// 分析统计端点
app.get('/api/v1/analytics/statistics', (req, res) => {
  res.json({
    success: true,
    data: {
      total_devices: 1,
      active_devices: 1,
      total_data_points: 2,
      alerts_count: 0,
      uptime_hours: Math.floor(process.uptime() / 3600)
    },
    message: 'Analytics loaded successfully'
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
  console.log(`🚀 Gateway API Server running on http://localhost:${PORT}`);
  console.log(`📊 Port Range: 20000-21000`);
  console.log(`🔗 Service Ports:`);
  console.log(`   - API Server: ${PORT}`);
  console.log(`   - Frontend: 20013`);
  console.log(`   - MQTT: 20002`);
  console.log(`   - Grafana: 20008`);
  console.log(`   - Prometheus: 20007`);
  console.log(`   - InfluxDB: 20012`);
  console.log(`   - PLC Simulator: 20001`);
  console.log(`✅ All endpoints ready for testing`);
});