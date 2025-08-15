<template>
  <div class="system-backup">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1>系统备份与恢复</h1>
          <p class="description">管理系统数据备份，确保数据安全和快速恢复</p>
        </div>
        <div class="header-actions">
          <el-button type="primary" @click="openCreateBackupDialog">
            <el-icon><Plus /></el-icon>
            创建备份
          </el-button>
          <el-button @click="openScheduleDialog">
            <el-icon><Timer /></el-icon>
            计划任务
          </el-button>
        </div>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ backupStats.totalBackups }}</div>
          <div class="stat-label">总备份数</div>
          <div class="stat-icon backup">
            <el-icon><Document /></el-icon>
          </div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">
            {{ formatFileSize(backupStats.totalSize) }}
          </div>
          <div class="stat-label">总大小</div>
          <div class="stat-icon size">
            <el-icon><Folder /></el-icon>
          </div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ backupStats.successRate }}%</div>
          <div class="stat-label">成功率</div>
          <div class="stat-icon success">
            <el-icon><SuccessFilled /></el-icon>
          </div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ backupStats.scheduledTasks }}</div>
          <div class="stat-label">定时任务</div>
          <div class="stat-icon scheduled">
            <el-icon><Clock /></el-icon>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 主要内容区域 -->
    <div class="main-content">
      <!-- 左侧备份列表 -->
      <div class="backup-list-section">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>备份列表</span>
              <div class="header-actions">
                <el-input
                  v-model="searchQuery"
                  placeholder="搜索备份..."
                  size="small"
                  style="width: 200px"
                  clearable
                >
                  <template #prefix>
                    <el-icon><Search /></el-icon>
                  </template>
                </el-input>
                <el-select
                  v-model="selectedType"
                  placeholder="备份类型"
                  size="small"
                  style="width: 120px"
                  clearable
                >
                  <el-option label="全部" value="" />
                  <el-option label="完整备份" value="full" />
                  <el-option label="配置备份" value="config" />
                  <el-option label="数据备份" value="data" />
                  <el-option label="增量备份" value="incremental" />
                </el-select>
              </div>
            </div>
          </template>

          <div v-loading="loading" class="backup-list">
            <div
              v-for="backup in filteredBackups"
              :key="backup.id"
              class="backup-item"
              :class="{ active: selectedBackup?.id === backup.id }"
              @click="selectBackup(backup)"
            >
              <div class="backup-info">
                <div class="backup-name">{{ backup.name }}</div>
                <div class="backup-meta">
                  <span class="backup-type" :class="backup.type">{{
                    getTypeLabel(backup.type)
                  }}</span>
                  <span class="backup-size">{{
                    formatFileSize(backup.size)
                  }}</span>
                </div>
                <div class="backup-date">
                  {{ formatDate(backup.createdAt) }}
                </div>
              </div>
              <div class="backup-status">
                <el-tag :type="getStatusType(backup.status)" size="small">
                  {{ getStatusLabel(backup.status) }}
                </el-tag>
              </div>
              <div class="backup-actions">
                <el-dropdown trigger="click">
                  <el-button text>
                    <el-icon><MoreFilled /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item @click="restoreBackup(backup)">
                        <el-icon><RefreshRight /></el-icon>
                        恢复
                      </el-dropdown-item>
                      <el-dropdown-item @click="downloadBackup(backup)">
                        <el-icon><Download /></el-icon>
                        下载
                      </el-dropdown-item>
                      <el-dropdown-item @click="verifyBackup(backup)">
                        <el-icon><CircleCheck /></el-icon>
                        验证
                      </el-dropdown-item>
                      <el-dropdown-item divided @click="deleteBackup(backup)">
                        <el-icon><Delete /></el-icon>
                        删除
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>
            </div>

            <div v-if="filteredBackups.length === 0" class="empty-state">
              <el-icon><Document /></el-icon>
              <p>{{ searchQuery ? '未找到匹配的备份' : '暂无备份记录' }}</p>
            </div>
          </div>
        </el-card>
      </div>

      <!-- 右侧详情面板 -->
      <div class="backup-detail-section">
        <el-card v-if="selectedBackup">
          <template #header>
            <div class="card-header">
              <span>备份详情</span>
              <div class="header-actions">
                <el-button
                  type="primary"
                  size="small"
                  @click="restoreBackup(selectedBackup)"
                >
                  <el-icon><RefreshRight /></el-icon>
                  恢复
                </el-button>
              </div>
            </div>
          </template>

          <div class="backup-details">
            <!-- 基本信息 -->
            <div class="detail-section">
              <h3>基本信息</h3>
              <div class="info-grid">
                <div class="info-item">
                  <label>备份名称:</label>
                  <span>{{ selectedBackup.name }}</span>
                </div>
                <div class="info-item">
                  <label>备份类型:</label>
                  <span class="backup-type" :class="selectedBackup.type">
                    {{ getTypeLabel(selectedBackup.type) }}
                  </span>
                </div>
                <div class="info-item">
                  <label>文件大小:</label>
                  <span>{{ formatFileSize(selectedBackup.size) }}</span>
                </div>
                <div class="info-item">
                  <label>创建时间:</label>
                  <span>{{ formatDate(selectedBackup.createdAt) }}</span>
                </div>
                <div class="info-item">
                  <label>备份状态:</label>
                  <el-tag :type="getStatusType(selectedBackup.status)">
                    {{ getStatusLabel(selectedBackup.status) }}
                  </el-tag>
                </div>
                <div class="info-item">
                  <label>压缩率:</label>
                  <span>{{ selectedBackup.compressionRatio }}%</span>
                </div>
              </div>
            </div>

            <!-- 备份内容 -->
            <div class="detail-section">
              <h3>备份内容</h3>
              <div class="content-list">
                <div
                  v-for="item in selectedBackup.contents"
                  :key="item.name"
                  class="content-item"
                >
                  <el-icon><FolderOpened /></el-icon>
                  <span class="item-name">{{ item.name }}</span>
                  <span class="item-size">{{ formatFileSize(item.size) }}</span>
                </div>
              </div>
            </div>

            <!-- 验证信息 -->
            <div v-if="selectedBackup.verification" class="detail-section">
              <h3>验证信息</h3>
              <div class="verification-info">
                <div class="verification-item">
                  <label>MD5校验:</label>
                  <span class="checksum">{{
                    selectedBackup.verification.md5
                  }}</span>
                </div>
                <div class="verification-item">
                  <label>完整性验证:</label>
                  <el-tag
                    :type="
                      selectedBackup.verification.integrity
                        ? 'success'
                        : 'danger'
                    "
                    size="small"
                  >
                    {{
                      selectedBackup.verification.integrity ? '通过' : '失败'
                    }}
                  </el-tag>
                </div>
                <div class="verification-item">
                  <label>最后验证时间:</label>
                  <span>{{
                    formatDate(selectedBackup.verification.lastVerified)
                  }}</span>
                </div>
              </div>
            </div>
          </div>
        </el-card>

        <!-- 进度监控 -->
        <el-card v-if="activeOperations.length > 0" class="progress-card">
          <template #header>
            <span>进度监控</span>
          </template>
          <div class="operations-list">
            <div
              v-for="operation in activeOperations"
              :key="operation.id"
              class="operation-item"
            >
              <div class="operation-info">
                <div class="operation-name">{{ operation.name }}</div>
                <div class="operation-type">{{ operation.type }}</div>
              </div>
              <div class="operation-progress">
                <el-progress
                  :percentage="operation.progress"
                  :status="
                    operation.status === 'error' ? 'exception' : undefined
                  "
                />
                <div class="progress-text">
                  {{ operation.currentStep }} - {{ operation.progress }}%
                </div>
              </div>
              <div class="operation-actions">
                <el-button
                  v-if="operation.status === 'running'"
                  text
                  @click="cancelOperation(operation.id)"
                >
                  取消
                </el-button>
              </div>
            </div>
          </div>
        </el-card>
      </div>
    </div>

    <!-- 创建备份对话框 -->
    <el-dialog
      v-model="createBackupVisible"
      title="创建备份"
      width="600px"
      :before-close="closeCreateBackupDialog"
    >
      <el-form
        ref="createBackupForm"
        :model="createBackupData"
        :rules="createBackupRules"
        label-width="100px"
      >
        <el-form-item label="备份名称" prop="name">
          <el-input
            v-model="createBackupData.name"
            placeholder="请输入备份名称"
            clearable
          />
        </el-form-item>

        <el-form-item label="备份类型" prop="type">
          <el-radio-group v-model="createBackupData.type">
            <el-radio label="full">完整备份</el-radio>
            <el-radio label="config">配置备份</el-radio>
            <el-radio label="data">数据备份</el-radio>
            <el-radio label="incremental">增量备份</el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item label="备份内容">
          <el-checkbox-group v-model="createBackupData.contents">
            <el-checkbox label="system">系统配置</el-checkbox>
            <el-checkbox label="drivers">驱动配置</el-checkbox>
            <el-checkbox label="connectors">连接器配置</el-checkbox>
            <el-checkbox label="datapoints">数据点配置</el-checkbox>
            <el-checkbox label="alerts">告警规则</el-checkbox>
            <el-checkbox label="users">用户数据</el-checkbox>
            <el-checkbox label="historical">历史数据</el-checkbox>
          </el-checkbox-group>
        </el-form-item>

        <el-form-item label="压缩选项">
          <el-switch
            v-model="createBackupData.compress"
            active-text="启用压缩"
            inactive-text="不压缩"
          />
        </el-form-item>

        <el-form-item label="加密保护">
          <el-switch
            v-model="createBackupData.encrypt"
            active-text="加密"
            inactive-text="不加密"
          />
        </el-form-item>

        <el-form-item
          v-if="createBackupData.encrypt"
          label="加密密码"
          prop="password"
        >
          <el-input
            v-model="createBackupData.password"
            type="password"
            placeholder="请输入加密密码"
            show-password
          />
        </el-form-item>

        <el-form-item label="备份描述">
          <el-input
            v-model="createBackupData.description"
            type="textarea"
            :rows="3"
            placeholder="备份描述（可选）"
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="closeCreateBackupDialog">取消</el-button>
        <el-button type="primary" :loading="creating" @click="createBackup">
          创建备份
        </el-button>
      </template>
    </el-dialog>

    <!-- 恢复确认对话框 -->
    <el-dialog v-model="restoreDialogVisible" title="确认恢复" width="500px">
      <div class="restore-warning">
        <el-icon class="warning-icon"><WarningFilled /></el-icon>
        <div class="warning-content">
          <h3>警告</h3>
          <p>恢复操作将覆盖当前系统数据，此操作不可撤销。</p>
          <div class="backup-info">
            <p><strong>备份名称:</strong> {{ restoreTarget?.name }}</p>
            <p>
              <strong>创建时间:</strong>
              {{ formatDate(restoreTarget?.createdAt) }}
            </p>
            <p>
              <strong>备份类型:</strong> {{ getTypeLabel(restoreTarget?.type) }}
            </p>
          </div>
        </div>
      </div>

      <el-form v-if="restoreTarget?.encrypted">
        <el-form-item label="解密密码">
          <el-input
            v-model="restorePassword"
            type="password"
            placeholder="请输入解密密码"
            show-password
          />
        </el-form-item>
      </el-form>

      <el-checkbox v-model="restoreConfirmed">
        我确认要执行此恢复操作
      </el-checkbox>

      <template #footer>
        <el-button @click="restoreDialogVisible = false">取消</el-button>
        <el-button
          type="danger"
          :disabled="!restoreConfirmed"
          :loading="restoring"
          @click="confirmRestore"
        >
          确认恢复
        </el-button>
      </template>
    </el-dialog>

    <!-- 计划任务对话框 -->
    <el-dialog
      v-model="scheduleDialogVisible"
      title="备份计划任务"
      width="700px"
    >
      <div class="schedule-section">
        <!-- 现有任务列表 -->
        <div class="existing-schedules">
          <h3>现有计划任务</h3>
          <div class="schedule-list">
            <div
              v-for="schedule in schedules"
              :key="schedule.id"
              class="schedule-item"
            >
              <div class="schedule-info">
                <div class="schedule-name">{{ schedule.name }}</div>
                <div class="schedule-details">
                  <span>{{ schedule.cron }}</span>
                  <span>{{ getTypeLabel(schedule.backupType) }}</span>
                </div>
              </div>
              <div class="schedule-status">
                <el-switch
                  v-model="schedule.enabled"
                  @change="toggleSchedule(schedule)"
                />
              </div>
              <div class="schedule-actions">
                <el-button text @click="editSchedule(schedule)">编辑</el-button>
                <el-button text type="danger" @click="deleteSchedule(schedule)"
                  >删除</el-button
                >
              </div>
            </div>
          </div>
        </div>

        <!-- 创建新任务 -->
        <div class="create-schedule">
          <h3>创建新任务</h3>
          <el-form :model="newSchedule" label-width="100px">
            <el-form-item label="任务名称">
              <el-input v-model="newSchedule.name" placeholder="任务名称" />
            </el-form-item>
            <el-form-item label="备份类型">
              <el-select v-model="newSchedule.backupType">
                <el-option label="完整备份" value="full" />
                <el-option label="配置备份" value="config" />
                <el-option label="数据备份" value="data" />
                <el-option label="增量备份" value="incremental" />
              </el-select>
            </el-form-item>
            <el-form-item label="执行频率">
              <el-select
                v-model="newSchedule.frequency"
                @change="updateCronExpression"
              >
                <el-option label="每天" value="daily" />
                <el-option label="每周" value="weekly" />
                <el-option label="每月" value="monthly" />
                <el-option label="自定义" value="custom" />
              </el-select>
            </el-form-item>
            <el-form-item
              v-if="newSchedule.frequency !== 'custom'"
              label="执行时间"
            >
              <el-time-picker
                v-model="newSchedule.time"
                format="HH:mm"
                value-format="HH:mm"
              />
            </el-form-item>
            <el-form-item
              v-if="newSchedule.frequency === 'custom'"
              label="Cron表达式"
            >
              <el-input v-model="newSchedule.cron" placeholder="0 2 * * *" />
            </el-form-item>
            <el-form-item label="保留份数">
              <el-input-number
                v-model="newSchedule.retention"
                :min="1"
                :max="30"
              />
            </el-form-item>
          </el-form>
        </div>
      </div>

      <template #footer>
        <el-button @click="scheduleDialogVisible = false">关闭</el-button>
        <el-button type="primary" @click="createSchedule">创建任务</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  Plus,
  Timer,
  Search,
  Document,
  Folder,
  SuccessFilled,
  Clock,
  MoreFilled,
  RefreshRight,
  Download,
  CircleCheck,
  Delete,
  FolderOpened,
  WarningFilled,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'

