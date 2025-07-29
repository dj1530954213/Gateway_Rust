<template>
  <div class="devices-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="page-title">
        <h1>设备管理</h1>
        <p class="page-description">管理物联网设备，包括添加、配置、监控和维护设备连接</p>
      </div>
      <div class="page-actions">
        <el-button type="primary" @click="openCreateDialog">
          <el-icon><Plus /></el-icon>
          新增设备
        </el-button>
        <el-button @click="refreshDevices">
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
            placeholder="搜索设备名称"
            clearable
            @input="handleSearch"
            style="width: 240px"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
        
        <div class="filter-item">
          <el-select
            v-model="selectedProtocol"
            placeholder="协议类型"
            clearable
            @change="handleProtocolFilter"
            style="width: 150px"
          >
            <el-option label="Modbus TCP" value="ModbusTcp" />
            <el-option label="Modbus RTU" value="ModbusRtu" />
            <el-option label="OPC UA" value="OpcUa" />
            <el-option label="MQTT" value="Mqtt" />
          </el-select>
        </div>
        
        <div class="filter-item">
          <el-select
            v-model="selectedStatus"
            placeholder="设备状态"
            clearable
            @change="handleStatusFilter"
            style="width: 120px"
          >
            <el-option label="已启用" :value="true" />
            <el-option label="已禁用" :value="false" />
          </el-select>
        </div>
        
        <div class="filter-item">
          <el-button @click="resetFilters">重置筛选</el-button>
        </div>
      </div>
      
      <!-- 批量操作栏 -->
      <div v-if="selectedDevices.length > 0" class="batch-actions">
        <span class="selected-info">已选择 {{ selectedDevices.length }} 个设备</span>
        <el-button type="danger" @click="handleBatchDelete">批量删除</el-button>
        <el-button @click="clearSelection">取消选择</el-button>
      </div>
    </el-card>

    <!-- 统计卡片 -->
    <div class="stats-row">
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ devicesStore.state.total }}</div>
          <div class="stat-label">总设备数</div>
        </div>
        <el-icon class="stat-icon"><Monitor /></el-icon>
      </el-card>
      
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ enabledDevicesCount }}</div>
          <div class="stat-label">已启用</div>
        </div>
        <el-icon class="stat-icon"><CircleCheck /></el-icon>
      </el-card>
      
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ disabledDevicesCount }}</div>
          <div class="stat-label">已禁用</div>
        </div>
        <el-icon class="stat-icon"><CircleClose /></el-icon>
      </el-card>
      
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ protocolTypesCount }}</div>
          <div class="stat-label">协议类型</div>
        </div>
        <el-icon class="stat-icon"><Setting /></el-icon>
      </el-card>
    </div>

    <!-- 设备列表 -->
    <el-card class="table-card" shadow="never">
      <DevicesTable
        :devices="devicesStore.state.devices"
        :loading="devicesStore.isLoading"
        :selected-devices="selectedDevices"
        @selection-change="handleSelectionChange"
        @edit-device="handleEditDevice"
        @delete-device="handleDeleteDevice"
        @toggle-device="handleToggleDevice"
        @test-connection="handleTestConnection"
      />
      
      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="devicesStore.state.total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 新增/编辑设备对话框 -->
    <DeviceFormDialog
      v-model:visible="formDialogVisible"
      :device="currentDevice"
      :mode="dialogMode"
      @success="handleFormSuccess"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * DevicesPage —— 设备管理主页面
 *
 * 📝 Responsibilities:
 *  1. 设备列表展示和分页
 *  2. 搜索和过滤功能
 *  3. 设备CRUD操作
 *  4. 批量操作支持
 *  5. 统计数据展示
 *
 * 📦 Dependencies:
 *  - DevicesTable 设备列表表格组件
 *  - DeviceFormDialog 设备表单对话框
 *  - useDevicesStore 设备状态管理
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ref, computed, onMounted, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Refresh, Search, Monitor, CircleCheck, CircleClose, Setting } from '@element-plus/icons-vue'
import { useDevicesStore } from '@/stores'
import DevicesTable from '@/components/devices/DevicesTable.vue'
import DeviceFormDialog from '@/components/devices/DeviceFormDialog.vue'
import type { DeviceVO } from '@/api/devices'

