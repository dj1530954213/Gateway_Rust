/**
 * 设备管理 Pinia Store - 简化版本
 * 避免复杂继承，直接使用defineStore
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'

// 设备类型定义
export interface Device {
  id: string
  name: string
  protocol: string
  endpoint: string
  enabled: boolean
  location?: string
  description?: string
  created_at?: string
  updated_at?: string
}

export interface DeviceCreateReq {
  name: string
  protocol: string
  endpoint: string
  enabled?: boolean
  location?: string
  description?: string
}

export interface DeviceQuery {
  page?: number
  size?: number
  name_contains?: string
  protocol?: string
  enabled?: boolean
}

export const useDevicesStore = defineStore('devices', () => {
  // 状态
  const loading = ref(false)
  const error = ref<string | null>(null)
  const devices = ref<Device[]>([])
  const selectedDevices = ref<Device[]>([])
  const currentDevice = ref<Device | null>(null)
  
  // 分页信息
  const pagination = ref({
    page: 1,
    size: 20,
    total: 0,
    pages: 0
  })

  // 搜索条件
  const searchQuery = ref<DeviceQuery>({
    page: 1,
    size: 20
  })

  // 计算属性
  const enabledDevices = computed(() => 
    devices.value.filter(device => device.enabled)
  )

  const disabledDevices = computed(() => 
    devices.value.filter(device => !device.enabled)
  )

  const devicesByProtocol = computed(() => {
    const groups: Record<string, Device[]> = {}
    devices.value.forEach(device => {
      if (!groups[device.protocol]) {
        groups[device.protocol] = []
      }
      groups[device.protocol].push(device)
    })
    return groups
  })

  const hasDevices = computed(() => devices.value.length > 0)
  const isLoading = computed(() => loading.value)
  const hasError = computed(() => error.value !== null)

  const totalPages = computed(() => 
    Math.ceil(pagination.value.total / pagination.value.size)
  )

  const selectedIds = computed(() => 
    selectedDevices.value.map(device => device.id)
  )

  // 操作方法
  async function fetchDevices(query?: Partial<DeviceQuery>) {
    loading.value = true
    error.value = null
    
    try {
      const finalQuery = { ...searchQuery.value, ...query }
      const queryParams = new URLSearchParams()
      
      Object.entries(finalQuery).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== '') {
          queryParams.append(key, String(value))
        }
      })
      
      const response = await fetch(`http://localhost:50016/api/devices?${queryParams}`)
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      
      const data = await response.json()
      
      // 适配不同的响应格式
      if (data.success && data.data) {
        devices.value = Array.isArray(data.data) ? data.data : []
        pagination.value.total = data.total || devices.value.length
      } else if (Array.isArray(data)) {
        devices.value = data
        pagination.value.total = data.length
      } else if (data.data && Array.isArray(data.data)) {
        devices.value = data.data
        pagination.value.total = data.total || devices.value.length
      } else {
        devices.value = []
        pagination.value.total = 0
      }
      
      // 更新分页信息
      pagination.value.page = finalQuery.page || 1
      pagination.value.size = finalQuery.size || 20
      pagination.value.pages = totalPages.value
      
      searchQuery.value = finalQuery
      
      return devices.value
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '获取设备列表失败'
      error.value = errorMessage
      ElMessage.error(errorMessage)
      devices.value = []
      return []
    } finally {
      loading.value = false
    }
  }

  async function fetchDevice(id: string) {
    loading.value = true
    error.value = null
    
    try {
      const response = await fetch(`http://localhost:50016/api/devices/${id}`)
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      
      const data = await response.json()
      const device = data.success ? data.data : data
      
      currentDevice.value = device
      return device
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '获取设备详情失败'
      error.value = errorMessage
      ElMessage.error(errorMessage)
      return null
    } finally {
      loading.value = false
    }
  }

  async function createDevice(deviceData: DeviceCreateReq) {
    loading.value = true
    error.value = null
    
    try {
      const response = await fetch('http://localhost:50016/api/devices', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(deviceData)
      })
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      
      const data = await response.json()
      const newDevice = data.success ? data.data : data
      
      devices.value.unshift(newDevice)
      pagination.value.total += 1
      
      ElMessage.success('设备创建成功')
      return newDevice
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '创建设备失败'
      error.value = errorMessage
      ElMessage.error(errorMessage)
      return null
    } finally {
      loading.value = false
    }
  }

  async function updateDevice(id: string, deviceData: Partial<DeviceCreateReq>) {
    loading.value = true
    error.value = null
    
    try {
      const response = await fetch(`http://localhost:50016/api/devices/${id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(deviceData)
      })
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      
      const data = await response.json()
      const updatedDevice = data.success ? data.data : data
      
      // 更新列表中的设备
      const index = devices.value.findIndex(device => device.id === id)
      if (index !== -1) {
        devices.value[index] = updatedDevice
      }
      
      // 更新当前设备
      if (currentDevice.value && currentDevice.value.id === id) {
        currentDevice.value = updatedDevice
      }
      
      ElMessage.success('设备更新成功')
      return updatedDevice
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '更新设备失败'
      error.value = errorMessage
      ElMessage.error(errorMessage)
      return null
    } finally {
      loading.value = false
    }
  }

  async function deleteDevice(id: string) {
    loading.value = true
    error.value = null
    
    try {
      const response = await fetch(`http://localhost:50016/api/devices/${id}`, {
        method: 'DELETE'
      })
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      
      // 从列表中移除
      devices.value = devices.value.filter(device => device.id !== id)
      pagination.value.total -= 1
      
      // 清除当前设备
      if (currentDevice.value && currentDevice.value.id === id) {
        currentDevice.value = null
      }
      
      // 从选中列表中移除
      selectedDevices.value = selectedDevices.value.filter(device => device.id !== id)
      
      ElMessage.success('设备删除成功')
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '删除设备失败'
      error.value = errorMessage
      ElMessage.error(errorMessage)
      return false
    } finally {
      loading.value = false
    }
  }

  async function batchDeleteDevices(ids: string[]) {
    loading.value = true
    error.value = null
    
    try {
      const results = await Promise.allSettled(
        ids.map(id => fetch(`http://localhost:50016/api/devices/${id}`, {
          method: 'DELETE'
        }))
      )
      
      let successful = 0
      let failed = 0
      
      results.forEach((result, index) => {
        if (result.status === 'fulfilled' && result.value.ok) {
          const id = ids[index]
          devices.value = devices.value.filter(device => device.id !== id)
          successful++
        } else {
          failed++
        }
      })
      
      pagination.value.total -= successful
      selectedDevices.value = []
      
      ElMessage.success(`批量删除完成：成功 ${successful} 项，失败 ${failed} 项`)
      return { successful, failed }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '批量删除失败'
      error.value = errorMessage
      ElMessage.error(errorMessage)
      return { successful: 0, failed: ids.length }
    } finally {
      loading.value = false
    }
  }

  // 设备特定操作
  async function toggleDevice(id: string) {
    const device = devices.value.find(d => d.id === id)
    if (!device) return false
    
    return updateDevice(id, { enabled: !device.enabled })
  }

  async function testDeviceConnection(id: string) {
    loading.value = true
    error.value = null
    
    try {
      const response = await fetch(`http://localhost:50016/api/devices/${id}/test`, {
        method: 'POST'
      })
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      
      const data = await response.json()
      const result = data.success ? data.data : data
      
      ElMessage.success('设备连接测试成功')
      return result
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '设备连接测试失败'
      error.value = errorMessage
      ElMessage.error(errorMessage)
      return null
    } finally {
      loading.value = false
    }
  }

  // 搜索和过滤
  function searchDevices(keyword: string) {
    return fetchDevices({
      name_contains: keyword || undefined,
      page: 1
    })
  }

  function filterByProtocol(protocol: string) {
    return fetchDevices({
      protocol: protocol || undefined,
      page: 1
    })
  }

  function filterByStatus(enabled: boolean) {
    return fetchDevices({
      enabled,
      page: 1
    })
  }

  function resetSearch() {
    return fetchDevices({
      page: 1,
      size: pagination.value.size
    })
  }

  // 分页操作
  function changePage(page: number) {
    return fetchDevices({ page })
  }

  function changePageSize(size: number) {
    return fetchDevices({ size, page: 1 })
  }

  // 选择操作
  function selectDevices(deviceList: Device[]) {
    selectedDevices.value = deviceList
  }

  function toggleSelectDevice(device: Device) {
    const index = selectedDevices.value.findIndex(d => d.id === device.id)
    if (index !== -1) {
      selectedDevices.value.splice(index, 1)
    } else {
      selectedDevices.value.push(device)
    }
  }

  function toggleSelectAll() {
    if (selectedDevices.value.length === devices.value.length) {
      selectedDevices.value = []
    } else {
      selectedDevices.value = [...devices.value]
    }
  }

  function clearSelection() {
    selectedDevices.value = []
  }

  function setCurrentDevice(device: Device | null) {
    currentDevice.value = device
  }

  // 刷新
  function refresh() {
    return fetchDevices(searchQuery.value)
  }

  // 重置
  function reset() {
    loading.value = false
    error.value = null
    devices.value = []
    selectedDevices.value = []
    currentDevice.value = null
    pagination.value = {
      page: 1,
      size: 20,
      total: 0,
      pages: 0
    }
    searchQuery.value = {
      page: 1,
      size: 20
    }
  }

  return {
    // 状态
    loading: computed(() => loading.value),
    error: computed(() => error.value),
    devices: computed(() => devices.value),
    selectedDevices: computed(() => selectedDevices.value),
    currentDevice: computed(() => currentDevice.value),
    pagination: computed(() => pagination.value),
    searchQuery: computed(() => searchQuery.value),
    
    // 计算属性
    enabledDevices,
    disabledDevices,
    devicesByProtocol,
    hasDevices,
    isLoading,
    hasError,
    totalPages,
    selectedIds,
    
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
    reset
  }
})

export type DevicesStore = ReturnType<typeof useDevicesStore>