import { backupApi, wsClient } from '@/api'
import type {
  BackupItem,
  BackupStats,
  BackupCreateReq,
  BackupSchedule,
  BackupOperation,
} from '@/api/backup'

// 使用从 API 导入的类型

// 响应式数据
const loading = ref(false)
const searchQuery = ref('')
const selectedType = ref('')
const selectedBackup = ref<BackupItem | null>(null)
const createBackupVisible = ref(false)
const restoreDialogVisible = ref(false)
const scheduleDialogVisible = ref(false)
const creating = ref(false)
const restoring = ref(false)
const restoreTarget = ref<BackupItem | null>(null)
const restorePassword = ref('')
const restoreConfirmed = ref(false)

// 表单引用
const createBackupForm = ref<FormInstance>()

// 备份数据
const backups = ref<BackupItem[]>([])
const activeOperations = ref<BackupOperation[]>([])
const schedules = ref<BackupSchedule[]>([])

// 统计信息
const backupStats = ref<BackupStats>({
  totalBackups: 0,
  totalSize: 0,
  successRate: 95,
  scheduledTasks: 0,
})

// 创建备份表单数据
const createBackupData = reactive({
  name: '',
  type: 'full',
  contents: ['system', 'drivers', 'connectors', 'datapoints'],
  compress: true,
  encrypt: false,
  password: '',
  description: '',
})