// ===== Store =====
const devicesStore = useDevicesStore()

// ===== 响应式数据 =====
const searchKeyword = ref('')
const selectedProtocol = ref<string>()
const selectedStatus = ref<boolean>()
const selectedDevices = ref<DeviceVO[]>([])
const currentDevice = ref<DeviceVO | null>(null)
const formDialogVisible = ref(false)
const dialogMode = ref<'create' | 'edit'>('create')

// 分页数据
const currentPage = ref(1)
const pageSize = ref(20)

// ===== 计算属性 =====
const enabledDevicesCount = computed(() => {
  return devicesStore.enabledDevices.length
})

const disabledDevicesCount = computed(() => {
  return devicesStore.disabledDevices.length
})

const protocolTypesCount = computed(() => {
  return Object.keys(devicesStore.devicesByProtocol).length
})

// ===== 方法 =====

/**
 * 获取设备列表
 */
async function fetchDevices() {
  await devicesStore.fetchDevices({
    page: currentPage.value,
    size: pageSize.value,
    name_contains: searchKeyword.value || undefined,
    protocol: selectedProtocol.value,
    enabled: selectedStatus.value,
  })
}

/**
 * 刷新设备列表
 */
async function refreshDevices() {
  await fetchDevices()
  ElMessage.success('设备列表已刷新')
}

/**
 * 处理搜索
 */
async function handleSearch() {
  currentPage.value = 1
  await fetchDevices()
}

/**
 * 处理协议筛选
 */
async function handleProtocolFilter() {
  currentPage.value = 1
  await fetchDevices()
}

/**
 * 处理状态筛选
 */
async function handleStatusFilter() {
  currentPage.value = 1
  await fetchDevices()
}

/**
 * 重置筛选条件
 */
async function resetFilters() {
  searchKeyword.value = ''
  selectedProtocol.value = undefined
  selectedStatus.value = undefined
  currentPage.value = 1
  await fetchDevices()
}

/**
 * 打开创建设备对话框
 */
function openCreateDialog() {
  currentDevice.value = null
  dialogMode.value = 'create'
  formDialogVisible.value = true
}

/**
 * 处理编辑设备
 */
function handleEditDevice(device: DeviceVO) {
  currentDevice.value = device
  dialogMode.value = 'edit'
  formDialogVisible.value = true
}

/**
 * 处理删除设备
 */
async function handleDeleteDevice(device: DeviceVO) {
  await devicesStore.deleteDevice(device.id)
}

/**
 * 处理启用/禁用设备
 */
async function handleToggleDevice(device: DeviceVO) {
  await devicesStore.toggleDevice(device.id, !device.enabled)
}

/**
 * 处理测试连接
 */
async function handleTestConnection(device: DeviceVO) {
  // 构造测试连接的请求数据
  const testData = {
    name: device.name,
    protocol: device.protocol,
    connection_config: device.connection_config,
    enabled: true,
  }
  
  await devicesStore.testDeviceConnection(testData)
}

/**
 * 处理选择变更
 */
function handleSelectionChange(selection: DeviceVO[]) {
  selectedDevices.value = selection
}

/**
 * 清空选择
 */
function clearSelection() {
  selectedDevices.value = []
}

/**
 * 处理批量删除
 */
async function handleBatchDelete() {
  const ids = selectedDevices.value.map(device => device.id)
  await devicesStore.batchDeleteDevices(ids)
  selectedDevices.value = []
}

/**
 * 处理表单提交成功
 */
async function handleFormSuccess() {
  formDialogVisible.value = false
  await fetchDevices()
}

/**
 * 处理分页大小变更
 */
async function handleSizeChange(newSize: number) {
  pageSize.value = newSize
  currentPage.value = 1
  await fetchDevices()
}

/**
 * 处理当前页变更
 */
async function handleCurrentChange(newPage: number) {
  currentPage.value = newPage
  await fetchDevices()
}

// ===== 生命周期 =====
onMounted(() => {
  fetchDevices()
})

// 监听分页变化
watch([currentPage, pageSize], () => {
  fetchDevices()
})
</script>

<style scoped lang="scss">
.devices-page {
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
  .devices-page {
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