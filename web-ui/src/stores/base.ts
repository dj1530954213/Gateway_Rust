/**
 * Base Store - 通用Store抽象基类
 * 
 * 提供标准化的状态管理模式和错误处理机制
 */

import { ref, computed, readonly } from 'vue'
import { ElMessage } from 'element-plus'
import type { ApiResponse, ApiError } from '@/types/api'

/**
 * 基础状态接口
 */
export interface BaseState {
  loading: boolean
  error: ApiError | null
  lastUpdated: Date | null
}

/**
 * 基础查询参数
 */
export interface BaseQuery {
  page?: number
  size?: number
  [key: string]: any
}

/**
 * 分页信息
 */
export interface PaginationInfo {
  page: number
  size: number
  total: number
  pages: number
  hasNext: boolean
  hasPrev: boolean
}

/**
 * 异步操作结果
 */
export interface OperationResult<T = any> {
  success: boolean
  data?: T
  error?: string
}

/**
 * 基础Store类
 */
export abstract class BaseStore<TItem, TCreateReq, TUpdateReq, TQuery extends BaseQuery> {
  // 状态
  protected state = ref<BaseState>({
    loading: false,
    error: null,
    lastUpdated: null,
  })

  // 数据列表
  protected items = ref<TItem[]>([])
  
  // 分页信息
  protected pagination = ref<PaginationInfo>({
    page: 1,
    size: 20,
    total: 0,
    pages: 0,
    hasNext: false,
    hasPrev: false,
  })

  // 查询参数
  protected query = ref<TQuery>({
    page: 1,
    size: 20,
  } as TQuery)

  // 选中项
  protected selectedItems = ref<TItem[]>([])
  
  // 当前编辑项
  protected currentItem = ref<TItem | null>(null)

  // 计算属性
  get isLoading() {
    return computed(() => this.state.value.loading)
  }

  get hasError() {
    return computed(() => this.state.value.error !== null)
  }

  get hasItems() {
    return computed(() => this.items.value.length > 0)
  }

  get itemCount() {
    return computed(() => this.items.value.length)
  }

  get totalPages() {
    return computed(() => this.pagination.value.pages)
  }

  get canLoadMore() {
    return computed(() => this.pagination.value.hasNext)
  }

  // 只读访问器
  get $state() {
    return readonly(this.state)
  }

  get $items() {
    return readonly(this.items)
  }

  get $pagination() {
    return readonly(this.pagination)
  }

  get $query() {
    return readonly(this.query)
  }

  get $selectedItems() {
    return readonly(this.selectedItems)
  }

  get $currentItem() {
    return readonly(this.currentItem)
  }

  // 抽象方法，子类必须实现
  protected abstract getApiClient(): {
    list: (query: TQuery) => Promise<ApiResponse<{ items: TItem[]; total: number; page: number; size: number }>>;
    get: (id: string) => Promise<ApiResponse<TItem>>;
    create: (data: TCreateReq) => Promise<ApiResponse<TItem>>;
    update: (id: string, data: TUpdateReq) => Promise<ApiResponse<TItem>>;
    delete: (id: string) => Promise<ApiResponse<void>>;
  }

  protected abstract getItemId(item: TItem): string
  protected abstract getItemName(item: TItem): string

  // 通用方法实现
  
  /**
   * 设置加载状态
   */
  protected setLoading(loading: boolean) {
    this.state.value.loading = loading
  }

  /**
   * 设置错误状态
   */
  protected setError(error: ApiError | null) {
    this.state.value.error = error
    if (error) {
      ElMessage.error(error.message || '操作失败')
    }
  }

  /**
   * 处理API响应
   */
  protected handleResponse<T>(response: ApiResponse<T>): T {
    if (!response.success || !response.data) {
      throw new Error(response.error?.message || '操作失败')
    }
    return response.data
  }

  /**
   * 安全执行异步操作
   */
  protected async safeExecute<T>(
    operation: () => Promise<T>,
    options: {
      showLoading?: boolean;
      showSuccess?: string;
      showError?: boolean;
      onSuccess?: (result: T) => void;
      onError?: (error: Error) => void;
    } = {}
  ): Promise<OperationResult<T>> {
    const {
      showLoading = true,
      showSuccess,
      showError = true,
      onSuccess,
      onError,
    } = options

    try {
      if (showLoading) {
        this.setLoading(true)
      }
      
      this.setError(null)
      
      const result = await operation()
      
      this.state.value.lastUpdated = new Date()
      
      if (showSuccess) {
        ElMessage.success(showSuccess)
      }
      
      if (onSuccess) {
        onSuccess(result)
      }
      
      return { success: true, data: result }
    } catch (error) {
      const errorObj = error instanceof Error ? error : new Error(String(error))
      
      if (showError) {
        this.setError({
          code: 'OPERATION_FAILED',
          message: errorObj.message,
        } as ApiError)
      }
      
      if (onError) {
        onError(errorObj)
      }
      
      return { success: false, error: errorObj.message }
    } finally {
      if (showLoading) {
        this.setLoading(false)
      }
    }
  }

  /**
   * 获取列表
   */
  async fetchItems(params?: Partial<TQuery>): Promise<OperationResult<TItem[]>> {
    const finalQuery = { ...this.query.value, ...params } as TQuery
    
    return this.safeExecute(async () => {
      const response = await this.getApiClient().list(finalQuery)
      const data = this.handleResponse(response)
      
      this.items.value = data.items
      this.pagination.value = {
        page: data.page,
        size: data.size,
        total: data.total,
        pages: Math.ceil(data.total / data.size),
        hasNext: data.page < Math.ceil(data.total / data.size),
        hasPrev: data.page > 1,
      }
      this.query.value = finalQuery
      
      return data.items
    }, { showLoading: true })
  }