// 新建计划任务数据
const newSchedule = reactive({
  name: '',
  backupType: 'full',
  frequency: 'daily',
  time: '02:00',
  cron: '0 2 * * *',
  retention: 7,
})

// 表单验证规则
const createBackupRules: FormRules = {
  name: [{ required: true, message: '请输入备份名称', trigger: 'blur' }],
  type: [{ required: true, message: '请选择备份类型', trigger: 'change' }],
  password: [
    { required: true, message: '请输入加密密码', trigger: 'blur' },
    { min: 6, message: '密码长度不能少于6位', trigger: 'blur' },
  ],
}

// 计算属性
const filteredBackups = computed(() => {
  let filtered = backups.value

  if (searchQuery.value) {
    filtered = filtered.filter(backup =>
      backup.name.toLowerCase().includes(searchQuery.value.toLowerCase())
    )
  }

  if (selectedType.value) {
    filtered = filtered.filter(backup => backup.type === selectedType.value)
  }

  return filtered.sort(
    (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
  )
})

// 工具函数
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
}

const formatDate = (dateString: string): string => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const getTypeLabel = (type: string): string => {
  const labels = {
    full: '完整备份',
    config: '配置备份',
    data: '数据备份',
    incremental: '增量备份',
  }
  return labels[type as keyof typeof labels] || type
}

