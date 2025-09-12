<template>
  <div class="devices-view">
    <div class="page-header">
      <h1>设备管理</h1>
      <p>管理IoT设备的连接、配置和监控</p>
    </div>

    <div class="devices-content">
      <el-card v-loading="devicesStore.isLoading" class="devices-card">
        <template #header>
          <div class="card-header">
            <div class="header-left">
              <span>设备列表</span>
              <el-tag type="info" size="small">
                共 {{ devicesStore.pagination.total }} 台设备
              </el-tag>
            </div>
            <div class="header-actions">
              <el-button size="small" @click="handleRefresh">
                <el-icon><Refresh /></el-icon>
                刷新
              </el-button>
              <el-button type="primary" size="small" @click="handleAddDevice">
                <el-icon><Plus /></el-icon>
                添加设备
              </el-button>
            </div>
          </div>
        </template>

        <!-- 搜索和筛选 -->
        <div class="search-bar">
          <el-input
            v-model="searchKeyword"
            placeholder="搜索设备名称"
            size="small"
            style="width: 200px"
            clearable
            @keyup.enter="handleSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>

          <el-select
            v-model="protocolFilter"
            placeholder="协议类型"
            size="small"
            style="width: 120px"
            clearable
            @change="handleProtocolFilter"
          >
            <el-option label="Modbus TCP" value="ModbusTcp" />
            <el-option label="OPC UA" value="OpcUa" />
            <el-option label="MQTT" value="Mqtt" />
          </el-select>

          <el-select
            v-model="statusFilter"
            placeholder="设备状态"
            size="small"
            style="width: 100px"
            clearable
            @change="handleStatusFilter"
          >
            <el-option label="启用" :value="true" />
            <el-option label="禁用" :value="false" />
          </el-select>
        </div>

        <el-table :data="devicesStore.items" style="width: 100%">
          <el-table-column prop="name" label="设备名称" width="180" />
          <el-table-column prop="protocol" label="协议类型" width="120">
            <template #default="{ row }">
              <el-tag size="small" type="info">{{ row.protocol }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="endpoint" label="连接地址" />
          <el-table-column prop="enabled" label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="row.enabled ? 'success' : 'danger'" size="small">
                {{ row.enabled ? '启用' : '禁用' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="created_at" label="创建时间" width="180">
            <template #default="{ row }">
              {{ formatTime(row.created_at) }}
            </template>
          </el-table-column>
          <el-table-column label="操作" fixed="right" width="200">
            <template #default="{ row }">
              <el-button
                size="small"
                type="primary"
                @click="handleEditDevice(row)"
              >
                编辑
              </el-button>
              <el-button
                size="small"
                :type="row.enabled ? 'warning' : 'success'"
                @click="handleToggleDevice(row)"
              >
                {{ row.enabled ? '禁用' : '启用' }}
              </el-button>
              <el-button
                size="small"
                type="danger"
                @click="handleDeleteDevice(row)"
              >
                删除
              </el-button>
            </template>
          </el-table-column>
        </el-table>

        <!-- 分页 -->
        <div v-if="devicesStore.pagination.total > 0" class="pagination-wrapper">
          <el-pagination
            :current-page="devicesStore.pagination.page"
            :page-size="devicesStore.pagination.size"
            :page-sizes="[10, 20, 50, 100]"
            :total="devicesStore.pagination.total"
            layout="total, sizes, prev, pager, next, jumper"
            @size-change="handleSizeChange"
            @current-change="handleCurrentChange"
          />
        </div>

        <!-- 空状态 -->
        <el-empty
          v-if="!devicesStore.isLoading && devicesStore.items.length === 0"
          description="暂无设备数据"
          :image-size="80"
        />
      </el-card>
    </div>

    <!-- 设备表单对话框 -->
    <DeviceFormDialog
      v-model:visible="showDeviceDialog"
      :device="editDevice"
      :mode="editDevice ? 'edit' : 'create'"
      @success="handleDeviceSuccess"
    />
  </div>
</template>

<script setup lang="ts">
import { Plus, Refresh, Search } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'

import type { DeviceVO } from '@/api/devices'
import DeviceFormDialog from '@/components/devices/DeviceFormDialog.vue'
import { useDevicesStore } from '@/stores'
import { formatTime } from '@/utils/date'

const router = useRouter()
const devicesStore = useDevicesStore()

// 搜索和筛选状态
const searchKeyword = ref('')
const protocolFilter = ref<string>()
const statusFilter = ref<boolean>()

// 对话框状态
const showDeviceDialog = ref(false)
const editDevice = ref<DeviceVO | null>(null)

// 方法
const handleRefresh = () => {
  devicesStore.refresh()
}

const handleAddDevice = () => {
  editDevice.value = null
  showDeviceDialog.value = true
}

const handleEditDevice = (device: DeviceVO) => {
  editDevice.value = device
  showDeviceDialog.value = true
}

const handleDeleteDevice = async (device: DeviceVO) => {
  try {
    await devicesStore.deleteDevice(device.id)
  } catch (error) {
    console.error('删除设备失败:', error)
  }
}

const handleToggleDevice = async (device: DeviceVO) => {
  try {
    await devicesStore.toggleDevice(device.id, !device.enabled)
  } catch (error) {
    console.error('切换设备状态失败:', error)
  }
}

const handleSearch = () => {
  devicesStore.searchDevices(searchKeyword.value)
}

const handleProtocolFilter = (protocol: string | undefined) => {
  devicesStore.filterByProtocol(protocol as any)
}

const handleStatusFilter = (enabled: boolean | undefined) => {
  devicesStore.filterByStatus(enabled)
}

const handleSizeChange = (size: number) => {
  devicesStore.changePageSize(size)
}

const handleCurrentChange = (page: number) => {
  devicesStore.changePage(page)
}

const handleDeviceSuccess = () => {
  showDeviceDialog.value = false
  devicesStore.fetchDevices()
}

// 初始化
onMounted(async () => {
  await devicesStore.fetchDevices()
})
</script>

<style scoped lang="scss">
.devices-view {
  padding: 20px;
}

.page-header {
  margin-bottom: 20px;

  h1 {
    margin: 0 0 8px 0;
    font-size: 24px;
    color: var(--el-text-color-primary);
  }

  p {
    margin: 0;
    color: var(--el-text-color-secondary);
  }
}

.devices-content {
  .devices-card {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .header-left {
        display: flex;
        align-items: center;
        gap: 12px;
      }

      .header-actions {
        display: flex;
        gap: 8px;
      }
    }

    .search-bar {
      display: flex;
      gap: 12px;
      margin-bottom: 16px;
      padding: 16px 0;
      border-bottom: 1px solid var(--el-border-color-lighter);
    }

    .pagination-wrapper {
      display: flex;
      justify-content: center;
      margin-top: 20px;
      padding-top: 16px;
      border-top: 1px solid var(--el-border-color-lighter);
    }
  }
}

@media (max-width: 768px) {
  .devices-view {
    padding: 12px;

    .devices-content .devices-card {
      .card-header {
        flex-direction: column;
        gap: 12px;
        align-items: stretch;

        .header-left,
        .header-actions {
          justify-content: center;
        }
      }

      .search-bar {
        flex-direction: column;
        gap: 8px;
      }
    }
  }
}
</style>
