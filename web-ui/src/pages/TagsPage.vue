<template>
  <div class="tags-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="page-title">
        <h1>点位管理</h1>
        <p class="page-description">
          管理数据点位配置，包括创建、编辑、删除数据点位以及批量导入导出
        </p>
      </div>
    </div>

    <!-- 搜索和筛选区域 -->
    <el-card class="search-card" shadow="never">
      <div class="search-area">
        <div class="search-left">
          <!-- 关键词搜索 -->
          <el-input
            v-model="searchKeyword"
            placeholder="搜索点位名称..."
            class="search-input"
            clearable
            @keyup.enter="handleSearch"
            @clear="handleSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>

          <!-- 设备筛选 -->
          <el-select
            v-model="selectedDevice"
            placeholder="选择设备"
            clearable
            class="filter-select"
            @change="handleDeviceFilter"
          >
            <el-option label="全部设备" value="" />
            <el-option
              v-for="device in availableDevices"
              :key="device.id"
              :label="device.name"
              :value="device.id"
            />
          </el-select>

          <!-- 数据类型筛选 -->
          <el-select
            v-model="selectedDataType"
            placeholder="数据类型"
            clearable
            class="filter-select"
            @change="handleDataTypeFilter"
          >
            <el-option label="全部类型" value="" />
            <el-option label="布尔型" value="Bool" />
            <el-option label="整数型" value="Int32" />
            <el-option label="浮点型" value="Float32" />
            <el-option label="字符串" value="String" />
          </el-select>

          <!-- 访问模式筛选 -->
          <el-select
            v-model="selectedAccessMode"
            placeholder="访问模式"
            clearable
            class="filter-select"
            @change="handleAccessModeFilter"
          >
            <el-option label="全部模式" value="" />
            <el-option label="只读" value="ReadOnly" />
            <el-option label="只写" value="WriteOnly" />
            <el-option label="读写" value="ReadWrite" />
          </el-select>
        </div>

        <div class="search-right">
          <el-button @click="handleResetFilters">
            <el-icon><Refresh /></el-icon>
            重置筛选
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- 统计卡片 -->
    <div class="stats-cards">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card class="stats-card" shadow="never">
            <div class="stats-item">
              <div class="stats-icon total">
                <el-icon><DataLine /></el-icon>
              </div>
              <div class="stats-content">
                <div class="stats-value">{{ tagsStore.state.total }}</div>
                <div class="stats-label">总点位数</div>
              </div>
            </div>
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card class="stats-card" shadow="never">
            <div class="stats-item">
              <div class="stats-icon read-only">
                <el-icon><View /></el-icon>
              </div>
              <div class="stats-content">
                <div class="stats-value">
                  {{ tagsStore.readOnlyTags.length }}
                </div>
                <div class="stats-label">只读点位</div>
              </div>
            </div>
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card class="stats-card" shadow="never">
            <div class="stats-item">
              <div class="stats-icon write-only">
                <el-icon><Edit /></el-icon>
              </div>
              <div class="stats-content">
                <div class="stats-value">
                  {{ tagsStore.writeOnlyTags.length }}
                </div>
                <div class="stats-label">只写点位</div>
              </div>
            </div>
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card class="stats-card" shadow="never">
            <div class="stats-item">
              <div class="stats-icon read-write">
                <el-icon><EditPen /></el-icon>
              </div>
              <div class="stats-content">
                <div class="stats-value">
                  {{ tagsStore.readWriteTags.length }}
                </div>
                <div class="stats-label">读写点位</div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 操作栏 -->
    <el-card class="action-card" shadow="never">
      <div class="action-bar">
        <div class="action-left">
          <el-button type="primary" @click="handleCreateTag">
            <el-icon><Plus /></el-icon>
            新增点位
          </el-button>

          <el-button
            type="danger"
            :disabled="!hasSelectedTags"
            @click="handleBatchDelete"
          >
            <el-icon><Delete /></el-icon>
            批量删除
          </el-button>

          <el-upload
            ref="uploadRef"
            :show-file-list="false"
            :on-change="handleImportFile"
            :auto-upload="false"
            accept=".csv,.xlsx,.xls"
          >
            <el-button type="success">
              <el-icon><Upload /></el-icon>
              导入点位
            </el-button>
          </el-upload>

          <el-button @click="handleExport">
            <el-icon><Download /></el-icon>
            导出点位
          </el-button>
        </div>

        <div class="action-right">
          <span v-if="hasSelectedTags" class="selection-info">
            已选择 {{ selectedTagsCount }} 个点位
          </span>

          <el-button
            type="info"
            :loading="tagsStore.isLoading"
            @click="handleRefresh"
          >
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- 点位表格 -->
    <el-card class="table-card" shadow="never">
      <TagsTable
        :tags="tagsStore.state.tags"
        :loading="tagsStore.isLoading"
        :selected-tags="tagsStore.state.selectedTags"
        @selection-change="handleSelectionChange"
        @edit-tag="handleEditTag"
        @delete-tag="handleDeleteTag"
        @toggle-tag="handleToggleTag"
        @test-read="handleTestRead"
        @test-write="handleTestWrite"
      />

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="tagsStore.state.total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handlePageSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 点位表单对话框 -->
    <TagFormDialog
      v-model:visible="formDialogVisible"
      :tag="currentTag"
      :mode="formMode"
      :available-devices="availableDevices"
      @success="handleFormSuccess"
    />

    <!-- 导入进度对话框 -->
    <el-dialog
      v-model="importDialogVisible"
      title="导入点位"
      width="400px"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
    >
      <div class="import-progress">
        <el-progress
          :percentage="tagsStore.state.importProgress"
          :status="importStatus"
        />
        <p class="import-text">{{ importText }}</p>
      </div>

      <template #footer>
        <el-button
          v-if="importStatus !== 'success'"
          @click="importDialogVisible = false"
        >
          取消
        </el-button>
        <el-button v-else type="primary" @click="importDialogVisible = false">
          完成
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * TagsPage —— 点位管理页面
 *
 * 📝 Responsibilities:
 *  1. 点位列表展示与管理
 *  2. 搜索和筛选功能
 *  3. 批量操作（删除、导入、导出）
 *  4. 点位CRUD操作界面
 *
 * 📦 Dependencies:
 *  - Element Plus UI 组件
 *  - Tags Store 状态管理
 *  - Tags API 接口
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Search,
  Refresh,
  DataLine,
  View,
  Edit,
  EditPen,
  Plus,
  Delete,
  Upload,
  Download,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { UploadFile } from 'element-plus'