const getStatusType = (status: string) => {
  const types = {
    success: 'success',
    failed: 'danger',
    running: 'warning',
    pending: 'info',
  }
  return types[status as keyof typeof types] || 'info'
}

const getStatusLabel = (status: string): string => {
  const labels = {
    success: '成功',
    failed: '失败',
    running: '进行中',
    pending: '等待中',
  }
  return labels[status as keyof typeof labels] || status
}

// 备份操作
const selectBackup = (backup: BackupItem) => {
  selectedBackup.value = backup
}

const openCreateBackupDialog = () => {
  createBackupData.name = `Backup_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}`
  createBackupVisible.value = true
}

const closeCreateBackupDialog = () => {
  createBackupVisible.value = false
  createBackupForm.value?.resetFields()
  Object.assign(createBackupData, {
    name: '',
    type: 'full',
    contents: ['system', 'drivers', 'connectors', 'datapoints'],
    compress: true,
    encrypt: false,
    password: '',
    description: '',
  })
}

const createBackup = async () => {
  if (!createBackupForm.value) return

  try {
    await createBackupForm.value.validate()
    creating.value = true

    const backupRequest: BackupCreateReq = {
      name: createBackupData.name,
      type: createBackupData.type,
      contents: createBackupData.contents,
      compress: createBackupData.compress,
      encrypt: createBackupData.encrypt,
      password: createBackupData.password,
      description: createBackupData.description,
    }

    await backupApi.create(backupRequest)
    ElMessage.success('备份创建成功')
    closeCreateBackupDialog()
    await loadBackups()
    await loadOperations()
  } catch (error) {
    console.error('创建备份失败:', error)
    ElMessage.error('创建备份失败')
  } finally {
    creating.value = false
  }
}

