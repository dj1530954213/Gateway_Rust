import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { ElMessage } from 'element-plus'

import { useAuthStore } from '@/stores/auth'
import Layout from '@/layouts/MainLayout.vue'

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/auth/LoginView.vue'),
    meta: {
      title: '登录',
      requiresAuth: false,
    },
  },
  {
    path: '/',
    component: Layout,
    redirect: '/dashboard',
    meta: {
      requiresAuth: true,
    },
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/dashboard/DashboardView.vue'),
        meta: {
          title: '仪表板',
          icon: 'Dashboard',
        },
      },
      {
        path: 'realtime',
        name: 'Realtime',
        component: () => import('@/views/realtime/RealtimeView.vue'),
        meta: {
          title: '实时数据',
          icon: 'Monitor',
        },
      },
      {
        path: 'drivers',
        name: 'Drivers',
        component: () => import('@/views/drivers/DriversView.vue'),
        redirect: '/drivers/list',
        meta: {
          title: '驱动管理',
          icon: 'Cpu',
        },
        children: [
          {
            path: 'list',
            name: 'DriversList',
            component: () => import('@/views/drivers/DriversList.vue'),
            meta: {
              title: '驱动列表',
            },
          },
          {
            path: 'create',
            name: 'DriversCreate',
            component: () => import('@/views/drivers/DriversForm.vue'),
            meta: {
              title: '创建驱动',
            },
          },
          {
            path: ':id/edit',
            name: 'DriversEdit',
            component: () => import('@/views/drivers/DriversForm.vue'),
            meta: {
              title: '编辑驱动',
            },
          },
          {
            path: ':id',
            name: 'DriversDetail',
            component: () => import('@/views/drivers/DriversDetail.vue'),
            meta: {
              title: '驱动详情',
            },
          },
        ],
      },
      {
        path: 'devices',
        name: 'Devices',
        component: () => import('@/views/devices/DevicesView.vue'),
        meta: {
          title: '设备管理',
          icon: 'Monitor',
        },
      },
      {
        path: 'connectors',
        name: 'Connectors',
        component: () => import('@/views/connectors/ConnectorsView.vue'),
        redirect: '/connectors/list',
        meta: {
          title: '连接器管理',
          icon: 'Link',
        },
        children: [
          {
            path: 'list',
            name: 'ConnectorsList',
            component: () => import('@/views/connectors/ConnectorsList.vue'),
            meta: {
              title: '连接器列表',
            },
          },
          {
            path: 'create',
            name: 'ConnectorsCreate',
            component: () => import('@/views/connectors/ConnectorsForm.vue'),
            meta: {
              title: '创建连接器',
            },
          },
          {
            path: ':id/edit',
            name: 'ConnectorsEdit',
            component: () => import('@/views/connectors/ConnectorsForm.vue'),
            meta: {
              title: '编辑连接器',
            },
          },
          {
            path: ':id',
            name: 'ConnectorsDetail',
            component: () => import('@/views/connectors/ConnectorsDetail.vue'),
            meta: {
              title: '连接器详情',
            },
          },
        ],
      },
      {
        path: 'tags',
        name: 'Tags',
        component: () => import('@/views/tags/TagsView.vue'),
        meta: {
          title: '点位管理',
          icon: 'SetUp',
        },
      },
      {
        path: 'data-points',
        name: 'DataPoints',
        component: () => import('@/views/data-points/DataPointsView.vue'),
        redirect: '/data-points/list',
        meta: {
          title: '数据点管理',
          icon: 'DataBoard',
        },
        children: [
          {
            path: 'list',
            name: 'DataPointsList',
            component: () => import('@/views/data-points/DataPointsList.vue'),
            meta: {
              title: '数据点列表',
            },
          },
          {
            path: 'create',
            name: 'DataPointsCreate',
            component: () => import('@/views/data-points/DataPointsForm.vue'),
            meta: {
              title: '创建数据点',
            },
          },
          {
            path: ':id/edit',
            name: 'DataPointsEdit',
            component: () => import('@/views/data-points/DataPointsForm.vue'),
            meta: {
              title: '编辑数据点',
            },
          },
        ],
      },
      {
        path: 'alerts',
        name: 'Alerts',
        component: () => import('@/views/alerts/AlertsView.vue'),
        redirect: '/alerts/list',
        meta: {
          title: '告警管理',
          icon: 'Warning',
        },
        children: [
          {
            path: 'list',
            name: 'AlertsList',
            component: () => import('@/views/alerts/AlertsList.vue'),
            meta: {
              title: '告警列表',
            },
          },
          {
            path: 'rules',
            name: 'AlertRules',
            component: () => import('@/views/alerts/AlertRules.vue'),
            meta: {
              title: '告警规则',
            },
          },
          {
            path: 'history',
            name: 'AlertHistory',
            component: () => import('@/views/alerts/AlertHistory.vue'),
            meta: {
              title: '告警历史',
            },
          },
        ],
      },
      {
        path: 'analytics',
        name: 'Analytics',
        component: () => import('@/views/analytics/AnalyticsView.vue'),
        meta: {
          title: '数据分析',
          icon: 'TrendCharts',
        },
      },
      {
        path: 'system',
        name: 'System',
        component: () => import('@/views/system/SystemView.vue'),
        redirect: '/system/config',
        meta: {
          title: '系统管理',
          icon: 'Setting',
        },
        children: [
          {
            path: 'config',
            name: 'SystemConfig',
            component: () => import('@/views/system/SystemConfig.vue'),
            meta: {
              title: '系统配置',
            },
          },
          {
            path: 'users',
            name: 'UserManagement',
            component: () => import('@/views/system/UserManagement.vue'),
            meta: {
              title: '用户管理',
              requiresRole: 'admin',
            },
          },
          {
            path: 'logs',
            name: 'SystemLogs',
            component: () => import('@/views/system/SystemLogs.vue'),
            meta: {
              title: '系统日志',
            },
          },
          {
            path: 'backup',
            name: 'SystemBackup',
            component: () => import('@/views/system/SystemBackup.vue'),
            meta: {
              title: '备份恢复',
              requiresRole: 'admin',
            },
          },
        ],
      },
      {
        path: 'monitoring',
        name: 'Monitoring',
        component: () => import('@/views/monitoring/MonitoringView.vue'),
        redirect: '/monitoring/health',
        meta: {
          title: '监控中心',
          icon: 'DataAnalysis',
        },
        children: [
          {
            path: 'metrics',
            name: 'SystemMetrics',
            component: () => import('@/views/monitoring/SystemMetrics.vue'),
            meta: {
              title: '性能指标',
            },
          },
          {
            path: 'health',
            name: 'HealthStatus',
            component: () => import('@/views/monitoring/HealthStatus.vue'),
            meta: {
              title: '健康状态',
            },
          },
        ],
      },
    ],
  },
  {
    path: '/diagnostic',
    name: 'Diagnostic',
    component: () => import('@/views/test/DiagnosticView.vue'),
    meta: {
      title: '系统诊断',
      requiresAuth: false,
    },
  },
  {
    path: '/route-test',
    name: 'RouteTest',
    component: () => import('@/views/test/RouteTestView.vue'),
    meta: {
      title: '路由测试',
      requiresAuth: false,
    },
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('@/views/error/NotFound.vue'),
    meta: {
      title: '页面未找到',
    },
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      return savedPosition
    } else {
      return { top: 0 }
    }
  },
})

