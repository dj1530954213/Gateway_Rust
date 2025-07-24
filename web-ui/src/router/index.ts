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
        meta: {
          title: '驱动管理',
          icon: 'Connection',
        },
        children: [
          {
            path: '',
            name: 'DriversList',
            component: () => import('@/views/drivers/DriversList.vue'),
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
        path: 'connectors',
        name: 'Connectors',
        component: () => import('@/views/connectors/ConnectorsView.vue'),
        meta: {
          title: '连接器管理',
          icon: 'Link',
        },
        children: [
          {
            path: '',
            name: 'ConnectorsList',
            component: () => import('@/views/connectors/ConnectorsList.vue'),
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
        path: 'data-points',
        name: 'DataPoints',
        component: () => import('@/views/data-points/DataPointsView.vue'),
        meta: {
          title: '数据点配置',
          icon: 'SetUp',
        },
        children: [
          {
            path: '',
            name: 'DataPointsList',
            component: () => import('@/views/data-points/DataPointsList.vue'),
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
        meta: {
          title: '告警管理',
          icon: 'Warning',
        },
        children: [
          {
            path: '',
            name: 'AlertsList',
            component: () => import('@/views/alerts/AlertsList.vue'),
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

// Route guards
router.beforeEach((to, from, next) => {
  const authStore = useAuthStore()
  
  // Basic route logging for debugging
  console.log('Navigating to:', to.path, 'from:', from.path)
  console.log('Auth status:', authStore.isAuthenticated)
  
  // Set page title
  if (to.meta?.title) {
    document.title = `${to.meta.title} - 工控物联网边缘网关`
  }
  
  // Check authentication - skip for login page
  if (to.path !== '/login' && to.meta?.requiresAuth !== false && !authStore.isAuthenticated) {
    next('/login')
    return
  }
  
  // Check role requirements
  if (to.meta?.requiresRole && authStore.user?.role !== to.meta.requiresRole) {
    ElMessage.error('权限不足')
    next(from.path)
    return
  }
  
  // Redirect to dashboard if already authenticated and accessing login
  if (to.name === 'Login' && authStore.isAuthenticated) {
    next('/dashboard')
    return
  }
  
  next()
})

export default router