const restoreBackup = (backup: BackupItem) => {
  restoreTarget.value = backup
  restorePassword.value = ''
  restoreConfirmed.value = false
  restoreDialogVisible.value = true
}

const confirmRestore = async () => {
  if (!restoreTarget.value) return

  try {
    restoring.value = true

    await backupApi.restore({
      backupId: restoreTarget.value.id,
      password: restorePassword.value,
    })

    ElMessage.success('系统恢复成功')
    restoreDialogVisible.value = false
    await loadOperations()
  } catch (error) {
    console.error('恢复失败:', error)
    ElMessage.error('恢复失败')
  } finally {
    restoring.value = false
  }
}

const downloadBackup = async (backup: BackupItem) => {
  try {
    ElMessage.info(`开始下载备份: ${backup.name}`)
    const response = await backupApi.download(backup.id)

    // 创建下载链接
    const url = window.URL.createObjectURL(new Blob([response.data]))
    const link = document.createElement('a')
    link.href = url
    link.download = `${backup.name}.backup`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)

    ElMessage.success('备份下载完成')
  } catch (error) {
    console.error('下载备份失败:', error)
    ElMessage.error('下载备份失败')
  }
}

const verifyBackup = async (backup: BackupItem) => {
  try {
    ElMessage.info('开始验证备份完整性...')

    await backupApi.verify(backup.id)
    ElMessage.success('备份验证通过')
    await loadBackups()
  } catch (error) {
    console.error('验证备份失败:', error)
    ElMessage.error('备份验证失败')
  }
}

