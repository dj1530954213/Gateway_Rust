<template>
  <div class="drivers-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="page-title">
        <h1>驱动管理</h1>
        <p class="page-description">
          管理动态链接库驱动，支持上传、热重载和状态监控
        </p>
      </div>
      <div class="page-actions">
        <el-button type="primary" @click="openUploadDialog">
          <el-icon><Upload /></el-icon>
          上传驱动
        </el-button>
        <el-button
          :loading="driversStore.isReloading"
          @click="reloadAllDrivers"
        >
          <el-icon><Refresh /></el-icon>
          重载全部
        </el-button>
        <el-button @click="refreshDrivers">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- 搜索和过滤区域 -->
    <el-card class="filter-card" shadow="never">
      <div class="filter-row">
        <div class="filter-item">
          <el-input
            v-model="searchKeyword"
            placeholder="搜索驱动文件名"
            clearable
            style="width: 240px"
            @input="handleSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>

        <div class="filter-item">
          <el-select
            v-model="selectedStatus"
            placeholder="加载状态"
            clearable
            style="width: 150px"
            @change="handleStatusFilter"
          >
            <el-option label="已加载" value="Loaded" />
            <el-option label="加载失败" value="Failed" />
            <el-option label="未加载" value="Unloaded" />
          </el-select>
        </div>

        <div class="filter-item">
          <el-select
            v-model="selectedProtocol"
            placeholder="支持协议"
            clearable
            style="width: 150px"
            @change="handleProtocolFilter"
          >
            <el-option
              v-for="protocol in availableProtocols"
              :key="protocol"
              :label="protocol"
              :value="protocol"
            />
          </el-select>
        </div>

        <div class="filter-item">
          <el-button @click="resetFilters">重置筛选</el-button>
        </div>
      </div>

      <!-- 批量操作栏 -->
      <div v-if="selectedDrivers.length > 0" class="batch-actions">
        <span class="selected-info"
          >已选择 {{ selectedDrivers.length }} 个驱动</span
        >
        <el-button type="danger" @click="handleBatchDelete">批量删除</el-button>
        <el-button @click="handleBatchReload">批量重载</el-button>
        <el-button @click="clearSelection">取消选择</el-button>
      </div>
    </el-card>

    <!-- 统计卡片 -->
    <div class="stats-row">
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ driversStore.state.total }}</div>
          <div class="stat-label">总驱动数</div>
        </div>
        <el-icon class="stat-icon"><Cpu /></el-icon>
      </el-card>

      <el-card class="stat-card loaded">
        <div class="stat-content">
          <div class="stat-value">
            {{ driversStore.state.statusStats.loaded }}
          </div>
          <div class="stat-label">已加载</div>
        </div>
        <el-icon class="stat-icon"><CircleCheck /></el-icon>
      </el-card>

      <el-card class="stat-card failed">
        <div class="stat-content">
          <div class="stat-value">
            {{ driversStore.state.statusStats.failed }}
          </div>
          <div class="stat-label">加载失败</div>
        </div>
        <el-icon class="stat-icon"><CircleClose /></el-icon>
      </el-card>

      <el-card class="stat-card unloaded">
        <div class="stat-content">
          <div class="stat-value">
            {{ driversStore.state.statusStats.unloaded }}
          </div>
          <div class="stat-label">未加载</div>
        </div>
        <el-icon class="stat-icon"><Warning /></el-icon>
      </el-card>
    </div>

    <!-- 驱动列表 -->
    <el-card class="table-card" shadow="never">
      <DriversTable
        :drivers="driversStore.state.drivers"
        :loading="driversStore.isLoading"
        :selected-drivers="selectedDrivers"
        @selection-change="handleSelectionChange"
        @delete-driver="handleDeleteDriver"
        @reload-driver="handleReloadDriver"
        @view-details="handleViewDetails"
      />

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="driversStore.state.total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 上传驱动对话框 -->
    <DriverUploadDialog
      v-model:visible="uploadDialogVisible"
      @success="handleUploadSuccess"
    />

    <!-- 驱动详情对话框 -->
    <DriverDetailsDialog
      v-model:visible="detailsDialogVisible"
      :driver="currentDriver"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * DriversPage —— 驱动管理主页面
 *
 * 📝 Responsibilities:
 *  1. 驱动列表展示和分页
 *  2. 搜索和过滤功能
 *  3. 驱动上传、删除、重载操作
 *  4. 批量操作支持
 *  5. 状态统计展示
 *
 * 📦 Dependencies:
 *  - DriversTable 驱动列表表格组件
 *  - DriverUploadDialog 驱动上传对话框
 *  - DriverDetailsDialog 驱动详情对话框
 *  - useDriversStore 驱动状态管理
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Upload,
  Refresh,
  Search,
  Cpu,
  CircleCheck,
  CircleClose,
  Warning,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, onMounted, watch } from 'vue'