  /**
   * 获取详情
   */
  async fetchItem(id: string): Promise<OperationResult<TItem>> {
    return this.safeExecute(async () => {
      const response = await this.getApiClient().get(id)
      const data = this.handleResponse(response)
      
      this.currentItem.value = data
      return data
    }, { showLoading: true })
  }

  /**
   * 创建项目
   */
  async createItem(data: TCreateReq, options?: { addToList?: boolean }): Promise<OperationResult<TItem>> {
    const { addToList = true } = options || {}
    
    return this.safeExecute(async () => {
      const response = await this.getApiClient().create(data)
      const item = this.handleResponse(response)
      
      if (addToList) {
        this.items.value.unshift(item)
        this.pagination.value.total += 1
      }
      
      return item
    }, { showLoading: true, showSuccess: '创建成功' })
  }

  /**
   * 更新项目
   */
  async updateItem(id: string, data: TUpdateReq): Promise<OperationResult<TItem>> {
    return this.safeExecute(async () => {
      const response = await this.getApiClient().update(id, data)
      const item = this.handleResponse(response)
      
      // 更新列表中的项目
      const index = this.items.value.findIndex(i => this.getItemId(i) === id)
      if (index !== -1) {
        this.items.value[index] = item
      }
      
      // 更新当前项目
      if (this.currentItem.value && this.getItemId(this.currentItem.value) === id) {
        this.currentItem.value = item
      }
      
      return item
    }, { showLoading: true, showSuccess: '更新成功' })
  }

  /**
   * 删除项目
   */
  async deleteItem(id: string): Promise<OperationResult<void>> {
    return this.safeExecute(async () => {
      const response = await this.getApiClient().delete(id)
      this.handleResponse(response)
      
      // 从列表中移除
      this.items.value = this.items.value.filter(i => this.getItemId(i) !== id)
      this.pagination.value.total -= 1
      
      // 清除当前项目
      if (this.currentItem.value && this.getItemId(this.currentItem.value) === id) {
        this.currentItem.value = null
      }
      
      // 从选中列表中移除
      this.selectedItems.value = this.selectedItems.value.filter(i => this.getItemId(i) !== id)
    }, { showLoading: true, showSuccess: '删除成功' })
  }

  /**
   * 批量删除
   */
  async batchDeleteItems(ids: string[]): Promise<OperationResult<{ successful: number; failed: number }>> {
    return this.safeExecute(async () => {
      const results = await Promise.allSettled(
        ids.map(id => this.getApiClient().delete(id))
      )
      
      let successful = 0
      let failed = 0
      
      results.forEach((result, index) => {
        if (result.status === 'fulfilled') {
          const id = ids[index]
          // 从列表中移除成功删除的项目
          this.items.value = this.items.value.filter(i => this.getItemId(i) !== id)
          successful++
        } else {
          failed++
        }
      })
      
      this.pagination.value.total -= successful
      this.selectedItems.value = []
      
      return { successful, failed }
    }, { 
      showLoading: true, 
      showSuccess: `批量删除完成：成功 ${ids.length - (await this.batchDeleteItems(ids)).data?.failed || 0} 项` 
    })
  }

  /**
   * 刷新列表
   */
  async refresh(): Promise<OperationResult<TItem[]>> {
    return this.fetchItems(this.query.value)
  }

  /**
   * 搜索
   */
  async search(keyword: string, field: keyof TQuery = 'name_contains' as keyof TQuery): Promise<OperationResult<TItem[]>> {
    const searchQuery = {
      ...this.query.value,
      [field]: keyword || undefined,
      page: 1,
    } as TQuery
    
    return this.fetchItems(searchQuery)
  }

  /**
   * 分页跳转
   */
  async changePage(page: number): Promise<OperationResult<TItem[]>> {
    return this.fetchItems({ ...this.query.value, page } as Partial<TQuery>)
  }

  /**
   * 改变分页大小
   */
  async changePageSize(size: number): Promise<OperationResult<TItem[]>> {
    return this.fetchItems({ ...this.query.value, size, page: 1 } as Partial<TQuery>)
  }

  // 选择相关方法

  /**
   * 选择项目
   */
  selectItems(items: TItem[]) {
    this.selectedItems.value = items
  }

  /**
   * 切换选择
   */
  toggleSelectItem(item: TItem) {
    const id = this.getItemId(item)
    const index = this.selectedItems.value.findIndex(i => this.getItemId(i) === id)
    
    if (index !== -1) {
      this.selectedItems.value.splice(index, 1)
    } else {
      this.selectedItems.value.push(item)
    }
  }

  /**
   * 全选/取消全选
   */
  toggleSelectAll() {
    if (this.selectedItems.value.length === this.items.value.length) {
      this.selectedItems.value = []
    } else {
      this.selectedItems.value = [...this.items.value]
    }
  }

  /**
   * 清空选择
   */
  clearSelection() {
    this.selectedItems.value = []
  }

  /**
   * 设置当前项目
   */
  setCurrentItem(item: TItem | null) {
    this.currentItem.value = item
  }

  /**
   * 重置状态
   */
  reset() {
    this.state.value = {
      loading: false,
      error: null,
      lastUpdated: null,
    }
    this.items.value = []
    this.pagination.value = {
      page: 1,
      size: 20,
      total: 0,
      pages: 0,
      hasNext: false,
      hasPrev: false,
    }
    this.query.value = { page: 1, size: 20 } as TQuery
    this.selectedItems.value = []
    this.currentItem.value = null
  }
}

/**
 * 创建基础Store的工厂函数
 */
export function createBaseStore<TItem, TCreateReq, TUpdateReq, TQuery extends BaseQuery>() {
  return BaseStore<TItem, TCreateReq, TUpdateReq, TQuery>
}