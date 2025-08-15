/**
 * 设备管理 Pinia Store
 *
 * 使用新的基础架构，提供统一的状态管理和错误处理
 */

import { defineStore } from 'pinia'
import { computed } from 'vue'
import { BaseStore } from './base'
import { devicesApi } from '@/api'
import type {
  DeviceVO,
  DeviceCreateReq,
  DevicePatchReq,
  DeviceQuery,
  ProtocolKind,
  ApiResponse,
} from '@/types/api'

/**
 * 设备Store类
 */
class DevicesStore extends BaseStore<DeviceVO, DeviceCreateReq, DevicePatchReq, DeviceQuery> {
  protected getApiClient() {
    return {
      list: (query: DeviceQuery) => devicesApi.list(query),
      get: (id: string) => devicesApi.get(id),
      create: (data: DeviceCreateReq) => devicesApi.create(data),
      update: (id: string, data: DevicePatchReq) => devicesApi.update(id, data),
      delete: (id: string) => devicesApi.delete(id),
    }
  }

  protected getItemId(item: DeviceVO): string {
    return item.id
  }

  protected getItemName(item: DeviceVO): string {
    return item.name
  }

  // 设备特定的计算属性
  get enabledDevices() {
    return computed(() => this.items.value.filter(device => device.enabled))
  }

  get disabledDevices() {
    return computed(() => this.items.value.filter(device => !device.enabled))
  }

  get devicesByProtocol() {
    return computed(() => {
      const grouped: Record<string, DeviceVO[]> = {}
      this.items.value.forEach(device => {
        if (!grouped[device.protocol]) {
          grouped[device.protocol] = []
        }
        grouped[device.protocol].push(device)
      })
      return grouped
    })
  }

  get devicesByStatus() {
    return computed(() => {
      const enabled = this.items.value.filter(d => d.enabled)
      const disabled = this.items.value.filter(d => !d.enabled)
      return { enabled, disabled }
    })
  }

  // 设备特定方法

  /**
   * 切换设备状态
   */
  async toggleDeviceStatus(id: string, enabled: boolean) {
    return this.safeExecute(async () => {
      const result = await this.updateItem(id, { enabled } as DevicePatchReq)
      return result.data
    }, { 
      showLoading: true, 
      showSuccess: enabled ? '设备已启用' : '设备已禁用' 
    })
  }

  /**
   * 测试设备连接
   */
  async testDeviceConnection(data: DeviceCreateReq) {
    return this.safeExecute(async () => {
      const response = await devicesApi.testConnection(data)
      const result = this.handleResponse(response)
      return result
    }, { 
      showLoading: true, 
      showSuccess: '连接测试成功' 
    })
  }

  /**
   * 按协议筛选
   */
  async filterByProtocol(protocol: ProtocolKind | undefined) {
    return this.fetchItems({
      ...this.query.value,
      protocol,
      page: 1,
    })
  }

  /**
   * 按状态筛选
   */
  async filterByStatus(enabled: boolean | undefined) {
    return this.fetchItems({
      ...this.query.value,
      enabled,
      page: 1,
    })
  }

  /**
   * 按位置筛选
   */
  async filterByLocation(location: string | undefined) {
    return this.fetchItems({
      ...this.query.value,
      location_contains: location,
      page: 1,
    })
  }

  /**
   * 复合筛选
   */
  async applyFilters(filters: {
    protocol?: ProtocolKind;
    enabled?: boolean;
    location?: string;
    keyword?: string;
  }) {
    return this.fetchItems({
      ...this.query.value,
      protocol: filters.protocol,
      enabled: filters.enabled,
      location_contains: filters.location,
      name_contains: filters.keyword,
      page: 1,
    })
  }

  /**
   * 获取设备统计信息
   */
  get deviceStats() {
    return computed(() => {
      const total = this.items.value.length
      const enabled = this.enabledDevices.value.length
      const disabled = this.disabledDevices.value.length
      const protocols = Object.keys(this.devicesByProtocol.value)
      
      return {
        total,
        enabled,
        disabled,
        protocols: protocols.length,
        protocolDistribution: this.devicesByProtocol.value,
      }
    })
  }

  /**
   * 批量操作
   */
  async batchUpdateStatus(ids: string[], enabled: boolean) {
    return this.safeExecute(async () => {
      const results = await Promise.allSettled(
        ids.map(id => this.updateItem(id, { enabled } as DevicePatchReq))
      )
      
      let successful = 0
      let failed = 0
      
      results.forEach(result => {
        if (result.status === 'fulfilled') {
          successful++
        } else {
          failed++
        }
      })
      
      return { successful, failed }
    }, {
      showLoading: true,
      showSuccess: `批量${enabled ? '启用' : '禁用'}完成：成功 ${ids.length} 个设备`
    })
  }

  /**
   * 重置搜索条件
   */
  async resetFilters() {
    this.query.value = {
      page: 1,
      size: this.pagination.value.size,
    }
    return this.fetchItems()
  }

