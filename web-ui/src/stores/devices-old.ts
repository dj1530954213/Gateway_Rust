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

// 创建并导出 store 实例
export const useDevicesStore = defineStore('devices', () => {
  const store = new DevicesStore()
  
  return {
    // 导出store的所有属性和方法
    ...store,
    
    // 状态
    state: store.state,
    loading: store.loading,
    error: store.error,
    
    // 计算属性
    items: store.items,
    total: store.total,
    pagination: store.pagination,
    selectedIds: store.selectedIds,
    currentItem: store.currentItem,
    
    // 方法
    fetchItems: store.fetchItems.bind(store),
    fetchItem: store.fetchItem.bind(store),
    createItem: store.createItem.bind(store),
    updateItem: store.updateItem.bind(store),
    deleteItem: store.deleteItem.bind(store),
    batchDelete: store.batchDelete.bind(store),
    refresh: store.refresh.bind(store),
    reset: store.reset.bind(store),
    setCurrentItem: store.setCurrentItem.bind(store),
    selectItems: store.selectItems.bind(store),
    toggleSelectItem: store.toggleSelectItem.bind(store),
    toggleSelectAll: store.toggleSelectAll.bind(store),
    clearSelection: store.clearSelection.bind(store),
    changePage: store.changePage.bind(store),
    changePageSize: store.changePageSize.bind(store),
    toggleDevice: store.toggleDevice.bind(store),
    testDeviceConnection: store.testDeviceConnection.bind(store),
    searchDevices: store.searchDevices.bind(store),
    filterByProtocol: store.filterByProtocol.bind(store),
    filterByStatus: store.filterByStatus.bind(store),
    resetFilters: store.resetFilters.bind(store),
    advancedSearch: store.advancedSearch.bind(store),
  }
})

// 类型导出
export type DevicesStore = ReturnType<typeof useDevicesStore>
