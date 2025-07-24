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
import { systemApi } from '@/services/api'

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
      const response = await systemApi.getSystemInfo()
      if (response.success && response.data) {
        systemInfo.value = response.data as SystemInfo
      }
    } catch (error) {
      console.error('Failed to fetch system info:', error)
    }
  }

  const fetchSystemMetrics = async (): Promise<void> => {
    metricsLoading.value = true
    try {
      const response = await systemApi.getSystemMetrics()
      if (response.success && response.data) {
        systemMetrics.value = response.data as SystemMetrics
      }
    } catch (error) {
      console.error('Failed to fetch system metrics:', error)
    } finally {
      metricsLoading.value = false
    }
  }

  const fetchHealthStatus = async (): Promise<void> => {
    try {
      const response = await systemApi.getHealthStatus()
      if (response.success && response.data) {
        healthStatus.value = response.data as HealthStatus
      }
    } catch (error) {
      console.error('Failed to fetch health status:', error)
    }
  }

  const fetchGatewayConfig = async (): Promise<void> => {
    loading.value = true
    try {
      const response = await systemApi.getGatewayConfig()
      if (response.success && response.data) {
        gatewayConfig.value = response.data as GatewayConfig
      }
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
      const response = await systemApi.updateGatewayConfig(config)
      if (response.success && response.data) {
        gatewayConfig.value = response.data as GatewayConfig
        ElMessage.success('系统配置更新成功')
      } else {
        throw new Error(response.message || '配置更新失败')
      }
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
      const response = await systemApi.restartSystem()
      if (response.success) {
        ElMessage.success('系统重启命令已发送')
      } else {
        throw new Error(response.message || '重启失败')
      }
    } catch (error: any) {
      console.error('Failed to restart system:', error)
      ElMessage.error(error.message || '系统重启失败')
      throw error
    }
  }

  const backupConfig = async (): Promise<Blob> => {
    try {
      const response = await systemApi.backupConfig()
      ElMessage.success('配置备份已生成')
      return response
    } catch (error: any) {
      console.error('Failed to backup config:', error)
      ElMessage.error('配置备份失败')
      throw error
    }
  }

  const restoreConfig = async (file: File): Promise<void> => {
    loading.value = true
    try {
      const response = await systemApi.restoreConfig(file)
      if (response.success) {
        ElMessage.success('配置恢复成功')
        // Refresh config after restore
        await fetchGatewayConfig()
      } else {
        throw new Error(response.message || '配置恢复失败')
      }
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