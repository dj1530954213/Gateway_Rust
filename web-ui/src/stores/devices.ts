/**
 * 设备管理 Pinia Store
 * 
 * 管理设备列表、CRUD操作、搜索过滤等状态
 */

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { devicesApi } from '@/api'
import type { 
  DeviceVO, 
  DeviceCreateReq, 
  DevicePatchReq, 
  DeviceQuery, 
  ProtocolKind,
  PaginatedResponse 
} from '@/api/devices'

export interface DeviceState {
  // 设备列表数据
  devices: DeviceVO[]
  total: number
  
  // 分页信息
  currentPage: number
  pageSize: number
  
  // 加载状态
  loading: boolean
  
  // 查询条件
  query: DeviceQuery
  
  // 选中的设备
  selectedDevices: DeviceVO[]
  
  // 当前编辑的设备
  currentDevice: DeviceVO | null
}

export const useDevicesStore = defineStore('devices', () => {
  // ===== 状态定义 =====
  const state = ref<DeviceState>({
    devices: [],
    total: 0,
    currentPage: 1,
    pageSize: 20,
    loading: false,
    query: {
      page: 1,
      size: 20,
    },
    selectedDevices: [],
    currentDevice: null,
  })

  // ===== 计算属性 =====
  const totalPages = computed(() => {
    return Math.ceil(state.value.total / state.value.pageSize)
  })

  const hasDevices = computed(() => {
    return state.value.devices.length > 0
  })

  const isLoading = computed(() => {
    return state.value.loading
  })

  const enabledDevices = computed(() => {
    return state.value.devices.filter(device => device.enabled)
  })

  const disabledDevices = computed(() => {
    return state.value.devices.filter(device => !device.enabled)
  })

  const devicesByProtocol = computed(() => {
    const grouped: Record<string, DeviceVO[]> = {}
    state.value.devices.forEach(device => {
      if (!grouped[device.protocol]) {
        grouped[device.protocol] = []
      }
      grouped[device.protocol].push(device)
    })
    return grouped
  })

  // ===== 操作方法 =====

  /**
   * 获取设备列表
   */
  async function fetchDevices(params?: Partial<DeviceQuery>) {
    try {
      state.value.loading = true
      
      const query = {
        ...state.value.query,
        ...params,
      }

      const response: PaginatedResponse<DeviceVO> = await devicesApi.list(query)
      
      state.value.devices = response.items
      state.value.total = response.total
      state.value.currentPage = response.page
      state.value.pageSize = response.size
      
      // 更新查询条件
      state.value.query = { ...query }
      
    } catch (error) {
      console.error('获取设备列表失败:', error)
      ElMessage.error('获取设备列表失败')
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 获取设备详情
   */
  async function fetchDevice(id: string): Promise<DeviceVO | null> {
    try {
      const device = await devicesApi.get(id)
      state.value.currentDevice = device
      return device
    } catch (error) {
      console.error('获取设备详情失败:', error)
      ElMessage.error('获取设备详情失败')
      return null
    }
  }

  /**
   * 创建设备
   */
  async function createDevice(data: DeviceCreateReq): Promise<DeviceVO | null> {
    try {
      state.value.loading = true
      
      const device = await devicesApi.create(data)
      
      // 添加到列表开头
      state.value.devices.unshift(device)
      state.value.total += 1
      
      ElMessage.success('设备创建成功')
      return device
      
    } catch (error) {
      console.error('创建设备失败:', error)
      ElMessage.error('创建设备失败')
      return null
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 更新设备
   */
  async function updateDevice(id: string, data: DevicePatchReq): Promise<DeviceVO | null> {
    try {
      state.value.loading = true
      
      const updatedDevice = await devicesApi.update(id, data)
      
      // 更新列表中的设备
      const index = state.value.devices.findIndex(d => d.id === id)
      if (index !== -1) {
        state.value.devices[index] = updatedDevice
      }
      
      // 更新当前设备
      if (state.value.currentDevice?.id === id) {
        state.value.currentDevice = updatedDevice
      }
      
      ElMessage.success('设备更新成功')
      return updatedDevice
      
    } catch (error) {
      console.error('更新设备失败:', error)
      ElMessage.error('更新设备失败')
      return null
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 删除设备
   */
  async function deleteDevice(id: string): Promise<boolean> {
    try {
      await ElMessageBox.confirm(
        '确定要删除这个设备吗？删除后无法恢复。',
        '确认删除',
        {
          type: 'warning',
          confirmButtonText: '删除',
          cancelButtonText: '取消',
          confirmButtonClass: 'el-button--danger',
        }
      )

      state.value.loading = true
      
      await devicesApi.delete(id)
      
      // 从列表中移除
      state.value.devices = state.value.devices.filter(d => d.id !== id)
      state.value.total -= 1
      
      // 清除当前设备
      if (state.value.currentDevice?.id === id) {
        state.value.currentDevice = null
      }
      
      // 从选中列表中移除
      state.value.selectedDevices = state.value.selectedDevices.filter(d => d.id !== id)
      
      ElMessage.success('设备删除成功')
      return true
      
    } catch (error) {
      if (error !== 'cancel') {
        console.error('删除设备失败:', error)
        ElMessage.error('删除设备失败')
      }
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 批量删除设备
   */
  async function batchDeleteDevices(ids: string[]): Promise<boolean> {
    try {
      await ElMessageBox.confirm(
        `确定要删除选中的 ${ids.length} 个设备吗？删除后无法恢复。`,
        '确认批量删除',
        {
          type: 'warning',
          confirmButtonText: '删除',
          cancelButtonText: '取消',
          confirmButtonClass: 'el-button--danger',
        }
      )

      state.value.loading = true
      
      // 并行删除所有设备
      const results = await Promise.allSettled(
        ids.map(id => devicesApi.delete(id))
      )
      
      // 统计成功和失败的数量
      const successCount = results.filter(r => r.status === 'fulfilled').length
      const failedCount = results.filter(r => r.status === 'rejected').length
      
      // 从列表中移除成功删除的设备
      const successIds = ids.slice(0, successCount)
      state.value.devices = state.value.devices.filter(d => !successIds.includes(d.id))
      state.value.total -= successCount
      
      // 清空选中列表
      state.value.selectedDevices = []
      
      if (failedCount === 0) {
        ElMessage.success(`成功删除 ${successCount} 个设备`)
      } else {
        ElMessage.warning(`删除完成：成功 ${successCount} 个，失败 ${failedCount} 个`)
      }
      
      return failedCount === 0
      
    } catch (error) {
      if (error !== 'cancel') {
        console.error('批量删除设备失败:', error)
        ElMessage.error('批量删除设备失败')
      }
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 启用/禁用设备
   */
  async function toggleDevice(id: string, enabled: boolean): Promise<boolean> {
    try {
      const device = await devicesApi.update(id, { enabled })
      
      // 更新列表中的设备
      const index = state.value.devices.findIndex(d => d.id === id)
      if (index !== -1) {
        state.value.devices[index] = device
      }
      
      ElMessage.success(enabled ? '设备已启用' : '设备已禁用')
      return true
      
    } catch (error) {
      console.error('切换设备状态失败:', error)
      ElMessage.error('操作失败')
      return false
    }
  }

  /**
   * 测试设备连接
   */
  async function testDeviceConnection(data: DeviceCreateReq): Promise<boolean> {
    try {
      state.value.loading = true
      
      const result = await devicesApi.testConnection(data)
      
      if (result.success) {
        ElMessage.success('连接测试成功')
        return true
      } else {
        ElMessage.error(result.message || '连接测试失败')
        return false
      }
      
    } catch (error) {
      console.error('测试连接失败:', error)
      ElMessage.error('连接测试失败')
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 搜索设备
   */
  async function searchDevices(keyword: string) {
    await fetchDevices({
      ...state.value.query,
      name_contains: keyword || undefined,
      page: 1,
    })
  }

  /**
   * 按协议筛选
   */
  async function filterByProtocol(protocol: ProtocolKind | undefined) {
    await fetchDevices({
      ...state.value.query,
      protocol,
      page: 1,
    })
  }

  /**
   * 按状态筛选
   */
  async function filterByStatus(enabled: boolean | undefined) {
    await fetchDevices({
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
      size: state.value.pageSize,
    }
    await fetchDevices()
  }

  /**
   * 分页跳转
   */
  async function changePage(page: number) {
    await fetchDevices({
      ...state.value.query,
      page,
    })
  }

  /**
   * 更改分页大小
   */
  async function changePageSize(size: number) {
    await fetchDevices({
      ...state.value.query,
      size,
      page: 1,
    })
  }

  /**
   * 选择设备
   */
  function selectDevices(devices: DeviceVO[]) {
    state.value.selectedDevices = devices
  }

  /**
   * 切换设备选择
   */
  function toggleSelectDevice(device: DeviceVO) {
    const index = state.value.selectedDevices.findIndex(d => d.id === device.id)
    if (index !== -1) {
      state.value.selectedDevices.splice(index, 1)
    } else {
      state.value.selectedDevices.push(device)
    }
  }

  /**
   * 全选/取消全选
   */
  function toggleSelectAll() {
    if (state.value.selectedDevices.length === state.value.devices.length) {
      state.value.selectedDevices = []
    } else {
      state.value.selectedDevices = [...state.value.devices]
    }
  }

  /**
   * 清空选择
   */
  function clearSelection() {
    state.value.selectedDevices = []
  }

  /**
   * 设置当前设备
   */
  function setCurrentDevice(device: DeviceVO | null) {
    state.value.currentDevice = device
  }

  /**
   * 刷新列表
   */
  async function refresh() {
    await fetchDevices(state.value.query)
  }

  /**
   * 重置状态
   */
  function reset() {
    state.value.devices = []
    state.value.total = 0
    state.value.currentPage = 1
    state.value.pageSize = 20
    state.value.loading = false
    state.value.query = { page: 1, size: 20 }
    state.value.selectedDevices = []
    state.value.currentDevice = null
  }

  // ===== 返回 Store API =====
  return {
    // 状态
    state: readonly(state),
    
    // 计算属性
    totalPages,
    hasDevices,
    isLoading,
    enabledDevices,
    disabledDevices,
    devicesByProtocol,
    
    // 方法
    fetchDevices,
    fetchDevice,
    createDevice,
    updateDevice,
    deleteDevice,
    batchDeleteDevices,
    toggleDevice,
    testDeviceConnection,
    searchDevices,
    filterByProtocol,
    filterByStatus,
    resetSearch,
    changePage,
    changePageSize,
    selectDevices,
    toggleSelectDevice,
    toggleSelectAll,
    clearSelection,
    setCurrentDevice,
    refresh,
    reset,
  }
})

// 类型导出
export type DevicesStore = ReturnType<typeof useDevicesStore>