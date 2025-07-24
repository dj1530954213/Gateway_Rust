<template>
  <div class="main-container">
    <el-container>
      <!-- Sidebar -->
      <el-aside :width="sidebarWidth" class="sidebar">
        <div class="logo-container">
          <img
            src="/logo.svg"
            alt="Edge Gateway"
            class="logo"
            @click="$router.push('/dashboard')"
          />
          <span v-if="!isCollapsed" class="logo-text">边缘网关</span>
        </div>
        
        <el-menu
          :default-active="activeMenu"
          :collapse="isCollapsed"
          :unique-opened="true"
          router
          background-color="var(--el-menu-bg-color)"
          text-color="var(--el-menu-text-color)"
          active-text-color="var(--el-color-primary)"
        >
          <template v-for="item in menuItems" :key="item.path">
            <el-sub-menu
              v-if="item.children && item.children.length > 0"
              :index="item.path"
            >
              <template #title>
                <el-icon><component :is="item.icon" /></el-icon>
                <span>{{ item.title }}</span>
              </template>
              <el-menu-item
                v-for="child in item.children"
                :key="child.path"
                :index="child.path"
              >
                {{ child.title }}
              </el-menu-item>
            </el-sub-menu>
            
            <el-menu-item v-else :index="item.path">
              <el-icon><component :is="item.icon" /></el-icon>
              <span>{{ item.title }}</span>
            </el-menu-item>
          </template>
        </el-menu>
      </el-aside>

      <!-- Main Content -->
      <el-container direction="vertical">
        <!-- Header -->
        <el-header class="main-header">
          <div class="header-left">
            <el-button
              type="text"
              size="large"
              @click="toggleSidebar"
            >
              <el-icon><Fold v-if="!isCollapsed" /><Expand v-else /></el-icon>
            </el-button>
            
            <el-breadcrumb separator="/">
              <el-breadcrumb-item
                v-for="item in breadcrumbs"
                :key="item.path"
                :to="item.path"
              >
                {{ item.title }}
              </el-breadcrumb-item>
            </el-breadcrumb>
          </div>

          <div class="header-right">
            <!-- System Status Indicator -->
            <el-tooltip
              :content="systemStatus.text"
              placement="bottom"
            >
              <el-badge
                :type="systemStatus.type"
                is-dot
                class="status-badge"
              >
                <el-icon size="20">
                  <Connection v-if="systemStatus.healthy" />
                  <Warning v-else />
                </el-icon>
              </el-badge>
            </el-tooltip>

            <!-- Alerts Dropdown -->
            <el-dropdown trigger="click" @command="handleAlertCommand">
              <el-badge :value="unreadAlertsCount" :hidden="unreadAlertsCount === 0">
                <el-button type="text" size="large">
                  <el-icon><Bell /></el-icon>
                </el-button>
              </el-badge>
              <template #dropdown>
                <el-dropdown-menu>
                  <div class="alerts-header">
                    <span>最新告警</span>
                    <el-button type="text" size="small" @click="markAllAlertsRead">
                      全部已读
                    </el-button>
                  </div>
                  <el-dropdown-item
                    v-for="alert in recentAlerts"
                    :key="alert.id"
                    :command="{ type: 'view', id: alert.id }"
                    :class="`alert-item alert-${alert.level}`"
                  >
                    <div class="alert-content">
                      <div class="alert-message">{{ alert.message }}</div>
                      <div class="alert-time">
                        {{ formatTime(alert.created_at) }}
                      </div>
                    </div>
                  </el-dropdown-item>
                  <el-dropdown-item command="view-all" divided>
                    查看全部告警
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>

            <!-- Theme Toggle -->
            <el-button
              type="text"
              size="large"
              @click="toggleTheme"
            >
              <el-icon>
                <Sunny v-if="systemStore.isDarkMode" />
                <Moon v-else />
              </el-icon>
            </el-button>

            <!-- User Dropdown -->
            <el-dropdown trigger="click" @command="handleUserCommand">
              <div class="user-avatar">
                <el-avatar size="small">
                  {{ authStore.userName?.[0]?.toUpperCase() }}
                </el-avatar>
                <span v-if="authStore.userName" class="user-name">
                  {{ authStore.userName }}
                </span>
                <el-icon><CaretBottom /></el-icon>
              </div>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="profile">
                    <el-icon><User /></el-icon>
                    个人信息
                  </el-dropdown-item>
                  <el-dropdown-item command="settings">
                    <el-icon><Setting /></el-icon>
                    系统设置
                  </el-dropdown-item>
                  <el-dropdown-item command="logout" divided>
                    <el-icon><SwitchButton /></el-icon>
                    退出登录
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </el-header>

        <!-- Main Content Area -->
        <el-main class="main-content">
          <div class="content-wrapper">
            <router-view v-slot="{ Component }">
              <transition name="fade-slide" mode="out-in">
                <component :is="Component" />
              </transition>
            </router-view>
          </div>
        </el-main>
      </el-container>
    </el-container>

    <!-- Global Loading -->
    <el-loading
      v-loading="globalLoading"
      :text="loadingText"
      background="rgba(0, 0, 0, 0.8)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElNotification } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { useSystemStore } from '@/stores/system'