import { ref, computed, onMounted, watch } from 'vue'

import type { DeviceVO } from '@/api/devices'
import type { TagVO, TagCreateReq } from '@/api/tags'
import TagFormDialog from '@/components/tags/TagFormDialog.vue'
import TagsTable from '@/components/tags/TagsTable.vue'
import { useTagsStore, useDevicesStore } from '@/stores'

// ===== Store =====
const tagsStore = useTagsStore()
const devicesStore = useDevicesStore()

// ===== 响应式数据 =====
const searchKeyword = ref('')
const selectedDevice = ref('')
const selectedDataType = ref('')
const selectedAccessMode = ref('')

// 表单对话框
const formDialogVisible = ref(false)
const formMode = ref<'create' | 'edit'>('create')
const currentTag = ref<TagVO | null>(null)

// 分页
const currentPage = ref(1)
const pageSize = ref(20)

// 导入相关
const uploadRef = ref()
const importDialogVisible = ref(false)

// ===== 计算属性 =====
const hasSelectedTags = computed(() => {
  return tagsStore.state.selectedTags.length > 0
})

const selectedTagsCount = computed(() => {
  return tagsStore.state.selectedTags.length
})

const availableDevices = computed(() => {
  return devicesStore.state.devices.filter(d => d.enabled)
})

const importStatus = computed(() => {
  if (tagsStore.state.importProgress === 100) return 'success'
  if (tagsStore.state.importing) return 'active'
  return ''
})

const importText = computed(() => {
  if (tagsStore.state.importProgress === 100) {
    return '导入完成'
  }
  if (tagsStore.state.importing) {
    return `正在导入... ${tagsStore.state.importProgress}%`
  }
  return '准备导入'
})