const deleteBackup = async (backup: BackupItem) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除备份 "${backup.name}" 吗？此操作不可撤销。`,
      '确认删除',
      {
        type: 'warning',
        confirmButtonText: '删除',
        cancelButtonText: '取消',
      }
    )

    await backupApi.delete(backup.id)

    if (selectedBackup.value?.id === backup.id) {
      selectedBackup.value = null
    }

    ElMessage.success('备份删除成功')
    await loadBackups()
    await loadStats()
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除备份失败:', error)
      ElMessage.error('删除备份失败')
    }
  }
}

const cancelOperation = async (operationId: string) => {
  try {
    await backupApi.cancelOperation(operationId)
    ElMessage.warning('操作已取消')
    await loadOperations()
  } catch (error) {
    console.error('取消操作失败:', error)
    ElMessage.error('取消操作失败')
  }
}

// 计划任务管理
const openScheduleDialog = () => {
  scheduleDialogVisible.value = true
}

const updateCronExpression = () => {
  const time = newSchedule.time.split(':')
  const hour = time[0]
  const minute = time[1] || '0'

  switch (newSchedule.frequency) {
    case 'daily':
      newSchedule.cron = `${minute} ${hour} * * *`
      break
    case 'weekly':
      newSchedule.cron = `${minute} ${hour} * * 0`
      break
    case 'monthly':
      newSchedule.cron = `${minute} ${hour} 1 * *`
      break
  }
}

const createSchedule = async () => {
  if (!newSchedule.name) {
    ElMessage.error('请输入任务名称')
    return
  }

  try {
    await backupApi.createSchedule({
      name: newSchedule.name,
      backupType: newSchedule.backupType,
      cron: newSchedule.cron,
      retention: newSchedule.retention,
    })

    Object.assign(newSchedule, {
      name: '',
      backupType: 'full',
      frequency: 'daily',
      time: '02:00',
      cron: '0 2 * * *',
      retention: 7,
    })

    ElMessage.success('计划任务创建成功')
    await loadSchedules()
    await loadStats()
  } catch (error) {
    console.error('创建计划任务失败:', error)
    ElMessage.error('创建计划任务失败')
  }
}

const toggleSchedule = async (schedule: BackupSchedule) => {
  try {
    await backupApi.toggleSchedule(schedule.id, schedule.enabled)
    ElMessage.success(`计划任务已${schedule.enabled ? '启用' : '禁用'}`)
  } catch (error) {
    // 回滚状态
    schedule.enabled = !schedule.enabled
    console.error('切换计划任务状态失败:', error)
    ElMessage.error('切换计划任务状态失败')
  }
}

const editSchedule = (schedule: Schedule) => {
  ElMessage.info('编辑功能开发中...')
}

const deleteSchedule = async (schedule: BackupSchedule) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除计划任务 "${schedule.name}" 吗？`,
      '确认删除',
      {
        type: 'warning',
      }
    )

    await backupApi.deleteSchedule(schedule.id)
    ElMessage.success('计划任务删除成功')
    await loadSchedules()
    await loadStats()
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除计划任务失败:', error)
      ElMessage.error('删除计划任务失败')
    }
  }
}

const loadStats = async () => {
  try {
    const response = await backupApi.getStats()
    backupStats.value = response.data
  } catch (error) {
    console.error('加载统计信息失败:', error)
  }
}

// 数据加载函数
const loadBackups = async () => {
  try {
    loading.value = true
    const response = await backupApi.list({
      search: searchQuery.value,
      type: selectedType.value,
      page: 1,
      size: 100,
    })
    backups.value = response.data.items || response.data

    // 选中第一个备份
    if (backups.value.length > 0 && !selectedBackup.value) {
      selectedBackup.value = backups.value[0]
    }
  } catch (error) {
    console.error('加载备份列表失败:', error)
    ElMessage.error('加载备份列表失败')
  } finally {
    loading.value = false
  }
}

const loadOperations = async () => {
  try {
    const response = await backupApi.getOperations()
    activeOperations.value = response.data
  } catch (error) {
    console.error('加载操作列表失败:', error)
  }
}

const loadSchedules = async () => {
  try {
    const response = await backupApi.getSchedules()
    schedules.value = response.data
  } catch (error) {
    console.error('加载计划任务失败:', error)
  }
}

const initializeData = async () => {
  await Promise.all([
    loadBackups(),
    loadStats(),
    loadOperations(),
    loadSchedules(),
  ])
}