// Route guards simplified for development
router.beforeEach(async (to, from, next) => {
  console.log('Navigating to:', to.path)
  
  // Set page title
  if (to.meta?.title) {
    document.title = `${to.meta.title} - Gateway Rust`
  }
  
  // 在Mock模式下跳过所有认证检查
  if (import.meta.env.VITE_ENABLE_MOCK === 'true') {
    console.log('Mock mode: skipping auth checks')
    next()
    return
  }
  
  const authStore = useAuthStore()
  
  // Skip auth for specific paths
  const skipAuthPaths = ['/login', '/diagnostic', '/route-test']
  
  if (!skipAuthPaths.includes(to.path) && 
      to.meta?.requiresAuth !== false && 
      !authStore.isAuthenticated) {
    next('/login')
    return
  }
  
  next()
})

// Error handling for development
router.onError((error) => {
  console.error('Router error:', error)
  console.error('Stack trace:', error.stack)
  
  // 在开发模式下不自动刷新页面，只记录错误
  if (import.meta.env.MODE === 'development') {
    // 提供更友好的错误信息
    let errorMessage = error.message
    if (error.message.includes('parentNode')) {
      errorMessage = 'DOM元素引用错误，通常由组件清理不当导致'
    } else if (error.message.includes('Cannot read properties of null')) {
      errorMessage = '空值引用错误，检查组件生命周期'
    } else if (error.message.includes('Cannot read properties of undefined')) {
      errorMessage = '未定义值引用错误，检查数据初始化'
    }
    
    ElMessage.error(`路由加载失败: ${errorMessage}`)
  }
})

export default router