import { useWebSocket } from '@/composables/useWebSocket'
import { formatTime } from '@/utils/date'
import type { Alert } from '@/types'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const systemStore = useSystemStore()

// Sidebar state
const isCollapsed = ref(false)
const sidebarWidth = computed(() => isCollapsed.value ? '64px' : '240px')

// Global loading
const globalLoading = ref(false)
const loadingText = ref('')

// Alerts
const recentAlerts = ref<Alert[]>([])
const unreadAlertsCount = computed(() => 
  recentAlerts.value.filter(alert => !alert.acknowledged).length
)

// WebSocket connection
const { isConnected, lastMessage } = useWebSocket('/ws')

// Menu configuration
const menuItems = computed(() => [
  {
    path: '/dashboard',
    title: '仪表板',
    icon: 'Dashboard',
  },
  {
    path: '/realtime',
    title: '实时数据',
    icon: 'Monitor',
  },
  {
    path: '/drivers',
    title: '驱动管理',
    icon: 'Connection',
    children: [
      { path: '/drivers', title: '驱动列表' },
      { path: '/drivers/create', title: '创建驱动' },
    ],
  },
  {
    path: '/connectors',
    title: '连接器管理',
    icon: 'Link',
    children: [
      { path: '/connectors', title: '连接器列表' },
      { path: '/connectors/create', title: '创建连接器' },
    ],
  },
  {
    path: '/data-points',
    title: '数据点配置',
    icon: 'SetUp',
  },
  {
    path: '/alerts',
    title: '告警管理',
    icon: 'Warning',
    children: [
      { path: '/alerts', title: '告警列表' },
      { path: '/alerts/rules', title: '告警规则' },
      { path: '/alerts/history', title: '告警历史' },
    ],
  },
  {
    path: '/analytics',
    title: '数据分析',
    icon: 'TrendCharts',
  },
  {
    path: '/system',
    title: '系统管理',
    icon: 'Setting',
    children: [
      { path: '/system/config', title: '系统配置' },
      { path: '/system/users', title: '用户管理' },
      { path: '/system/logs', title: '系统日志' },
      { path: '/system/backup', title: '备份恢复' },
    ].filter(item => {
      // Filter menu items based on user permissions
      if (item.path === '/system/users' && !authStore.hasRole('admin')) {
        return false
      }
      return true
    }),
  },
  {
    path: '/monitoring',
    title: '监控中心',
    icon: 'DataAnalysis',
    children: [
      { path: '/monitoring/metrics', title: '性能指标' },
      { path: '/monitoring/health', title: '健康状态' },
    ],
  },
])

// Active menu calculation
const activeMenu = computed(() => {
  const path = route.path
  // Find the best matching menu item
  for (const item of menuItems.value) {
    if (item.children) {
      for (const child of item.children) {
        if (path.startsWith(child.path)) {
          return child.path
        }
      }
    } else if (path.startsWith(item.path)) {
      return item.path
    }
  }
  return path
})

// Breadcrumbs
const breadcrumbs = computed(() => {
  const items = []
  const pathSegments = route.path.split('/').filter(Boolean)
  
  let currentPath = ''
  for (const segment of pathSegments) {
    currentPath += `/${segment}`
    
    // Find title for this path
    let title = segment
    for (const item of menuItems.value) {
      if (item.path === currentPath) {
        title = item.title
        break
      }
      if (item.children) {
        const child = item.children.find(c => c.path === currentPath)
        if (child) {
          title = child.title
          break
        }
      }
    }
    
    items.push({
      path: currentPath,
      title: title.charAt(0).toUpperCase() + title.slice(1),
    })
  }
  
  return items
})

// System status
const systemStatus = computed(() => {
  const isHealthy = systemStore.isSystemHealthy
  return {
    healthy: isHealthy,
    type: isHealthy ? 'success' : 'danger',
    text: isHealthy ? '系统运行正常' : '系统异常',
  }
})

