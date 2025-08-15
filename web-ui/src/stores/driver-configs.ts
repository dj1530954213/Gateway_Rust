/**
 * 驱动配置管理 Pinia Store
 * 专门用于管理数据库中的驱动配置数据
 */

import { ElMessage, ElMessageBox } from 'element-plus'
import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'

import { driverConfigsApi } from '@/api/drivers'
import type {
  DriverConfigVO,
  DriverConfigQuery,
  DriverConfigListResponse,
  DriverConfigCreateReq,
  DriverConfigUpdateReq,
} from '@/api/drivers'

export interface DriverConfigState {
  // 驱动配置列表
  configs: DriverConfigVO[]
  total: number

  // 分页信息
  currentPage: number
  pageSize: number

  // 加载状态
  loading: boolean
  creating: boolean
  updating: boolean

  // 查询条件
  query: DriverConfigQuery

  // 当前配置
  currentConfig: DriverConfigVO | null
}

export const useDriverConfigsStore = defineStore('driver-configs', () => {
  // ===== 状态定义 =====
  const state = ref<DriverConfigState>({
    configs: [],
    total: 0,
    currentPage: 1,
    pageSize: 20,
    loading: false,
    creating: false,
    updating: false,
    query: {
      page: 1,
      page_size: 20,
    },
    currentConfig: null,
  })

  // ===== 计算属性 =====
  const totalPages = computed(() => {
    return Math.ceil(state.value.total / state.value.pageSize)
  })

  const hasConfigs = computed(() => {
    return state.value.configs.length > 0
  })

  const isLoading = computed(() => {
    return state.value.loading
  })

  const enabledConfigs = computed(() => {
    return state.value.configs.filter(config => config.enabled)
  })

  const disabledConfigs = computed(() => {
    return state.value.configs.filter(config => !config.enabled)
  })

  const configsByProtocol = computed(() => {
    const grouped: Record<string, DriverConfigVO[]> = {}
    state.value.configs.forEach(config => {
      const protocol = config.protocol || 'Unknown'
      if (!grouped[protocol]) {
        grouped[protocol] = []
      }
      grouped[protocol].push(config)
    })
    return grouped
  })

  // ===== 操作方法 =====

  /**
   * 获取驱动配置列表
   */
  async function fetchConfigs(params?: Partial<DriverConfigQuery>) {
    try {
      state.value.loading = true

      const query = {
        ...state.value.query,
        ...params,
      }

      const response: DriverConfigListResponse =
        await driverConfigsApi.list(query)

      state.value.configs = response.driver_configs || []
      state.value.total = response.total || 0
      state.value.currentPage = response.page || 1
      state.value.pageSize = response.page_size || 20

      // 更新查询条件
      state.value.query = { ...query }
    } catch (error) {
      console.error('获取驱动配置失败:', error)
      ElMessage.error('获取驱动配置失败')
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 获取驱动配置详情
   */
  async function fetchConfig(id: string): Promise<DriverConfigVO | null> {
    try {
      const response = await driverConfigsApi.get(id)
      state.value.currentConfig = response.driver_config
      return response.driver_config
    } catch (error) {
      console.error('获取驱动配置详情失败:', error)
      ElMessage.error('获取驱动配置详情失败')
      return null
    }
  }

  /**
   * 创建驱动配置
   */
  async function createConfig(config: DriverConfigCreateReq): Promise<boolean> {
    try {
      state.value.creating = true

      const response = await driverConfigsApi.create(config)

      if (response.driver_config) {
        ElMessage.success('驱动配置创建成功')
        // 刷新列表
        await fetchConfigs(state.value.query)
        return true
      } else {
        ElMessage.error('驱动配置创建失败')
        return false
      }
    } catch (error) {
      console.error('创建驱动配置失败:', error)
      ElMessage.error('创建驱动配置失败')
      return false
    } finally {
      state.value.creating = false
    }
  }

  /**
   * 更新驱动配置
   */
  async function updateConfig(
    id: string,
    update: DriverConfigUpdateReq
  ): Promise<boolean> {
    try {
      state.value.updating = true

      const response = await driverConfigsApi.update(id, update)

      if (response.driver_config) {
        // 更新列表中的配置
        const index = state.value.configs.findIndex(c => c.id === id)
        if (index !== -1) {
          state.value.configs[index] = response.driver_config
        }

        // 更新当前配置
        if (state.value.currentConfig?.id === id) {
          state.value.currentConfig = response.driver_config
        }

        ElMessage.success('驱动配置更新成功')
        return true
      } else {
        ElMessage.error('驱动配置更新失败')
        return false
      }
    } catch (error) {
      console.error('更新驱动配置失败:', error)
      ElMessage.error('更新驱动配置失败')
      return false
    } finally {
      state.value.updating = false
    }
  }

  /**
   * 删除驱动配置
   */
  async function deleteConfig(id: string): Promise<boolean> {
    try {
      const config = state.value.configs.find(c => c.id === id)
      if (!config) return false

      await ElMessageBox.confirm(
        `确定要删除驱动配置 "${config.name}" 吗？删除后无法恢复。`,
        '确认删除',
        {
          type: 'warning',
          confirmButtonText: '删除',
          cancelButtonText: '取消',
          confirmButtonClass: 'el-button--danger',
        }
      )

      state.value.loading = true

      await driverConfigsApi.delete(id)

      // 从列表中移除
      state.value.configs = state.value.configs.filter(c => c.id !== id)
      state.value.total -= 1

      // 清除当前配置
      if (state.value.currentConfig?.id === id) {
        state.value.currentConfig = null
      }

      ElMessage.success('驱动配置删除成功')
      return true
    } catch (error) {
      if (error !== 'cancel') {
        console.error('删除驱动配置失败:', error)
        ElMessage.error('删除驱动配置失败')
      }
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 启用/禁用配置
   */
  async function toggleConfigStatus(
    id: string,
    enabled: boolean
  ): Promise<boolean> {
    return await updateConfig(id, { enabled })
  }

  /**
   * 搜索配置
   */
  async function searchConfigs(keyword: string) {
    await fetchConfigs({
      ...state.value.query,
      name_contains: keyword || undefined,
      page: 1,
    })
  }

  /**
   * 按协议筛选
   */
  async function filterByProtocol(protocol: string | undefined) {
    await fetchConfigs({
      ...state.value.query,
      protocol,
      page: 1,
    })
  }

  /**
   * 按状态筛选
   */
  async function filterByEnabled(enabled: boolean | undefined) {
    await fetchConfigs({
      ...state.value.query,
      enabled,
      page: 1,
    })
  }

  /**
   * 重置搜索条件
   */
  async function resetSearch() {
    state.value.query = {
      page: 1,
      page_size: state.value.pageSize,
    }
    await fetchConfigs()
  }

  /**
   * 分页跳转
   */
  async function changePage(page: number) {
    await fetchConfigs({
      ...state.value.query,
      page,
    })
  }

  /**
   * 更改分页大小
   */
  async function changePageSize(size: number) {
    await fetchConfigs({
      ...state.value.query,
      page_size: size,
      page: 1,
    })
  }

  /**
   * 设置当前配置
   */
  function setCurrentConfig(config: DriverConfigVO | null) {
    state.value.currentConfig = config
  }

  /**
   * 刷新列表
   */
  async function refresh() {
    await fetchConfigs(state.value.query)
  }

  /**
   * 重置状态
   */
  function reset() {
    state.value.configs = []
    state.value.total = 0
    state.value.currentPage = 1
    state.value.pageSize = 20
    state.value.loading = false
    state.value.creating = false
    state.value.updating = false
    state.value.query = { page: 1, page_size: 20 }
    state.value.currentConfig = null
  }

  // ===== 返回 Store API =====
  return {
    // 状态
    state: readonly(state),

    // 计算属性
    totalPages,
    hasConfigs,
    isLoading,
    enabledConfigs,
    disabledConfigs,
    configsByProtocol,

    // 方法
    fetchConfigs,
    fetchConfig,
    createConfig,
    updateConfig,
    deleteConfig,
    toggleConfigStatus,
    searchConfigs,
    filterByProtocol,
    filterByEnabled,
    resetSearch,
    changePage,
    changePageSize,
    setCurrentConfig,
    refresh,
    reset,
  }
})

// 类型导出
export type DriverConfigsStore = ReturnType<typeof useDriverConfigsStore>