  /**
   * 高级搜索
   */
  async advancedSearch(criteria: {
    name?: string;
    protocol?: ProtocolKind;
    location?: string;
    enabled?: boolean;
    endpoint?: string;
    tags?: string[];
  }) {
    return this.fetchItems({
      page: 1,
      size: this.pagination.value.size,
      name_contains: criteria.name,
      protocol: criteria.protocol,
      location_contains: criteria.location,
      enabled: criteria.enabled,
      endpoint_contains: criteria.endpoint,
      // 可以扩展更多搜索条件
    })
  }
}

/**
 * 创建设备Store实例
 */
const devicesStore = new DevicesStore()

/**
 * 设备Store - 使用Pinia defineStore
 */
export const useDevicesStore = defineStore('devices', () => {
  return {
    // Store实例属性
    ...devicesStore,

    // 状态访问器
    state: devicesStore.$state,
    items: devicesStore.$items,
    pagination: devicesStore.$pagination,
    query: devicesStore.$query,
    selectedItems: devicesStore.$selectedItems,
    currentItem: devicesStore.$currentItem,

    // 计算属性
    isLoading: devicesStore.isLoading,
    hasError: devicesStore.hasError,
    hasItems: devicesStore.hasItems,
    itemCount: devicesStore.itemCount,
    totalPages: devicesStore.totalPages,
    canLoadMore: devicesStore.canLoadMore,
    
    // 设备特定计算属性
    enabledDevices: devicesStore.enabledDevices,
    disabledDevices: devicesStore.disabledDevices,
    devicesByProtocol: devicesStore.devicesByProtocol,
    devicesByStatus: devicesStore.devicesByStatus,
    deviceStats: devicesStore.deviceStats,

    // 方法
    fetchItems: devicesStore.fetchItems.bind(devicesStore),
    fetchItem: devicesStore.fetchItem.bind(devicesStore),
    createItem: devicesStore.createItem.bind(devicesStore),
    updateItem: devicesStore.updateItem.bind(devicesStore),
    deleteItem: devicesStore.deleteItem.bind(devicesStore),
    batchDeleteItems: devicesStore.batchDeleteItems.bind(devicesStore),
    refresh: devicesStore.refresh.bind(devicesStore),
    search: devicesStore.search.bind(devicesStore),
    changePage: devicesStore.changePage.bind(devicesStore),
    changePageSize: devicesStore.changePageSize.bind(devicesStore),
    selectItems: devicesStore.selectItems.bind(devicesStore),
    toggleSelectItem: devicesStore.toggleSelectItem.bind(devicesStore),
    toggleSelectAll: devicesStore.toggleSelectAll.bind(devicesStore),
    clearSelection: devicesStore.clearSelection.bind(devicesStore),
    setCurrentItem: devicesStore.setCurrentItem.bind(devicesStore),
    reset: devicesStore.reset.bind(devicesStore),

    // 设备特定方法
    toggleDeviceStatus: devicesStore.toggleDeviceStatus.bind(devicesStore),
    testDeviceConnection: devicesStore.testDeviceConnection.bind(devicesStore),
    filterByProtocol: devicesStore.filterByProtocol.bind(devicesStore),
    filterByStatus: devicesStore.filterByStatus.bind(devicesStore),
    filterByLocation: devicesStore.filterByLocation.bind(devicesStore),
    applyFilters: devicesStore.applyFilters.bind(devicesStore),
    batchUpdateStatus: devicesStore.batchUpdateStatus.bind(devicesStore),
    resetFilters: devicesStore.resetFilters.bind(devicesStore),
    advancedSearch: devicesStore.advancedSearch.bind(devicesStore),
    
    // 其他别名方法（保持向后兼容）
    fetchDevices: (params?: any) => devicesStore.fetchItems(params),
    fetchDevice: (id: string) => devicesStore.fetchItem(id),
    createDevice: (data: DeviceCreateReq) => devicesStore.createItem(data),
    updateDevice: (id: string, data: DevicePatchReq) => devicesStore.updateItem(id, data),
    deleteDevice: (id: string) => devicesStore.deleteItem(id),
    batchDeleteDevices: (ids: string[]) => devicesStore.batchDeleteItems(ids),
    searchDevices: (keyword: string) => devicesStore.search(keyword),
    selectDevices: (devices: DeviceVO[]) => devicesStore.selectItems(devices),
    toggleSelectDevice: (device: DeviceVO) => devicesStore.toggleSelectItem(device),
    setCurrentDevice: (device: DeviceVO | null) => devicesStore.setCurrentItem(device),
    toggleDevice: (id: string, enabled: boolean) => devicesStore.toggleDeviceStatus(id, enabled),
  }
})

// 类型导出
export type DevicesStore = ReturnType<typeof useDevicesStore>
export type DeviceState = {
  devices: DeviceVO[];
  total: number;
  currentPage: number;
  pageSize: number;
  loading: boolean;
  query: DeviceQuery;
  selectedDevices: DeviceVO[];
  currentDevice: DeviceVO | null;
}