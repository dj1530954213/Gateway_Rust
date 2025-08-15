<template>
  <div class="driver-configs-list-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-title">
        <el-icon :size="24">
          <Setting />
        </el-icon>
        <h1>驱动配置管理</h1>
        <el-tag v-if="hasConfigs" type="info" size="large">
          {{ configsStore.state.total }} 个配置
        </el-tag>
      </div>

      <div class="header-actions">
        <el-button type="primary" :icon="Plus" @click="handleAddConfig">
          新增配置
        </el-button>

        <el-dropdown trigger="click" @command="handleHeaderAction">
          <el-button :icon="MoreFilled" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="refresh" :icon="Refresh">
                刷新数据
              </el-dropdown-item>
              <el-dropdown-item command="export" :icon="Download">
                导出配置
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 配置统计概览 -->
    <div class="configs-overview">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--total">
            <div class="stat-content">
              <div class="stat-number">{{ configsStore.state.total }}</div>
              <div class="stat-label">总配置数</div>
            </div>
            <el-icon class="stat-icon">
              <Monitor />
            </el-icon>
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--enabled">
            <div class="stat-content">
              <div class="stat-number">{{ enabledConfigs.length }}</div>
              <div class="stat-label">已启用</div>
            </div>
            <el-icon class="stat-icon">
              <VideoPlay />
            </el-icon>
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--disabled">
            <div class="stat-content">
              <div class="stat-number">{{ disabledConfigs.length }}</div>
              <div class="stat-label">已禁用</div>
            </div>
            <el-icon class="stat-icon">
              <VideoPause />
            </el-icon>
          </el-card>
        </el-col>

        <el-col :span="6">
          <el-card shadow="never" class="stat-card stat-card--protocols">
            <div class="stat-content">
              <div class="stat-number">
                {{ Object.keys(configsByProtocol).length }}
              </div>
              <div class="stat-label">协议类型</div>
            </div>
            <el-icon class="stat-icon">
              <Connection />
            </el-icon>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 筛选和搜索 -->
    <div class="configs-filters">
      <el-card shadow="never">
        <div class="filter-content">
          <div class="filter-left">
            <el-input
              v-model="searchKeyword"
              placeholder="搜索配置名称、协议类型或描述"
              style="width: 300px"
              :prefix-icon="Search"
              clearable
              @input="handleSearch"
              @clear="handleSearch('')"
            />

            <el-select
              v-model="protocolFilter"
              placeholder="协议类型"
              clearable
              style="width: 150px; margin-left: 12px"
              @change="handleProtocolFilter"
            >
              <el-option
                v-for="protocol in availableProtocols"
                :key="protocol"
                :label="protocol"
                :value="protocol"
              />
            </el-select>

            <el-select
              v-model="statusFilter"
              placeholder="状态筛选"
              clearable
              style="width: 120px; margin-left: 12px"
              @change="handleStatusFilter"
            >
              <el-option label="已启用" :value="true" />
              <el-option label="已禁用" :value="false" />
            </el-select>
          </div>

          <div class="filter-right">
            <el-button :icon="RefreshRight" @click="handleResetFilters">
              重置
            </el-button>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 配置列表表格 -->
    <div class="configs-table">
      <el-table
        v-loading="configsStore.isLoading"
        :data="configsStore.state.configs"
        style="width: 100%"
        :row-key="row => row.id"
        @selection-change="handleSelectionChange"
      >
        <el-table-column type="selection" width="55" />

        <el-table-column
          prop="name"
          label="配置名称"
          width="200"
          show-overflow-tooltip
        >
          <template #default="{ row }">
            <div class="config-name">
              <el-icon class="config-icon">
                <Connection />
              </el-icon>
              <span>{{ row.name }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="protocol" label="协议类型" width="120">
          <template #default="{ row }">
            <el-tag :type="getProtocolTagType(row.protocol)" size="small">
              {{ row.protocol }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="connection_type" label="连接类型" width="100">
          <template #default="{ row }">
            <el-tag type="info" size="small">
              {{ row.connection_type }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="enabled" label="状态" width="80" align="center">
          <template #default="{ row }">
            <el-switch
              v-model="row.enabled"
              :loading="loadingConfigId === row.id"
              @change="handleToggleStatus(row)"
            />
          </template>
        </el-table-column>

        <el-table-column prop="description" label="描述" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.description || '暂无描述' }}
          </template>
        </el-table-column>

        <el-table-column label="连接信息" width="200">
          <template #default="{ row }">
            <div v-if="row.config" class="connection-info">
              <div class="info-item">
                <span class="info-label">地址:</span>
                <span class="info-value">{{ row.config.host || 'N/A' }}</span>
              </div>
              <div v-if="row.config.port" class="info-item">
                <span class="info-label">端口:</span>
                <span class="info-value">{{ row.config.port }}</span>
              </div>
              <div v-if="row.config.slave_id" class="info-item">
                <span class="info-label">从机ID:</span>
                <span class="info-value">{{ row.config.slave_id }}</span>
              </div>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="性能设置" width="150">
          <template #default="{ row }">
            <div class="performance-info">
              <div class="info-item">
                <span class="info-label">扫描间隔:</span>
                <span class="info-value">{{ row.scan_interval }}ms</span>
              </div>
              <div class="info-item">
                <span class="info-label">超时:</span>
                <span class="info-value">{{ row.timeout }}ms</span>
              </div>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="created_at" label="创建时间" width="160">
          <template #default="{ row }">
            {{ formatTime(row.created_at) }}
          </template>
        </el-table-column>

        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <div class="table-actions">
              <el-button
                type="primary"
                size="small"
                :icon="View"
                @click="handleViewConfig(row)"
              >
                详情
              </el-button>

              <el-button
                type="default"
                size="small"
                :icon="Edit"
                @click="handleEditConfig(row)"
              >
                编辑
              </el-button>

              <el-button
                type="danger"
                size="small"
                :icon="Delete"
                @click="handleDeleteConfig(row)"
              >
                删除
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="configsStore.state.total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </div>

    <!-- 配置详情抽屉 -->
    <el-drawer
      v-model="detailDrawerVisible"
      :title="`配置详情 - ${selectedConfig?.name}`"
      size="700px"
      :before-close="handleDetailDrawerClose"
    >
      <div v-if="selectedConfig" class="config-detail">
        <el-descriptions border :column="2">
          <el-descriptions-item label="配置名称">
            {{ selectedConfig.name }}
          </el-descriptions-item>
          <el-descriptions-item label="协议类型">
            <el-tag :type="getProtocolTagType(selectedConfig.protocol)">
              {{ selectedConfig.protocol }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="连接类型">
            <el-tag type="info">{{ selectedConfig.connection_type }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="启用状态">
            <el-tag :type="selectedConfig.enabled ? 'success' : 'danger'">
              {{ selectedConfig.enabled ? '已启用' : '已禁用' }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="扫描间隔" span="2">
            {{ selectedConfig.scan_interval }}ms
          </el-descriptions-item>
          <el-descriptions-item label="连接超时">
            {{ selectedConfig.timeout }}ms
          </el-descriptions-item>
          <el-descriptions-item label="最大并发">
            {{ selectedConfig.max_concurrent }}
          </el-descriptions-item>
          <el-descriptions-item label="批量大小">
            {{ selectedConfig.batch_size }}
          </el-descriptions-item>
          <el-descriptions-item label="最大重试">
            {{ selectedConfig.max_retries }}
          </el-descriptions-item>
          <el-descriptions-item label="重试间隔">
            {{ selectedConfig.retry_interval }}ms
          </el-descriptions-item>
          <el-descriptions-item label="指数退避">
            <el-tag
              :type="selectedConfig.exponential_backoff ? 'success' : 'info'"
              size="small"
            >
              {{ selectedConfig.exponential_backoff ? '启用' : '禁用' }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">
            {{ formatTime(selectedConfig.created_at) }}
          </el-descriptions-item>
          <el-descriptions-item label="更新时间">
            {{ formatTime(selectedConfig.updated_at) }}
          </el-descriptions-item>
          <el-descriptions-item label="配置描述" span="2">
            {{ selectedConfig.description || '暂无描述' }}
          </el-descriptions-item>
        </el-descriptions>

        <!-- 连接配置详情 -->
        <div style="margin-top: 20px">
          <h4>连接配置</h4>
          <el-card shadow="never">
            <pre>{{ JSON.stringify(selectedConfig.config, null, 2) }}</pre>
          </el-card>
        </div>

        <!-- 数据点列表 -->
        <div v-if="selectedConfig.config?.data_points" style="margin-top: 20px">
          <h4>数据点配置</h4>
          <el-table :data="selectedConfig.config.data_points" size="small">
            <el-table-column prop="name" label="名称" />
            <el-table-column prop="address" label="地址" />
            <el-table-column prop="data_type" label="数据类型" />
            <el-table-column prop="unit" label="单位" />
            <el-table-column prop="function_code" label="功能码" />
          </el-table>
        </div>
      </div>

      <template #footer>
        <div class="drawer-footer">
          <el-button @click="detailDrawerVisible = false">关闭</el-button>
          <el-button type="primary" @click="handleEditConfig(selectedConfig!)">
            编辑配置
          </el-button>
        </div>
      </template>
    </el-drawer>

    <!-- 配置表单对话框 -->
    <el-dialog
      v-model="formDialogVisible"
      :title="formTitle"
      width="800px"
      :before-close="handleFormDialogClose"
    >
      <div class="config-form">
        <el-form
          ref="configFormRef"
          :model="configForm"
          :rules="configFormRules"
          label-width="120px"
        >
          <el-form-item label="配置名称" prop="name">
            <el-input v-model="configForm.name" placeholder="请输入配置名称" />
          </el-form-item>

          <el-form-item label="协议类型" prop="protocol">
            <el-select
              v-model="configForm.protocol"
              placeholder="选择协议类型"
              style="width: 100%"
            >
              <el-option label="Modbus TCP" value="modbus_tcp" />
              <el-option label="Modbus RTU" value="modbus_rtu" />
              <el-option label="OPC UA" value="opcua" />
              <el-option label="MQTT5" value="mqtt5" />
            </el-select>
          </el-form-item>

          <el-form-item label="连接类型" prop="connection_type">
            <el-select
              v-model="configForm.connection_type"
              placeholder="选择连接类型"
              style="width: 100%"
            >
              <el-option label="以太网" value="ethernet" />
              <el-option label="串口" value="serial" />
              <el-option label="USB" value="usb" />
            </el-select>
          </el-form-item>

          <el-form-item label="启用状态">
            <el-switch v-model="configForm.enabled" />
          </el-form-item>

          <el-form-item label="配置描述">
            <el-input
              v-model="configForm.description"
              type="textarea"
              :rows="2"
              placeholder="请输入配置描述"
            />
          </el-form-item>

          <!-- 性能设置 -->
          <el-divider content-position="left">性能设置</el-divider>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="扫描间隔(ms)" prop="scan_interval">
                <el-input-number
                  v-model="configForm.scan_interval"
                  :min="100"
                  :max="60000"
                  style="width: 100%"
                />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="连接超时(ms)" prop="timeout">
                <el-input-number
                  v-model="configForm.timeout"
                  :min="1000"
                  :max="30000"
                  style="width: 100%"
                />
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="最大并发" prop="max_concurrent">
                <el-input-number
                  v-model="configForm.max_concurrent"
                  :min="1"
                  :max="50"
                  style="width: 100%"
                />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="批量大小" prop="batch_size">
                <el-input-number
                  v-model="configForm.batch_size"
                  :min="1"
                  :max="100"
                  style="width: 100%"
                />
              </el-form-item>
            </el-col>
          </el-row>

          <!-- 连接配置JSON -->
          <el-divider content-position="left">连接配置</el-divider>

          <el-form-item label="连接配置" prop="config">
            <el-input
              v-model="configJsonString"
              type="textarea"
              :rows="8"
              placeholder="请输入JSON格式的连接配置"
            />
            <div class="form-tip">
              请输入有效的JSON格式配置，例如：{"host": "192.168.1.100", "port":
              502, "slave_id": 1}
            </div>
          </el-form-item>
        </el-form>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="handleFormDialogClose">取消</el-button>
          <el-button
            type="primary"
            :loading="
              configsStore.state.creating || configsStore.state.updating
            "
            @click="handleSaveConfig"
          >
            保存
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  Setting,
  Plus,
  MoreFilled,
  Refresh,
  Download,
  Monitor,
  VideoPlay,
  VideoPause,
  Connection,
  Search,
  RefreshRight,
  View,
  Edit,
  Delete,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, onMounted, watch, type Ref } from 'vue'

// 导入 Store
import type {
  DriverConfigVO,
  DriverConfigCreateReq,
  DriverConfigUpdateReq,
} from '@/api/drivers'
import { useDriverConfigsStore } from '@/stores/driver-configs'

// 使用配置 Store
const configsStore = useDriverConfigsStore()

// 搜索和筛选状态
const searchKeyword = ref('')
const protocolFilter = ref<string>('')
const statusFilter = ref<boolean | ''>('')
const loadingConfigId = ref<string>('')

// 对话框状态
const detailDrawerVisible = ref(false)
const formDialogVisible = ref(false)
const selectedConfig = ref<DriverConfigVO | null>(null)
const editingConfig = ref<DriverConfigVO | null>(null)

// 分页状态
const currentPage = ref(1)
const pageSize = ref(20)

// 表单相关
const configFormRef = ref()
const configForm = ref<Partial<DriverConfigCreateReq>>({
  name: '',
  description: '',
  protocol: 'modbus_tcp',
  connection_type: 'ethernet',
  enabled: true,
  scan_interval: 1000,
  timeout: 3000,
  max_concurrent: 5,
  batch_size: 10,
  config: {},
})

const configJsonString = ref('')

// 计算属性
const hasConfigs = computed(() => configsStore.hasConfigs)
const enabledConfigs = computed(() => configsStore.enabledConfigs)
const disabledConfigs = computed(() => configsStore.disabledConfigs)
const configsByProtocol = computed(() => configsStore.configsByProtocol)

const availableProtocols = computed(() => {
  const protocols = new Set<string>()
  configsStore.state.configs.forEach(config => {
    if (config.protocol) {
      protocols.add(config.protocol)
    }
  })
  return Array.from(protocols).sort()
})

const formTitle = computed(() => {
  return editingConfig.value ? '编辑驱动配置' : '新增驱动配置'
})

// 表单验证规则
const configFormRules = {
  name: [{ required: true, message: '请输入配置名称', trigger: 'blur' }],
  protocol: [{ required: true, message: '请选择协议类型', trigger: 'change' }],
  connection_type: [
    { required: true, message: '请选择连接类型', trigger: 'change' },
  ],
  scan_interval: [
    { required: true, message: '请设置扫描间隔', trigger: 'blur' },
  ],
  timeout: [{ required: true, message: '请设置连接超时', trigger: 'blur' }],
}

// 监听JSON字符串变化
watch(configJsonString, newValue => {
  try {
    if (newValue.trim()) {
      configForm.value.config = JSON.parse(newValue)
    }
  } catch (error) {
    // JSON格式错误时不更新
  }
})

// 事件处理方法
const handleSearch = (keyword: string) => {
  configsStore.searchConfigs(keyword)
}

const handleProtocolFilter = (protocol: string) => {
  configsStore.filterByProtocol(protocol || undefined)
}

const handleStatusFilter = (enabled: boolean | '') => {
  if (enabled === '') {
    configsStore.filterByEnabled(undefined)
  } else {
    configsStore.filterByEnabled(enabled)
  }
}

const handleResetFilters = () => {
  searchKeyword.value = ''
  protocolFilter.value = ''
  statusFilter.value = ''
  configsStore.resetSearch()
}

const handleSelectionChange = (selection: DriverConfigVO[]) => {
  // 处理选择变更
}

const handleToggleStatus = async (config: DriverConfigVO) => {
  loadingConfigId.value = config.id
  try {
    await configsStore.toggleConfigStatus(config.id, config.enabled)
  } finally {
    loadingConfigId.value = ''
  }
}

const handleViewConfig = (config: DriverConfigVO) => {
  selectedConfig.value = config
  detailDrawerVisible.value = true
}

const handleEditConfig = (config: DriverConfigVO) => {
  editingConfig.value = config

  // 填充表单
  configForm.value = {
    name: config.name,
    description: config.description,
    protocol: config.protocol,
    connection_type: config.connection_type,
    enabled: config.enabled,
    scan_interval: config.scan_interval,
    timeout: config.timeout,
    max_concurrent: config.max_concurrent,
    batch_size: config.batch_size,
    config: config.config,
  }

  configJsonString.value = JSON.stringify(config.config || {}, null, 2)
  formDialogVisible.value = true
}

const handleDeleteConfig = async (config: DriverConfigVO) => {
  await configsStore.deleteConfig(config.id)
}

const handleAddConfig = () => {
  editingConfig.value = null

  // 重置表单
  configForm.value = {
    name: '',
    description: '',
    protocol: 'modbus_tcp',
    connection_type: 'ethernet',
    enabled: true,
    scan_interval: 1000,
    timeout: 3000,
    max_concurrent: 5,
    batch_size: 10,
    config: {},
  }

  configJsonString.value = '{}'
  formDialogVisible.value = true
}

const handleSaveConfig = async () => {
  try {
    await configFormRef.value?.validate()

    // 验证JSON格式
    try {
      if (configJsonString.value.trim()) {
        configForm.value.config = JSON.parse(configJsonString.value)
      }
    } catch (error) {
      ElMessage.error('连接配置JSON格式错误')
      return
    }

    let success = false

    if (editingConfig.value) {
      // 更新配置
      const updateData: DriverConfigUpdateReq = { ...configForm.value }
      success = await configsStore.updateConfig(
        editingConfig.value.id,
        updateData
      )
    } else {
      // 创建配置
      const createData: DriverConfigCreateReq =
        configForm.value as DriverConfigCreateReq
      success = await configsStore.createConfig(createData)
    }

    if (success) {
      formDialogVisible.value = false
    }
  } catch (error) {
    console.error('保存配置失败:', error)
  }
}

const handleHeaderAction = (command: string) => {
  switch (command) {
    case 'refresh':
      configsStore.refresh()
      break
    case 'export':
      handleExportConfigs()
      break
  }
}

const handleExportConfigs = () => {
  const data = {
    driver_configs: configsStore.state.configs,
    export_time: new Date().toISOString(),
    total: configsStore.state.total,
  }

  const blob = new Blob([JSON.stringify(data, null, 2)], {
    type: 'application/json',
  })

  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `driver_configs_${new Date().toISOString().split('T')[0]}.json`
  a.click()

  URL.revokeObjectURL(url)
  ElMessage.success('配置导出成功')
}

const handleCurrentChange = (page: number) => {
  configsStore.changePage(page)
}

const handleSizeChange = (size: number) => {
  configsStore.changePageSize(size)
}

const handleDetailDrawerClose = () => {
  detailDrawerVisible.value = false
  selectedConfig.value = null
}

const handleFormDialogClose = () => {
  formDialogVisible.value = false
  editingConfig.value = null
}

// 辅助方法
const getProtocolTagType = (protocol: string) => {
  const typeMap: Record<string, string> = {
    modbus_tcp: 'primary',
    modbus_rtu: 'success',
    opcua: 'warning',
    mqtt5: 'info',
  }
  return typeMap[protocol] || 'default'
}

const formatTime = (timeStr: string) => {
  if (!timeStr) return ''
  return new Date(timeStr).toLocaleString('zh-CN')
}

// 生命周期
onMounted(async () => {
  await configsStore.fetchConfigs()
})
</script>

<style scoped lang="scss">
.driver-configs-list-page {
  padding: 16px;
  background: var(--el-bg-color-page);
  min-height: 100vh;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;

  .header-title {
    display: flex;
    align-items: center;
    gap: 12px;

    h1 {
      margin: 0;
      font-size: 24px;
      font-weight: 600;
      color: var(--el-text-color-primary);
    }
  }

  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.configs-overview {
  margin-bottom: 20px;

  .stat-card {
    position: relative;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.3s;

    &:hover {
      transform: translateY(-2px);
      box-shadow: var(--el-box-shadow);
    }

    .stat-content {
      position: relative;
      z-index: 2;

      .stat-number {
        font-size: 32px;
        font-weight: bold;
        margin-bottom: 4px;
      }

      .stat-label {
        font-size: 14px;
        color: var(--el-text-color-secondary);
      }
    }

    .stat-icon {
      position: absolute;
      right: 16px;
      top: 50%;
      transform: translateY(-50%);
      font-size: 48px;
      opacity: 0.1;
    }

    &.stat-card--total .stat-number,
    &.stat-card--total .stat-icon {
      color: var(--el-color-primary);
    }

    &.stat-card--enabled .stat-number,
    &.stat-card--enabled .stat-icon {
      color: var(--el-color-success);
    }

    &.stat-card--disabled .stat-number,
    &.stat-card--disabled .stat-icon {
      color: var(--el-color-info);
    }

    &.stat-card--protocols .stat-number,
    &.stat-card--protocols .stat-icon {
      color: var(--el-color-warning);
    }
  }
}

.configs-filters {
  margin-bottom: 20px;

  .filter-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;

    .filter-left {
      display: flex;
      align-items: center;
    }
  }
}

.configs-table {
  background: var(--el-bg-color);
  border-radius: 8px;
  overflow: hidden;

  .config-name {
    display: flex;
    align-items: center;
    gap: 8px;

    .config-icon {
      color: var(--el-color-primary);
    }
  }

  .connection-info,
  .performance-info {
    font-size: 12px;

    .info-item {
      display: flex;
      justify-content: space-between;
      margin-bottom: 2px;

      .info-label {
        color: var(--el-text-color-secondary);
      }

      .info-value {
        font-weight: 500;
      }
    }
  }

  .table-actions {
    display: flex;
    gap: 4px;
  }

  .pagination-wrapper {
    padding: 16px;
    display: flex;
    justify-content: flex-end;
  }
}

.config-detail {
  .el-descriptions {
    margin-bottom: 20px;
  }

  pre {
    background: var(--el-fill-color-lighter);
    padding: 12px;
    border-radius: 4px;
    font-size: 12px;
    overflow-x: auto;
  }
}

.drawer-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.config-form {
  .form-tip {
    font-size: 12px;
    color: var(--el-text-color-secondary);
    margin-top: 4px;
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

// 响应式设计
@media (max-width: 768px) {
  .driver-configs-list-page {
    padding: 8px;
  }

  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .filter-content {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;

    .filter-left,
    .filter-right {
      width: 100%;
    }
  }
}
</style>