// Methods
const toggleSidebar = () => {
  isCollapsed.value = !isCollapsed.value
  localStorage.setItem('sidebar-collapsed', isCollapsed.value.toString())
}

const toggleTheme = () => {
  systemStore.toggleDarkMode()
}

const handleUserCommand = async (command: string) => {
  switch (command) {
    case 'profile':
      // Open profile modal or navigate to profile page
      ElMessage.info('个人信息功能开发中')
      break
    case 'settings':
      router.push('/system/config')
      break
    case 'logout':
      try {
        await authStore.logout()
        router.push('/login')
      } catch (error) {
        console.error('Logout failed:', error)
      }
      break
  }
}

const handleAlertCommand = (command: any) => {
  if (command.type === 'view') {
    router.push(`/alerts?id=${command.id}`)
  } else if (command === 'view-all') {
    router.push('/alerts')
  }
}

const markAllAlertsRead = () => {
  // Mark all alerts as read
  recentAlerts.value.forEach(alert => {
    alert.acknowledged = true
  })
  ElMessage.success('已标记所有告警为已读')
}

// Watch for WebSocket messages
watch(lastMessage, (message) => {
  if (message) {
    try {
      const data = JSON.parse(message)
      if (data.type === 'alert') {
        recentAlerts.value.unshift(data.data)
        // Keep only the latest 10 alerts
        if (recentAlerts.value.length > 10) {
          recentAlerts.value = recentAlerts.value.slice(0, 10)
        }
        
        // Show notification for critical alerts
        if (data.data.level === 'critical') {
          ElNotification({
            type: 'error',
            title: '严重告警',
            message: data.data.message,
            duration: 0, // Don't auto close
          })
        }
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error)
    }
  }
})

// Initialize component
onMounted(async () => {
  // Restore sidebar state
  const savedCollapsed = localStorage.getItem('sidebar-collapsed')
  if (savedCollapsed !== null) {
    isCollapsed.value = savedCollapsed === 'true'
  }
  
  // Initialize stores
  await Promise.all([
    authStore.init(),
    systemStore.init(),
  ])
  
  // Start metrics polling
  systemStore.startMetricsPolling()
})

// Cleanup
onUnmounted(() => {
  systemStore.stopMetricsPolling()
})
</script>

<style scoped lang="scss">
.main-container {
  height: 100vh;
  
  .el-container {
    height: 100%;
  }
}

.sidebar {
  transition: width 0.3s;
  box-shadow: 2px 0 6px rgba(0, 21, 41, 0.35);
  
  .logo-container {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 16px;
    border-bottom: 1px solid var(--el-border-color);
    
    .logo {
      height: 32px;
      cursor: pointer;
    }
    
    .logo-text {
      margin-left: 12px;
      font-size: 18px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }
  }
  
  .el-menu {
    border-right: none;
    height: calc(100vh - 60px);
    overflow-y: auto;
  }
}

.main-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  
  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
    
    .status-badge {
      margin-right: 8px;
    }
    
    .user-avatar {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 4px 8px;
      border-radius: 6px;
      cursor: pointer;
      transition: background-color 0.3s;
      
      &:hover {
        background-color: var(--el-fill-color-light);
      }
      
      .user-name {
        font-size: 14px;
        color: var(--el-text-color-primary);
      }
    }
  }
}

.main-content {
  background-color: var(--el-bg-color-page);
  overflow: auto;
  
  .content-wrapper {
    padding: 20px;
    min-height: calc(100vh - 100px);
  }
}

.alerts-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  font-weight: 600;
  border-bottom: 1px solid var(--el-border-color);
}

.alert-item {
  .alert-content {
    .alert-message {
      font-size: 13px;
      margin-bottom: 4px;
    }
    
    .alert-time {
      font-size: 12px;
      color: var(--el-text-color-secondary);
    }
  }
  
  &.alert-critical {
    border-left: 3px solid var(--el-color-danger);
  }
  
  &.alert-error {
    border-left: 3px solid var(--el-color-danger);
  }
  
  &.alert-warning {
    border-left: 3px solid var(--el-color-warning);
  }
  
  &.alert-info {
    border-left: 3px solid var(--el-color-info);
  }
}

// Transitions
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.3s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-30px);
}

// Responsive design - temporarily disabled for desktop debugging
/* @media (max-width: 768px) {
  .sidebar {
    position: fixed;
    z-index: 1000;
    height: 100vh;
  }
  
  .main-header {
    .header-left {
      .el-breadcrumb {
        display: none;
      }
    }
    
    .header-right {
      .user-name {
        display: none;
      }
    }
  }
  
  .content-wrapper {
    padding: 15px;
  }
} */
</style>