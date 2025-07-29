/**
 * 点位管理 Pinia Store
 * 
 * 管理数据点位列表、CRUD操作、搜索过滤、导入导出等状态
 */

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { tagsApi } from '@/api'
import type { 
  TagVO, 
  TagCreateReq, 
  TagPatchReq, 
  TagQuery, 
  TagDataType,
  PaginatedResponse 
} from '@/api/tags'

export interface TagState {
  // 点位列表数据
  tags: TagVO[]
  total: number
  
  // 分页信息
  currentPage: number
  pageSize: number
  
  // 加载状态
  loading: boolean
  
  // 查询条件
  query: TagQuery
  
  // 选中的点位
  selectedTags: TagVO[]
  
  // 当前编辑的点位
  currentTag: TagVO | null
  
  // 批量导入状态
  importing: boolean
  importProgress: number
}

export const useTagsStore = defineStore('tags', () => {
  // ===== 状态定义 =====
  const state = ref<TagState>({
    tags: [],
    total: 0,
    currentPage: 1,
    pageSize: 20,
    loading: false,
    query: {
      page: 1,
      size: 20,
    },
    selectedTags: [],
    currentTag: null,
    importing: false,
    importProgress: 0,
  })

  // ===== 计算属性 =====
  const totalPages = computed(() => {
    return Math.ceil(state.value.total / state.value.pageSize)
  })

  const hasTags = computed(() => {
    return state.value.tags.length > 0
  })

  const isLoading = computed(() => {
    return state.value.loading
  })

  const tagsByDataType = computed(() => {
    const grouped: Record<string, TagVO[]> = {}
    state.value.tags.forEach(tag => {
      if (!grouped[tag.data_type]) {
        grouped[tag.data_type] = []
      }
      grouped[tag.data_type].push(tag)
    })
    return grouped
  })

  const tagsByDevice = computed(() => {
    const grouped: Record<string, TagVO[]> = {}
    state.value.tags.forEach(tag => {
      if (!grouped[tag.device_id]) {
        grouped[tag.device_id] = []
      }
      grouped[tag.device_id].push(tag)
    })
    return grouped
  })

  const readOnlyTags = computed(() => {
    return state.value.tags.filter(tag => tag.access_mode === 'ReadOnly')
  })

  const writeOnlyTags = computed(() => {
    return state.value.tags.filter(tag => tag.access_mode === 'WriteOnly')
  })

  const readWriteTags = computed(() => {
    return state.value.tags.filter(tag => tag.access_mode === 'ReadWrite')
  })

  // ===== 操作方法 =====

  /**
   * 获取点位列表
   */
  async function fetchTags(params?: Partial<TagQuery>) {
    try {
      state.value.loading = true
      
      const query = {
        ...state.value.query,
        ...params,
      }

      const response: PaginatedResponse<TagVO> = await tagsApi.list(query)
      
      state.value.tags = response.items
      state.value.total = response.total
      state.value.currentPage = response.page
      state.value.pageSize = response.size
      
      // 更新查询条件
      state.value.query = { ...query }
      
    } catch (error) {
      console.error('获取点位列表失败:', error)
      ElMessage.error('获取点位列表失败')
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 获取点位详情
   */
  async function fetchTag(id: string): Promise<TagVO | null> {
    try {
      const tag = await tagsApi.get(id)
      state.value.currentTag = tag
      return tag
    } catch (error) {
      console.error('获取点位详情失败:', error)
      ElMessage.error('获取点位详情失败')
      return null
    }
  }

  /**
   * 创建点位
   */
  async function createTag(data: TagCreateReq): Promise<TagVO | null> {
    try {
      state.value.loading = true
      
      const tag = await tagsApi.create(data)
      
      // 添加到列表开头
      state.value.tags.unshift(tag)
      state.value.total += 1
      
      ElMessage.success('点位创建成功')
      return tag
      
    } catch (error) {
      console.error('创建点位失败:', error)
      ElMessage.error('创建点位失败')
      return null
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 更新点位
   */
  async function updateTag(id: string, data: TagPatchReq): Promise<TagVO | null> {
    try {
      state.value.loading = true
      
      const updatedTag = await tagsApi.update(id, data)
      
      // 更新列表中的点位
      const index = state.value.tags.findIndex(t => t.id === id)
      if (index !== -1) {
        state.value.tags[index] = updatedTag
      }
      
      // 更新当前点位
      if (state.value.currentTag?.id === id) {
        state.value.currentTag = updatedTag
      }
      
      ElMessage.success('点位更新成功')
      return updatedTag
      
    } catch (error) {
      console.error('更新点位失败:', error)
      ElMessage.error('更新点位失败')
      return null
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 删除点位
   */
  async function deleteTag(id: string): Promise<boolean> {
    try {
      await ElMessageBox.confirm(
        '确定要删除这个点位吗？删除后无法恢复。',
        '确认删除',
        {
          type: 'warning',
          confirmButtonText: '删除',
          cancelButtonText: '取消',
          confirmButtonClass: 'el-button--danger',
        }
      )

      state.value.loading = true
      
      await tagsApi.delete(id)
      
      // 从列表中移除
      state.value.tags = state.value.tags.filter(t => t.id !== id)
      state.value.total -= 1
      
      // 清除当前点位
      if (state.value.currentTag?.id === id) {
        state.value.currentTag = null
      }
      
      // 从选中列表中移除
      state.value.selectedTags = state.value.selectedTags.filter(t => t.id !== id)
      
      ElMessage.success('点位删除成功')
      return true
      
    } catch (error) {
      if (error !== 'cancel') {
        console.error('删除点位失败:', error)
        ElMessage.error('删除点位失败')
      }
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 批量删除点位
   */
  async function batchDeleteTags(ids: string[]): Promise<boolean> {
    try {
      await ElMessageBox.confirm(
        `确定要删除选中的 ${ids.length} 个点位吗？删除后无法恢复。`,
        '确认批量删除',
        {
          type: 'warning',
          confirmButtonText: '删除',
          cancelButtonText: '取消',
          confirmButtonClass: 'el-button--danger',
        }
      )

      state.value.loading = true
      
      // 并行删除所有点位
      const results = await Promise.allSettled(
        ids.map(id => tagsApi.delete(id))
      )
      
      // 统计成功和失败的数量
      const successCount = results.filter(r => r.status === 'fulfilled').length
      const failedCount = results.filter(r => r.status === 'rejected').length
      
      // 从列表中移除成功删除的点位
      const successIds = ids.slice(0, successCount)
      state.value.tags = state.value.tags.filter(t => !successIds.includes(t.id))
      state.value.total -= successCount
      
      // 清空选中列表
      state.value.selectedTags = []
      
      if (failedCount === 0) {
        ElMessage.success(`成功删除 ${successCount} 个点位`)
      } else {
        ElMessage.warning(`删除完成：成功 ${successCount} 个，失败 ${failedCount} 个`)
      }
      
      return failedCount === 0
      
    } catch (error) {
      if (error !== 'cancel') {
        console.error('批量删除点位失败:', error)
        ElMessage.error('批量删除点位失败')
      }
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 批量导入点位
   */
  async function importTags(file: File): Promise<boolean> {
    try {
      state.value.importing = true
      state.value.importProgress = 0
      
      const result = await tagsApi.importTags(file, (progress) => {
        state.value.importProgress = progress
      })
      
      if (result.success) {
        ElMessage.success(`成功导入 ${result.imported_count} 个点位`)
        // 刷新列表
        await fetchTags(state.value.query)
        return true
      } else {
        ElMessage.error(result.message || '导入失败')
        return false
      }
      
    } catch (error) {
      console.error('导入点位失败:', error)
      ElMessage.error('导入点位失败')
      return false
    } finally {
      state.value.importing = false
      state.value.importProgress = 0
    }
  }

  /**
   * 导出点位
   */
  async function exportTags(format: 'csv' | 'excel' = 'csv'): Promise<boolean> {
    try {
      state.value.loading = true
      
      const blob = await tagsApi.exportTags(format, state.value.query)
      
      // 创建下载链接
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `tags_export_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.${format}`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      window.URL.revokeObjectURL(url)
      
      ElMessage.success('导出成功')
      return true
      
    } catch (error) {
      console.error('导出点位失败:', error)
      ElMessage.error('导出点位失败')
      return false
    } finally {
      state.value.loading = false
    }
  }

  /**
   * 搜索点位
   */
  async function searchTags(keyword: string) {
    await fetchTags({
      ...state.value.query,
      name_contains: keyword || undefined,
      page: 1,
    })
  }

  /**
   * 按设备筛选
   */
  async function filterByDevice(deviceId: string | undefined) {
    await fetchTags({
      ...state.value.query,
      device_id: deviceId,
      page: 1,
    })
  }

  /**
   * 按数据类型筛选
   */
  async function filterByDataType(dataType: TagDataType | undefined) {
    await fetchTags({
      ...state.value.query,
      data_type: dataType,
      page: 1,
    })
  }

  /**
   * 按访问模式筛选
   */
  async function filterByAccessMode(accessMode: 'ReadOnly' | 'WriteOnly' | 'ReadWrite' | undefined) {
    await fetchTags({
      ...state.value.query,
      access_mode: accessMode,
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
    await fetchTags()
  }

  /**
   * 分页跳转
   */
  async function changePage(page: number) {
    await fetchTags({
      ...state.value.query,
      page,
    })
  }

  /**
   * 更改分页大小
   */
  async function changePageSize(size: number) {
    await fetchTags({
      ...state.value.query,
      size,
      page: 1,
    })
  }

  /**
   * 选择点位
   */
  function selectTags(tags: TagVO[]) {
    state.value.selectedTags = tags
  }

  /**
   * 切换点位选择
   */
  function toggleSelectTag(tag: TagVO) {
    const index = state.value.selectedTags.findIndex(t => t.id === tag.id)
    if (index !== -1) {
      state.value.selectedTags.splice(index, 1)
    } else {
      state.value.selectedTags.push(tag)
    }
  }

  /**
   * 全选/取消全选
   */
  function toggleSelectAll() {
    if (state.value.selectedTags.length === state.value.tags.length) {
      state.value.selectedTags = []
    } else {
      state.value.selectedTags = [...state.value.tags]
    }
  }

  /**
   * 清空选择
   */
  function clearSelection() {
    state.value.selectedTags = []
  }

  /**
   * 设置当前点位
   */
  function setCurrentTag(tag: TagVO | null) {
    state.value.currentTag = tag
  }

  /**
   * 刷新列表
   */
  async function refresh() {
    await fetchTags(state.value.query)
  }

  /**
   * 重置状态
   */
  function reset() {
    state.value.tags = []
    state.value.total = 0
    state.value.currentPage = 1
    state.value.pageSize = 20
    state.value.loading = false
    state.value.query = { page: 1, size: 20 }
    state.value.selectedTags = []
    state.value.currentTag = null
    state.value.importing = false
    state.value.importProgress = 0
  }

  // ===== 返回 Store API =====
  return {
    // 状态
    state: readonly(state),
    
    // 计算属性
    totalPages,
    hasTags,
    isLoading,
    tagsByDataType,
    tagsByDevice,
    readOnlyTags,
    writeOnlyTags,
    readWriteTags,
    
    // 方法
    fetchTags,
    fetchTag,
    createTag,
    updateTag,
    deleteTag,
    batchDeleteTags,
    importTags,
    exportTags,
    searchTags,
    filterByDevice,
    filterByDataType,
    filterByAccessMode,
    resetSearch,
    changePage,
    changePageSize,
    selectTags,
    toggleSelectTag,
    toggleSelectAll,
    clearSelection,
    setCurrentTag,
    refresh,
    reset,
  }
})

// 类型导出
export type TagsStore = ReturnType<typeof useTagsStore>