// ===== 方法 =====

/**
 * 处理搜索
 */
async function handleSearch() {
  await tagsStore.searchTags(searchKeyword.value.trim())
  currentPage.value = 1
}

/**
 * 处理设备筛选
 */
async function handleDeviceFilter() {
  await tagsStore.filterByDevice(selectedDevice.value || undefined)
  currentPage.value = 1
}

/**
 * 处理数据类型筛选
 */
async function handleDataTypeFilter() {
  await tagsStore.filterByDataType((selectedDataType.value as any) || undefined)
  currentPage.value = 1
}

/**
 * 处理访问模式筛选
 */
async function handleAccessModeFilter() {
  await tagsStore.filterByAccessMode(
    (selectedAccessMode.value as any) || undefined
  )
  currentPage.value = 1
}

/**
 * 重置筛选条件
 */
async function handleResetFilters() {
  searchKeyword.value = ''
  selectedDevice.value = ''
  selectedDataType.value = ''
  selectedAccessMode.value = ''
  await tagsStore.resetSearch()
  currentPage.value = 1
}

/**
 * 创建点位
 */
function handleCreateTag() {
  currentTag.value = null
  formMode.value = 'create'
  formDialogVisible.value = true
}

/**
 * 编辑点位
 */
function handleEditTag(tag: TagVO) {
  currentTag.value = tag
  formMode.value = 'edit'
  formDialogVisible.value = true
}

/**
 * 删除点位
 */
async function handleDeleteTag(tag: TagVO) {
  await tagsStore.deleteTag(tag.id)
}

/**
 * 切换点位启用状态
 */
async function handleToggleTag(tag: TagVO) {
  const newEnabled = !tag.enabled
  const result = await tagsStore.updateTag(tag.id, { enabled: newEnabled })
  if (result) {
    ElMessage.success(`点位已${newEnabled ? '启用' : '禁用'}`)
  }
}

/**
 * 测试读取
 */
async function handleTestRead(tag: TagVO) {
  try {
    // TODO: 实现点位读取测试
    ElMessage.info('读取测试功能开发中...')
  } catch (error) {
    ElMessage.error('读取测试失败')
  }
}

/**
 * 测试写入
 */
async function handleTestWrite(tag: TagVO) {
  try {
    // TODO: 实现点位写入测试
    ElMessage.info('写入测试功能开发中...')
  } catch (error) {
    ElMessage.error('写入测试失败')
  }
}

/**
 * 批量删除
 */
async function handleBatchDelete() {
  const selectedIds = tagsStore.state.selectedTags.map(t => t.id)
  await tagsStore.batchDeleteTags(selectedIds)
}

/**
 * 处理导入文件
 */
async function handleImportFile(file: UploadFile) {
  if (!file.raw) return

  // 验证文件类型
  const allowedTypes = [
    'text/csv',
    'application/vnd.ms-excel',
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  ]

  if (!allowedTypes.includes(file.raw.type)) {
    ElMessage.error('只支持 CSV 和 Excel 文件')
    return
  }

  // 确认导入
  try {
    await ElMessageBox.confirm(
      '确定要导入这个文件吗？导入过程中请勿关闭页面。',
      '确认导入',
      {
        type: 'info',
        confirmButtonText: '导入',
        cancelButtonText: '取消',
      }
    )

    importDialogVisible.value = true
    await tagsStore.importTags(file.raw)
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('导入失败')
    }
  }

  // 清除文件选择
  uploadRef.value?.clearFiles()
}

/**
 * 导出点位
 */
async function handleExport() {
  try {
    await ElMessageBox.confirm(
      '确定要导出当前筛选条件下的所有点位吗？',
      '确认导出',
      {
        type: 'info',
        confirmButtonText: '导出',
        cancelButtonText: '取消',
      }
    )

    await tagsStore.exportTags('csv')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('导出失败')
    }
  }
}

/**
 * 处理选择变更
 */
function handleSelectionChange(tags: TagVO[]) {
  tagsStore.selectTags(tags)
}