// 生命周期
onMounted(async () => {
  await initializeData()

  // 设置WebSocket监听器
  wsClient.on('backup_progress', (data: BackupOperation) => {
    const index = activeOperations.value.findIndex(op => op.id === data.id)
    if (index > -1) {
      activeOperations.value[index] = data
    } else {
      activeOperations.value.push(data)
    }
  })

  wsClient.on('backup_completed', async (data: { operationId: string }) => {
    const index = activeOperations.value.findIndex(
      op => op.id === data.operationId
    )
    if (index > -1) {
      activeOperations.value.splice(index, 1)
    }
    await loadBackups()
    await loadStats()
  })
})

onUnmounted(() => {
  wsClient.off('backup_progress')
  wsClient.off('backup_completed')
})
</script>

<style scoped lang="scss">
.system-backup {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f5f7fa;

  .page-header {
    background: white;
    padding: 24px;
    border-bottom: 1px solid #e4e7ed;

    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .title-section {
        h1 {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #303133;
        }

        .description {
          margin: 8px 0 0 0;
          color: #909399;
          font-size: 14px;
        }
      }

      .header-actions {
        display: flex;
        gap: 12px;
      }
    }
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    padding: 20px 24px;

    .stat-card {
      .stat-content {
        display: flex;
        align-items: center;
        position: relative;

        .stat-value {
          font-size: 32px;
          font-weight: bold;
          color: #303133;
          margin-bottom: 4px;
        }

        .stat-label {
          color: #909399;
          font-size: 14px;
        }

        .stat-icon {
          position: absolute;
          right: 12px;
          top: 50%;
          transform: translateY(-50%);
          font-size: 32px;
          opacity: 0.3;

          &.backup {
            color: #409eff;
          }
          &.size {
            color: #67c23a;
          }
          &.success {
            color: #67c23a;
          }
          &.scheduled {
            color: #e6a23c;
          }
        }
      }
    }
  }

  .main-content {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 400px;
    gap: 20px;
    padding: 0 24px 24px;
    min-height: 0;

    .backup-list-section {
      .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;

        .header-actions {
          display: flex;
          gap: 12px;
          align-items: center;
        }
      }

      .backup-list {
        max-height: 600px;
        overflow-y: auto;

        .backup-item {
          display: flex;
          align-items: center;
          padding: 16px;
          border: 1px solid #e4e7ed;
          border-radius: 8px;
          margin-bottom: 12px;
          cursor: pointer;
          transition: all 0.3s;

          &:hover {
            border-color: #409eff;
            box-shadow: 0 2px 8px rgba(64, 158, 255, 0.1);
          }

          &.active {
            border-color: #409eff;
            background: #f0f9ff;
          }

          .backup-info {
            flex: 1;

            .backup-name {
              font-weight: 500;
              color: #303133;
              margin-bottom: 6px;
            }

            .backup-meta {
              display: flex;
              gap: 12px;
              margin-bottom: 4px;

              .backup-type {
                padding: 2px 8px;
                border-radius: 4px;
                font-size: 12px;

                &.full {
                  background: #e1f3d8;
                  color: #529b2e;
                }
                &.config {
                  background: #e6f7ff;
                  color: #1890ff;
                }
                &.data {
                  background: #fff2e8;
                  color: #d46b08;
                }
                &.incremental {
                  background: #f6f6f6;
                  color: #666;
                }
              }

              .backup-size {
                font-size: 12px;
                color: #909399;
              }
            }

            .backup-date {
              font-size: 12px;
              color: #909399;
            }
          }

          .backup-status {
            margin: 0 12px;
          }

          .backup-actions {
            .el-button {
              padding: 4px;
            }
          }
        }

        .empty-state {
          text-align: center;
          padding: 40px;
          color: #909399;

          .el-icon {
            font-size: 48px;
            margin-bottom: 16px;
          }
        }
      }
    }

    .backup-detail-section {
      display: flex;
      flex-direction: column;
      gap: 16px;

      .backup-details {
        .detail-section {
          margin-bottom: 24px;

          h3 {
            margin: 0 0 16px 0;
            font-size: 16px;
            font-weight: 500;
            color: #303133;
            padding-bottom: 8px;
            border-bottom: 1px solid #e4e7ed;
          }

          .info-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 12px;

            .info-item {
              display: flex;

              label {
                min-width: 80px;
                font-weight: 500;
                color: #606266;
              }

              span {
                flex: 1;
                color: #303133;

                &.backup-type {
                  padding: 2px 8px;
                  border-radius: 4px;
                  font-size: 12px;
                  display: inline-block;

                  &.full {
                    background: #e1f3d8;
                    color: #529b2e;
                  }
                  &.config {
                    background: #e6f7ff;
                    color: #1890ff;
                  }
                  &.data {
                    background: #fff2e8;
                    color: #d46b08;
                  }
                  &.incremental {
                    background: #f6f6f6;
                    color: #666;
                  }
                }
              }
            }
          }

          .content-list {
            .content-item {
              display: flex;
              align-items: center;
              padding: 8px 0;
              border-bottom: 1px solid #f5f7fa;

              .el-icon {
                margin-right: 8px;
                color: #409eff;
              }

              .item-name {
                flex: 1;
                color: #303133;
              }

              .item-size {
                color: #909399;
                font-size: 12px;
              }
            }
          }

          .verification-info {
            .verification-item {
              display: flex;
              align-items: center;
              margin-bottom: 8px;

              label {
                min-width: 100px;
                font-weight: 500;
                color: #606266;
              }

              .checksum {
                font-family: monospace;
                font-size: 12px;
                color: #67c23a;
                background: #f0f9ff;
                padding: 2px 6px;
                border-radius: 4px;
              }
            }
          }
        }
      }

      .progress-card {
        .operations-list {
          .operation-item {
            padding: 16px 0;
            border-bottom: 1px solid #f5f7fa;

            &:last-child {
              border-bottom: none;
            }

            .operation-info {
              display: flex;
              justify-content: space-between;
              align-items: center;
              margin-bottom: 8px;

              .operation-name {
                font-weight: 500;
                color: #303133;
              }

              .operation-type {
                font-size: 12px;
                color: #909399;
              }
            }

            .operation-progress {
              margin-bottom: 8px;

              .progress-text {
                font-size: 12px;
                color: #909399;
                margin-top: 4px;
              }
            }

            .operation-actions {
              text-align: right;
            }
          }
        }
      }
    }
  }

  .restore-warning {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    margin-bottom: 20px;

    .warning-icon {
      font-size: 24px;
      color: #e6a23c;
      margin-top: 2px;
    }

    .warning-content {
      flex: 1;

      h3 {
        margin: 0 0 8px 0;
        color: #e6a23c;
      }

      p {
        margin: 0 0 16px 0;
        color: #606266;
      }

      .backup-info {
        background: #fdf6ec;
        padding: 12px;
        border-radius: 4px;
        border: 1px solid #f5dab1;

        p {
          margin: 4px 0;
          font-size: 14px;
        }
      }
    }
  }

  .schedule-section {
    .existing-schedules,
    .create-schedule {
      margin-bottom: 24px;

      h3 {
        margin: 0 0 16px 0;
        font-size: 16px;
        color: #303133;
        padding-bottom: 8px;
        border-bottom: 1px solid #e4e7ed;
      }
    }

    .schedule-list {
      .schedule-item {
        display: flex;
        align-items: center;
        padding: 16px;
        border: 1px solid #e4e7ed;
        border-radius: 8px;
        margin-bottom: 12px;

        .schedule-info {
          flex: 1;

          .schedule-name {
            font-weight: 500;
            color: #303133;
            margin-bottom: 6px;
          }

          .schedule-details {
            display: flex;
            gap: 12px;
            font-size: 12px;
            color: #909399;
          }
        }

        .schedule-status {
          margin: 0 16px;
        }

        .schedule-actions {
          display: flex;
          gap: 8px;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .system-backup {
    .main-content {
      grid-template-columns: 1fr;

      .backup-detail-section {
        order: -1;
      }
    }
  }
}

@media (max-width: 768px) {
  .system-backup {
    .stats-grid {
      grid-template-columns: 1fr 1fr;
      padding: 16px;
    }

    .page-header {
      padding: 16px;

      .header-content {
        flex-direction: column;
        align-items: flex-start;
        gap: 16px;
      }
    }

    .main-content {
      padding: 0 16px 16px;
    }
  }
}
</style>
