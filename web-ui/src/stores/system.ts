import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { ElMessage } from 'element-plus'
import type { 
  SystemInfo, 
  SystemMetrics, 
  HealthStatus, 
  ThemeConfig, 
  ThemeMode,
  GatewayConfig 
} from '@/types'
import { systemApi, BackupInfo } from '@/services/api'

export const useSystemStore = defineStore('system', () => {
  // State
  const systemInfo = ref<SystemInfo | null>(null)
  const systemMetrics = ref<SystemMetrics | null>(null)
  const healthStatus = ref<HealthStatus | null>(null)
  const gatewayConfig = ref<GatewayConfig | null>(null)
  const theme = ref<ThemeConfig>({
    mode: (localStorage.getItem('theme-mode') as ThemeMode) || 'light',
    primaryColor: localStorage.getItem('theme-primary') || '#409eff',
    accentColor: localStorage.getItem('theme-accent') || '#67c23a',
    fontSize: parseInt(localStorage.getItem('theme-font-size') || '14'),
  })
  const loading = ref(false)
  const metricsLoading = ref(false)

  // Getters
  const isDarkMode = computed(() => {
    if (theme.value.mode === 'auto') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return theme.value.mode === 'dark'
  })

  const isSystemHealthy = computed(() => {
    if (!healthStatus.value) return false
    return ['excellent', 'good'].includes(healthStatus.value.overall)
  })

  const criticalComponents = computed(() => {
    if (!healthStatus.value) return []
    return healthStatus.value.components.filter(
      comp => ['critical', 'failed'].includes(comp.status)
    )
  })

  // Actions
  const fetchSystemInfo = async (): Promise<void> => {
    try {
      // 检查是否启用Mock模式
      if (import.meta.env.VITE_ENABLE_MOCK === 'true') {
        systemInfo.value = {
          version: '1.0.0',
          uptime: 86400,
          memory_usage: 65.5,
          cpu_usage: 23.8,
          disk_usage: 45.2,
          connected_devices: 12,
          active_tags: 256,
          alert_count: 3
        }
        return
      }
      
      const info = await systemApi.getInfo()
      systemInfo.value = info
    } catch (error) {
      console.error('Failed to fetch system info:', error)
    }
  }

  const fetchSystemMetrics = async (): Promise<void> => {
    metricsLoading.value = true
    try {
      // 检查是否启用Mock模式
      if (import.meta.env.VITE_ENABLE_MOCK === 'true') {
        // 生成随机变化的Mock数据
        systemMetrics.value = {
          timestamp: new Date().toISOString(),
          cpu_usage: Math.random() * 30 + 20,
          memory_usage: Math.random() * 20 + 60,
          disk_usage: Math.random() * 10 + 40,
          network_io: {
            bytes_in: Math.floor(Math.random() * 10000),
            bytes_out: Math.floor(Math.random() * 5000)
          },
          active_connections: Math.floor(Math.random() * 20 + 10),
          request_rate: Math.random() * 100 + 50,
          error_rate: Math.random() * 2
        }
        metricsLoading.value = false
        return
      }
      
      const metrics = await systemApi.getMetrics()
      systemMetrics.value = metrics[0] || null // getMetrics returns array, take first item
    } catch (error) {
      console.error('Failed to fetch system metrics:', error)
      // 如果是404错误，停止轮询
      if ((error as any)?.response?.status === 404) {
        stopMetricsPolling()
        console.warn('API服务不可用，已停止指标轮询')
      }
    } finally {
      metricsLoading.value = false
    }
  }

  const fetchHealthStatus = async (): Promise<void> => {
    try {
      // 检查是否启用Mock模式
      if (import.meta.env.VITE_ENABLE_MOCK === 'true') {
        healthStatus.value = {
          overall: 'good',
          components: [
            {
              name: 'database',
              status: 'healthy',
              message: 'PostgreSQL连接正常',
              last_check: new Date().toISOString()
            },
            {
              name: 'influxdb', 
              status: 'healthy',
              message: 'InfluxDB运行正常',
              last_check: new Date().toISOString()
            },
            {
              name: 'drivers',
              status: 'warning',
              message: '部分驱动连接异常',
              last_check: new Date().toISOString()
            },
            {
              name: 'websocket',
              status: 'healthy',
              message: 'WebSocket服务正常',
              last_check: new Date().toISOString()
            }
          ],
          uptime: 86400,
          version: '1.0.0',
          timestamp: new Date().toISOString()
        }
        return
      }
      
      const health = await systemApi.getHealth()
      healthStatus.value = health
    } catch (error) {
      console.error('Failed to fetch health status:', error)
      // 如果是404错误，停止轮询
      if ((error as any)?.response?.status === 404) {
        stopMetricsPolling()
        console.warn('API服务不可用，已停止健康状态轮询')
      }
    }
  }

  const fetchGatewayConfig = async (): Promise<void> => {
    loading.value = true
    try {
      const config = await systemApi.getConfig()
      gatewayConfig.value = config
    } catch (error) {
      console.error('Failed to fetch gateway config:', error)
      ElMessage.error('获取系统配置失败')
    } finally {
      loading.value = false
    }
  }

  const updateGatewayConfig = async (config: Partial<GatewayConfig>): Promise<void> => {
    loading.value = true
    try {
      await systemApi.updateConfig(config)
      // Refetch config to get updated data
      await fetchGatewayConfig()
      ElMessage.success('系统配置更新成功')
    } catch (error: any) {
      console.error('Failed to update gateway config:', error)
      ElMessage.error(error.message || '系统配置更新失败')
      throw error
    } finally {
      loading.value = false
    }
  }

  const restartSystem = async (): Promise<void> => {
    try {
      await systemApi.restart()
      ElMessage.success('系统重启命令已发送')
    } catch (error: any) {
      console.error('Failed to restart system:', error)
      ElMessage.error(error.message || '系统重启失败')
      throw error
    }
  }

  const backupConfig = async (): Promise<BackupInfo> => {
    try {
      const backup = await systemApi.createBackup()
      ElMessage.success('配置备份已生成')
      return backup
    } catch (error: any) {
      console.error('Failed to backup config:', error)
      ElMessage.error('配置备份失败')
      throw error
    }
  }

  const restoreConfig = async (backupId: string): Promise<void> => {
    loading.value = true
    try {
      await systemApi.restoreBackup(backupId)
      ElMessage.success('配置恢复成功')
      // Refresh config after restore
      await fetchGatewayConfig()
    } catch (error: any) {
      console.error('Failed to restore config:', error)
      ElMessage.error(error.message || '配置恢复失败')
      throw error
    } finally {
      loading.value = false
    }
  }

  const setTheme = (newTheme: Partial<ThemeConfig>): void => {
    theme.value = { ...theme.value, ...newTheme }
    
    // Save to localStorage
    localStorage.setItem('theme-mode', theme.value.mode)
    localStorage.setItem('theme-primary', theme.value.primaryColor)
    localStorage.setItem('theme-accent', theme.value.accentColor)
    localStorage.setItem('theme-font-size', theme.value.fontSize.toString())
    
    // Apply theme to DOM
    applyThemeToDOM()
  }

  const toggleDarkMode = (): void => {
    const newMode = isDarkMode.value ? 'light' : 'dark'
    setTheme({ mode: newMode })
  }

  const applyThemeToDOM = (): void => {
    const root = document.documentElement
    
    // Apply dark mode class
    if (isDarkMode.value) {
      root.classList.add('dark')
    } else {
      root.classList.remove('dark')
    }
    
    // Apply custom CSS variables
    root.style.setProperty('--el-color-primary', theme.value.primaryColor)
    root.style.setProperty('--theme-accent-color', theme.value.accentColor)
    root.style.setProperty('--theme-font-size', `${theme.value.fontSize}px`)
  }

  const startMetricsPolling = (interval: number = 5000): void => {
    const timer = setInterval(async () => {
      await fetchSystemMetrics()
      await fetchHealthStatus()
    }, interval)
    
    // Store timer for cleanup
    ;(window as any).__metricsTimer = timer
  }

  const stopMetricsPolling = (): void => {
    if ((window as any).__metricsTimer) {
      clearInterval((window as any).__metricsTimer)
      delete (window as any).__metricsTimer
    }
  }

  // Initialize store
  const init = async (): Promise<void> => {
    // Apply saved theme
    applyThemeToDOM()
    
    // Watch for system theme changes if in auto mode
    if (theme.value.mode === 'auto') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      mediaQuery.addEventListener('change', applyThemeToDOM)
    }
    
    // Fetch initial system data
    await Promise.all([
      fetchSystemInfo(),
      fetchSystemMetrics(),
      fetchHealthStatus(),
    ])
  }

  return {
    // State
    systemInfo: readonly(systemInfo),
    systemMetrics: readonly(systemMetrics),
    healthStatus: readonly(healthStatus),
    gatewayConfig: readonly(gatewayConfig),
    theme: readonly(theme),
    loading: readonly(loading),
    metricsLoading: readonly(metricsLoading),
    
    // Getters
    isDarkMode,
    isSystemHealthy,
    criticalComponents,
    
    // Actions
    fetchSystemInfo,
    fetchSystemMetrics,
    fetchHealthStatus,
    fetchGatewayConfig,
    updateGatewayConfig,
    restartSystem,
    backupConfig,
    restoreConfig,
    setTheme,
    toggleDarkMode,
    startMetricsPolling,
    stopMetricsPolling,
    init,
  }
})