import type { DriverVO } from '@/api/drivers'
import DriverDetailsDialog from '@/components/drivers/DriverDetailsDialog.vue'
import DriversTable from '@/components/drivers/DriversTable.vue'
import DriverUploadDialog from '@/components/drivers/DriverUploadDialog.vue'
import { useDriversStore } from '@/stores'

// ===== Store =====
const driversStore = useDriversStore()

// ===== 响应式数据 =====
const searchKeyword = ref('')
const selectedStatus = ref<'Loaded' | 'Failed' | 'Unloaded'>()
const selectedProtocol = ref<string>()
const selectedDrivers = ref<DriverVO[]>([])
const currentDriver = ref<DriverVO | null>(null)
const uploadDialogVisible = ref(false)
const detailsDialogVisible = ref(false)

// 分页数据
const currentPage = ref(1)
const pageSize = ref(20)

// ===== 计算属性 =====
const availableProtocols = computed(() => {
  const protocols = new Set<string>()
  driversStore.state.drivers.forEach(driver => {
    if (driver.info?.protocol) {
      protocols.add(driver.info.protocol)
    }
  })
  return Array.from(protocols).sort()
})

// ===== 方法 =====

/**
 * 获取驱动列表
 */
async function fetchDrivers() {
  await driversStore.fetchDrivers({
    page: currentPage.value,
    size: pageSize.value,
    name_contains: searchKeyword.value || undefined,
    status: selectedStatus.value,
    protocol: selectedProtocol.value,
  })
}

/**
 * 刷新驱动列表
 */
async function refreshDrivers() {
  await fetchDrivers()
  ElMessage.success('驱动列表已刷新')
}

/**
 * 处理搜索
 */
async function handleSearch() {
  currentPage.value = 1
  await fetchDrivers()
}

/**
 * 处理状态筛选
 */
async function handleStatusFilter() {
  currentPage.value = 1
  await fetchDrivers()
}

/**
 * 处理协议筛选
 */
async function handleProtocolFilter() {
  currentPage.value = 1
  await fetchDrivers()
}

/**
 * 重置筛选条件
 */
async function resetFilters() {
  searchKeyword.value = ''
  selectedStatus.value = undefined
  selectedProtocol.value = undefined
  currentPage.value = 1
  await fetchDrivers()
}

/**
 * 打开上传对话框
 */
function openUploadDialog() {
  uploadDialogVisible.value = true
}

/**
 * 重载所有驱动
 */
async function reloadAllDrivers() {
  await driversStore.reloadAllDrivers()
}

/**
 * 处理删除驱动
 */
async function handleDeleteDriver(driver: DriverVO) {
  await driversStore.deleteDriver(driver.id)
}

/**
 * 处理重载驱动
 */
async function handleReloadDriver(driver: DriverVO) {
  await driversStore.reloadDriver(driver.id)
}

/**
 * 处理查看详情
 */