/**
 * 表单成功处理
 */
function handleFormSuccess() {
  formDialogVisible.value = false
  // 表单提交成功后，Store已经自动更新了列表
}

/**
 * 刷新数据
 */
async function handleRefresh() {
  await tagsStore.refresh()
}

/**
 * 分页变更
 */
async function handlePageChange(page: number) {
  await tagsStore.changePage(page)
}

/**
 * 分页大小变更
 */
async function handlePageSizeChange(size: number) {
  await tagsStore.changePageSize(size)
  currentPage.value = 1
}

// ===== 监听器 =====
watch(
  () => tagsStore.state.currentPage,
  newPage => {
    currentPage.value = newPage
  }
)

watch(
  () => tagsStore.state.pageSize,
  newSize => {
    pageSize.value = newSize
  }
)

// ===== 生命周期 =====
onMounted(async () => {
  // 加载设备列表（用于筛选）
  if (devicesStore.state.devices.length === 0) {
    await devicesStore.fetchDevices()
  }

  // 加载点位列表
  await tagsStore.fetchTags()
})
</script>

<style scoped lang="scss">
.tags-page {
  padding: 24px;

  .page-header {
    margin-bottom: 24px;

    .page-title {
      h1 {
        margin: 0 0 8px 0;
        font-size: 24px;
        font-weight: 600;
        color: #303133;
      }

      .page-description {
        margin: 0;
        color: #606266;
        font-size: 14px;
        line-height: 1.5;
      }
    }
  }

  .search-card {
    margin-bottom: 20px;

    .search-area {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;

      .search-left {
        display: flex;
        align-items: center;
        gap: 12px;
        flex: 1;

        .search-input {
          width: 300px;
        }

        .filter-select {
          width: 140px;
        }
      }

      .search-right {
        display: flex;
        align-items: center;
        gap: 12px;
      }
    }
  }

  .stats-cards {
    margin-bottom: 20px;

    .stats-card {
      .stats-item {
        display: flex;
        align-items: center;
        gap: 12px;

        .stats-icon {
          width: 48px;
          height: 48px;
          border-radius: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 20px;

          &.total {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
          }

          &.read-only {
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            color: white;
          }

          &.write-only {
            background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
            color: white;
          }

          &.read-write {
            background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
            color: white;
          }
        }

        .stats-content {
          .stats-value {
            font-size: 24px;
            font-weight: 600;
            color: #303133;
            line-height: 1;
            margin-bottom: 4px;
          }

          .stats-label {
            font-size: 14px;
            color: #909399;
            line-height: 1;
          }
        }
      }
    }
  }

  .action-card {
    margin-bottom: 20px;

    .action-bar {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .action-left {
        display: flex;
        align-items: center;
        gap: 12px;
      }

      .action-right {
        display: flex;
        align-items: center;
        gap: 12px;

        .selection-info {
          font-size: 14px;
          color: #606266;
        }
      }
    }
  }

  .table-card {
    .pagination-wrapper {
      margin-top: 20px;
      display: flex;
      justify-content: center;
    }
  }
}

// 导入进度样式
.import-progress {
  text-align: center;
  padding: 20px 0;

  .import-text {
    margin-top: 12px;
    color: #606266;
    font-size: 14px;
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .tags-page {
    .search-area {
      flex-direction: column;
      align-items: stretch;

      .search-left {
        flex-wrap: wrap;

        .search-input {
          width: 100%;
        }

        .filter-select {
          flex: 1;
          min-width: 120px;
        }
      }
    }

    .stats-cards {
      .el-col {
        margin-bottom: 16px;
      }
    }

    .action-bar {
      flex-direction: column;
      align-items: stretch;
      gap: 16px;

      .action-left {
        flex-wrap: wrap;
      }

      .action-right {
        justify-content: center;
      }
    }
  }
}

@media (max-width: 768px) {
  .tags-page {
    padding: 16px;

    .stats-cards {
      .el-col {
        // CSS for responsive grid columns
      }
    }

    .action-left {
      .el-button {
        flex: 1;
        justify-content: center;
      }
    }
  }
}
</style>
