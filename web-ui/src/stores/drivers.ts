/**
 * 驱动管理 Pinia Store
 * 
 * 管理驱动列表、上传、热重载、注册状态等操作
 */

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { driversApi } from '@/api'
import type { 
  DriverVO, 
  DriverQuery, 
  DriverUploadResponse,
  DriverStatusVO,
  PaginatedResponse 
} from '@/api/drivers'

export interface DriverState {
  // 驱动列表数据
  drivers: DriverVO[]
  total: number
  
  // 分页信息
  currentPage: number
  pageSize: number
  
  // 加载状态
  loading: boolean
  uploading: boolean
  reloading: boolean
  
  // 查询条件
  query: DriverQuery
  
  // 选中的驱动
  selectedDrivers: DriverVO[]
  
  // 当前查看的驱动
  currentDriver: DriverVO | null
  
  // 上传进度
  uploadProgress: number
  
  // 状态统计
  statusStats: {
    loaded: number
    failed: number
    unloaded: number
  }
}

export const useDriversStore = defineStore('drivers', () => {
  // ===== 状态定义 =====
  const state = ref<DriverState>({
    drivers: [],
    total: 0,
    currentPage: 1,
    pageSize: 20,
    loading: false,
    uploading: false,
    reloading: false,
    query: {
      page: 1,
      size: 20,
    },
    selectedDrivers: [],
    currentDriver: null,
    uploadProgress: 0,
    statusStats: {
      loaded: 0,
      failed: 0,
      unloaded: 0,
    },
  })

  // ===== 计算属性 =====
  const totalPages = computed(() => {
    return Math.ceil(state.value.total / state.value.pageSize)
  })

  const hasDrivers = computed(() => {
    return state.value.drivers.length > 0
  })

  const isLoading = computed(() => {
    return state.value.loading
  })

  const isUploading = computed(() => {
    return state.value.uploading
  })

  const isReloading = computed(() => {
    return state.value.reloading
  })

  const loadedDrivers = computed(() => {
    return state.value.drivers.filter(driver => driver.status === 'Loaded')
  })

  const failedDrivers = computed(() => {
    return state.value.drivers.filter(driver => driver.status === 'Failed')
  })

  const unloadedDrivers = computed(() => {
    return state.value.drivers.filter(driver => driver.status === 'Unloaded')
  })

  const driversByProtocol = computed(() => {
    const grouped: Record<string, DriverVO[]> = {}
    state.value.drivers.forEach(driver => {
      const protocol = driver.info?.protocol || 'Unknown'
      if (!grouped[protocol]) {
        grouped[protocol] = []
      }
      grouped[protocol].push(driver)
    })
    return grouped
  })

  const canReload = computed(() => {
    return state.value.drivers.some(driver => driver.status === 'Failed' || driver.status === 'Unloaded')
  })

  // ===== 操作方法 =====

  /**
   * 获取驱动列表
   */
  async function fetchDrivers(params?: Partial<DriverQuery>) {
    try {
      state.value.loading = true
      
      const query = {
        ...state.value.query,
        ...params,
      }

      const response: any = await driversApi.list(query)
      
      // 处理后端返回的数据格式 {drivers: [], total: 0, page: 1, page_size: 20}
      state.value.drivers = response.drivers || response.items || []
      state.value.total = response.total || 0
      state.value.currentPage = response.page || 1
      state.value.pageSize = response.page_size || response.size || 20
      
      // 更新查询条件
      state.value.query = { ...query }
      
      // 更新状态统计
      updateStatusStats()
      
    } catch (error) {
      console.error('获取驱动列表失败:', error)
      ElMessage.error('获取驱动列表失败')
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 获取驱动详情
   */
  async function fetchDriver(id: string): Promise<DriverVO | null> {
    try {
      const driver = await driversApi.get(id)
      state.value.currentDriver = driver
      return driver
    } catch (error) {
      console.error('获取驱动详情失败:', error)
      ElMessage.error('获取驱动详情失败')
      return null
    }
  }

  /**
   * 上传驱动文件
   */
  async function uploadDriver(file: File): Promise<DriverUploadResponse | null> {
    try {
      state.value.uploading = true
      state.value.uploadProgress = 0
      
      const response = await driversApi.upload(file, (progress) => {
        state.value.uploadProgress = progress
      })
      
      if (response.success) {
        ElMessage.success('驱动上传成功')
        // 刷新驱动列表
        await fetchDrivers(state.value.query)
        return response
      } else {
        ElMessage.error(response.message || '驱动上传失败')
        return null
      }
      
    } catch (error) {
      console.error('上传驱动失败:', error)
      ElMessage.error('上传驱动失败')
      return null
    } finally {
      state.value.uploading = false
      state.value.uploadProgress = 0
    }
  }

  /**
   * 删除驱动文件
   */
  async function deleteDriver(id: string): Promise<boolean> {
    try {
      const driver = state.value.drivers.find(d => d.id === id)
      if (!driver) return false

      await ElMessageBox.confirm(
        `确定要删除驱动 "${driver.filename}" 吗？删除后无法恢复。`,
        '确认删除',
        {
          type: 'warning',
          confirmButtonText: '删除',
          cancelButtonText: '取消',
          confirmButtonClass: 'el-button--danger',
        }
      )

      state.value.loading = true
      
      await driversApi.delete(id)
      
      // 从列表中移除
      state.value.drivers = state.value.drivers.filter(d => d.id !== id)
      state.value.total -= 1
      
      // 清除当前驱动
      if (state.value.currentDriver?.id === id) {
        state.value.currentDriver = null
      }
      
      // 从选中列表中移除
      state.value.selectedDrivers = state.value.selectedDrivers.filter(d => d.id !== id)
      
      // 更新状态统计
      updateStatusStats()
      
      ElMessage.success('驱动删除成功')
      return true
      
    } catch (error) {
      if (error !== 'cancel') {
        console.error('删除驱动失败:', error)
        ElMessage.error('删除驱动失败')
      }
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 重新加载单个驱动
   */
  async function reloadDriver(id: string): Promise<boolean> {
    try {
      state.value.loading = true
      
      const result = await driversApi.reload(id)
      
      if (result.success) {
        // 更新驱动状态
        const index = state.value.drivers.findIndex(d => d.id === id)
        if (index !== -1) {
          state.value.drivers[index] = result.driver
        }
        
        // 更新当前驱动
        if (state.value.currentDriver?.id === id) {
          state.value.currentDriver = result.driver
        }
        
        // 更新状态统计
        updateStatusStats()
        
        ElMessage.success('驱动重载成功')
        return true
      } else {
        ElMessage.error(result.message || '驱动重载失败')
        return false
      }
      
    } catch (error) {
      console.error('重载驱动失败:', error)
      ElMessage.error('重载驱动失败')
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 重新加载所有驱动
   */
  async function reloadAllDrivers(): Promise<boolean> {
    try {
      await ElMessageBox.confirm(
        '确定要重新加载所有驱动吗？这可能会影响正在运行的设备连接。',
        '确认重新加载',
        {
          type: 'warning',
          confirmButtonText: '重新加载',
          cancelButtonText: '取消',
        }
      )

      state.value.reloading = true
      
      const result = await driversApi.reloadAll()
      
      if (result.success) {
        ElMessage.success(`重载完成：成功 ${result.success_count} 个，失败 ${result.failed_count} 个`)
        // 刷新驱动列表
        await fetchDrivers(state.value.query)
        return true
      } else {
        ElMessage.error(result.message || '驱动重载失败')
        return false
      }
      
    } catch (error) {
      if (error !== 'cancel') {
        console.error('重载所有驱动失败:', error)
        ElMessage.error('重载所有驱动失败')
      }
      return false
    } finally {
      state.value.reloading = false
    }
  }

  /**
   * 获取驱动状态
   */
  async function fetchDriverStatus(): Promise<DriverStatusVO | null> {
    try {
      const status = await driversApi.getStatus()
      
      // 更新状态统计
      state.value.statusStats = {
        loaded: status.loaded_count,
        failed: status.failed_count,
        unloaded: status.unloaded_count,
      }
      
      return status
    } catch (error) {
      console.error('获取驱动状态失败:', error)
      return null
    }
  }

  /**
   * 搜索驱动
   */
  async function searchDrivers(keyword: string) {
    await fetchDrivers({
      ...state.value.query,
      name_contains: keyword || undefined,
      page: 1,
    })
  }

  /**
   * 按状态筛选
   */
  async function filterByStatus(status: 'Loaded' | 'Failed' | 'Unloaded' | undefined) {
    await fetchDrivers({
      ...state.value.query,
      status,
      page: 1,
    })
  }

  /**
   * 按协议筛选
   */
  async function filterByProtocol(protocol: string | undefined) {
    await fetchDrivers({
      ...state.value.query,
      protocol,
      page: 1,
    })
  }

  /**
   * 重置搜索条件
   */
  async function resetSearch() {
    state.value.query = {
      page: 1,
      size: state.value.pageSize,
    }
    await fetchDrivers()
  }

  /**
   * 分页跳转
   */
  async function changePage(page: number) {
    await fetchDrivers({
      ...state.value.query,
      page,
    })
  }

  /**
   * 更改分页大小
   */
  async function changePageSize(size: number) {
    await fetchDrivers({
      ...state.value.query,
      size,
      page: 1,
    })
  }

  /**
   * 选择驱动
   */
  function selectDrivers(drivers: DriverVO[]) {
    state.value.selectedDrivers = drivers
  }

  /**
   * 切换驱动选择
   */
  function toggleSelectDriver(driver: DriverVO) {
    const index = state.value.selectedDrivers.findIndex(d => d.id === driver.id)
    if (index !== -1) {
      state.value.selectedDrivers.splice(index, 1)
    } else {
      state.value.selectedDrivers.push(driver)
    }
  }

  /**
   * 全选/取消全选
   */
  function toggleSelectAll() {
    if (state.value.selectedDrivers.length === state.value.drivers.length) {
      state.value.selectedDrivers = []
    } else {
      state.value.selectedDrivers = [...state.value.drivers]
    }
  }

  /**
   * 清空选择
   */
  function clearSelection() {
    state.value.selectedDrivers = []
  }

  /**
   * 设置当前驱动
   */
  function setCurrentDriver(driver: DriverVO | null) {
    state.value.currentDriver = driver
  }

  /**
   * 更新状态统计
   */
  function updateStatusStats() {
    const drivers = state.value.drivers || []
    state.value.statusStats = {
      loaded: drivers.filter(d => d.status === 'Loaded').length,
      failed: drivers.filter(d => d.status === 'Failed').length,
      unloaded: drivers.filter(d => d.status === 'Unloaded').length,
    }
  }

  /**
   * 刷新列表
   */
  async function refresh() {
    await fetchDrivers(state.value.query)
  }

  /**
   * 重置状态
   */
  function reset() {
    state.value.drivers = []
    state.value.total = 0
    state.value.currentPage = 1
    state.value.pageSize = 20
    state.value.loading = false
    state.value.uploading = false
    state.value.reloading = false
    state.value.query = { page: 1, size: 20 }
    state.value.selectedDrivers = []
    state.value.currentDriver = null
    state.value.uploadProgress = 0
    state.value.statusStats = {
      loaded: 0,
      failed: 0,
      unloaded: 0,
    }
  }

  // ===== 返回 Store API =====
  return {
    // 状态
    state: readonly(state),
    
    // 计算属性
    totalPages,
    hasDrivers,
    isLoading,
    isUploading,
    isReloading,
    loadedDrivers,
    failedDrivers,
    unloadedDrivers,
    driversByProtocol,
    canReload,
    
    // 方法
    fetchDrivers,
    fetchDriver,
    uploadDriver,
    deleteDriver,
    reloadDriver,
    reloadAllDrivers,
    fetchDriverStatus,
    searchDrivers,
    filterByStatus,
    filterByProtocol,
    resetSearch,
    changePage,
    changePageSize,
    selectDrivers,
    toggleSelectDriver,
    toggleSelectAll,
    clearSelection,
    setCurrentDriver,
    refresh,
    reset,
  }
})

// 类型导出
export type DriversStore = ReturnType<typeof useDriversStore>