function handleViewDetails(driver: DriverVO) {
  currentDriver.value = driver
  detailsDialogVisible.value = true
}

/**
 * 处理选择变更
 */
function handleSelectionChange(selection: DriverVO[]) {
  selectedDrivers.value = selection
}

/**
 * 清空选择
 */
function clearSelection() {
  selectedDrivers.value = []
}

/**
 * 处理批量删除
 */
async function handleBatchDelete() {
  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${selectedDrivers.value.length} 个驱动文件吗？删除后无法恢复。`,
      '批量删除确认',
      {
        type: 'warning',
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        confirmButtonClass: 'el-button--danger',
      }
    )

    for (const driver of selectedDrivers.value) {
      await driversStore.deleteDriver(driver.id)
    }

    selectedDrivers.value = []
    ElMessage.success('批量删除完成')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('批量删除失败')
    }
  }
}

/**
 * 处理批量重载
 */
async function handleBatchReload() {
  try {
    await ElMessageBox.confirm(
      `确定要重载选中的 ${selectedDrivers.value.length} 个驱动吗？`,
      '批量重载确认',
      {
        type: 'warning',
        confirmButtonText: '重载',
        cancelButtonText: '取消',
      }
    )

    let successCount = 0
    let failedCount = 0

    for (const driver of selectedDrivers.value) {
      const success = await driversStore.reloadDriver(driver.id)
      if (success) {
        successCount++
      } else {
        failedCount++
      }
    }

    selectedDrivers.value = []
    ElMessage.success(
      `批量重载完成：成功 ${successCount} 个，失败 ${failedCount} 个`
    )
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('批量重载失败')
    }
  }
}

/**
 * 处理上传成功
 */
async function handleUploadSuccess() {
  uploadDialogVisible.value = false
  await fetchDrivers()
}

/**
 * 处理分页大小变更
 */
async function handleSizeChange(newSize: number) {
  pageSize.value = newSize
  currentPage.value = 1
  await fetchDrivers()
}

/**
 * 处理当前页变更
 */
async function handleCurrentChange(newPage: number) {
  currentPage.value = newPage
  await fetchDrivers()
}

// ===== 生命周期 =====
onMounted(() => {
  fetchDrivers()
})

// 监听分页变化
watch([currentPage, pageSize], () => {
  fetchDrivers()
})
</script>

<style scoped lang="scss">
.drivers-page {
  padding: 24px;
  background-color: #f5f5f5;
  min-height: 100vh;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
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
      line-height: 1.4;
    }
  }

  .page-actions {
    display: flex;
    gap: 12px;
  }
}

.filter-card {
  margin-bottom: 16px;

  .filter-row {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .filter-item {
    display: flex;
    align-items: center;
  }

  .batch-actions {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid #ebeef5;
    display: flex;
    align-items: center;
    gap: 12px;

    .selected-info {
      color: #409eff;
      font-weight: 500;
    }
  }
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 16px;

  .stat-card {
    .el-card__body {
      padding: 20px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .stat-content {
      .stat-value {
        font-size: 28px;
        font-weight: 600;
        color: #303133;
        line-height: 1;
        margin-bottom: 8px;
      }

      .stat-label {
        font-size: 14px;
        color: #909399;
      }
    }

    .stat-icon {
      font-size: 32px;
      color: #409eff;
    }

    &.loaded .stat-icon {
      color: #67c23a;
    }

    &.failed .stat-icon {
      color: #f56c6c;
    }

    &.unloaded .stat-icon {
      color: #e6a23c;
    }
  }
}

.table-card {
  .pagination-wrapper {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .drivers-page {
    padding: 16px;
  }

  .page-header {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
  }

  .stats-row {
    grid-template-columns: repeat(2, 1fr);
  }

  .filter-row {
    flex-direction: column;
    align-items: stretch;

    .filter-item {
      width: 100%;

      :deep(.el-input),
      :deep(.el-select) {
        width: 100% !important;
      }
    }
  }
}
</style>
