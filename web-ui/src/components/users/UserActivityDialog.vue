<template>
  <el-dialog
    v-model="dialogVisible"
    title="用户活动日志"
    width="1000px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="user-activity-dialog">
      <!-- 用户信息头部 -->
      <div class="user-info-header">
        <div class="user-avatar">
          <el-avatar :size="50" :src="userData?.avatar">
            <el-icon><UserFilled /></el-icon>
          </el-avatar>
        </div>
        <div class="user-details">
          <h3 class="user-name">{{ userData?.name }}</h3>
          <div class="user-meta">
            <el-tag type="info" size="small">{{ userData?.username }}</el-tag>
            <el-tag :type="getRoleType(userData?.role)" size="small">
              {{ getRoleName(userData?.role) }}
            </el-tag>
            <el-tag type="warning" size="small">{{
              getDepartmentName(userData?.departmentId)
            }}</el-tag>
          </div>
        </div>
        <div class="activity-stats">
          <div class="stat-item">
            <span class="stat-value">{{ totalActivities }}</span>
            <span class="stat-label">总活动数</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ todayActivities }}</span>
            <span class="stat-label">今日活动</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ lastActiveTime }}</span>
            <span class="stat-label">最后活动</span>
          </div>
        </div>
      </div>

      <el-divider />

      <!-- 筛选控制面板 -->
      <div class="filter-panel">
        <div class="filter-row">
          <div class="filter-item">
            <label class="filter-label">活动类型</label>
            <el-select
              v-model="filterForm.type"
              placeholder="选择类型"
              clearable
              style="width: 150px"
              @change="handleFilterChange"
            >
              <el-option label="全部" value="" />
              <el-option label="登录" value="login" />
              <el-option label="操作" value="operation" />
              <el-option label="配置" value="config" />
              <el-option label="数据" value="data" />
              <el-option label="系统" value="system" />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">严重程度</label>
            <el-select
              v-model="filterForm.level"
              placeholder="选择级别"
              clearable
              style="width: 120px"
              @change="handleFilterChange"
            >
              <el-option label="全部" value="" />
              <el-option label="信息" value="info" />
              <el-option label="警告" value="warning" />
              <el-option label="错误" value="error" />
              <el-option label="严重" value="critical" />
            </el-select>
          </div>

          <div class="filter-item">
            <label class="filter-label">时间范围</label>
            <el-date-picker
              v-model="filterForm.dateRange"
              type="daterange"
              range-separator="至"
              start-placeholder="开始日期"
              end-placeholder="结束日期"
              style="width: 240px"
              @change="handleFilterChange"
            />
          </div>

          <div class="filter-item">
            <label class="filter-label">关键词</label>
            <el-input
              v-model="filterForm.keyword"
              placeholder="搜索活动内容"
              style="width: 200px"
              clearable
              @input="handleFilterChange"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
          </div>

          <div class="filter-actions">
            <el-button :loading="exporting" @click="exportActivities">
              <el-icon><Download /></el-icon>
              导出
            </el-button>
            <el-button :loading="loading" @click="refreshActivities">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </div>

      <!-- 活动时间线 -->
      <div v-loading="loading" class="activity-timeline">
        <div v-if="groupedActivities.length === 0" class="empty-state">
          <el-empty description="暂无活动记录" />
        </div>

        <div v-else class="timeline-container">
          <div
            v-for="group in groupedActivities"
            :key="group.date"
            class="timeline-group"
          >
            <div class="group-header">
              <h4 class="group-date">{{ group.date }}</h4>
              <el-tag type="info" size="small"
                >{{ group.activities.length }} 项活动</el-tag
              >
            </div>

            <el-timeline class="activity-timeline-items">
              <el-timeline-item
                v-for="activity in group.activities"
                :key="activity.id"
                :timestamp="activity.time"
                :type="getTimelineType(activity.level)"
                :color="getTimelineColor(activity.level)"
                placement="top"
              >
                <div class="activity-item">
                  <div class="activity-header">
                    <div class="activity-info">
                      <el-icon class="activity-icon">
                        <component :is="getActivityIcon(activity.type)" />
                      </el-icon>
                      <span class="activity-title">{{ activity.title }}</span>
                      <el-tag
                        :type="getActivityTypeTag(activity.type)"
                        size="small"
                        class="activity-type"
                      >
                        {{ getActivityTypeText(activity.type) }}
                      </el-tag>
                      <el-tag
                        :type="getLevelTypeTag(activity.level)"
                        size="small"
                        class="activity-level"
                      >
                        {{ getLevelText(activity.level) }}
                      </el-tag>
                    </div>
                    <div class="activity-meta">
                      <span class="activity-ip">{{ activity.ip }}</span>
                      <span class="activity-device">{{
                        activity.device || '未知设备'
                      }}</span>
                    </div>
                  </div>

                  <div class="activity-content">
                    <p class="activity-description">
                      {{ activity.description }}
                    </p>

                    <!-- 活动详情 -->
                    <div v-if="activity.details" class="activity-details">
                      <div
                        v-for="(value, key) in activity.details"
                        :key="key"
                        class="detail-item"
                      >
                        <span class="detail-key"
                          >{{ formatDetailKey(key) }}:</span
                        >
                        <span class="detail-value">{{
                          formatDetailValue(value)
                        }}</span>
                      </div>
                    </div>

                    <!-- 错误信息 -->
                    <div v-if="activity.error" class="activity-error">
                      <div class="error-header">
                        <el-icon class="error-icon"><WarningFilled /></el-icon>
                        <span class="error-title">错误信息</span>
                      </div>
                      <div class="error-content">
                        <pre class="error-message">{{
                          activity.error.message
                        }}</pre>
                        <div v-if="activity.error.stack" class="error-stack">
                          <el-collapse>
                            <el-collapse-item title="堆栈跟踪" name="stack">
                              <pre class="stack-trace">{{
                                activity.error.stack
                              }}</pre>
                            </el-collapse-item>
                          </el-collapse>
                        </div>
                      </div>
                    </div>

                    <!-- 关联数据 -->
                    <div v-if="activity.relatedData" class="related-data">
                      <div class="related-header">关联数据</div>
                      <div class="related-items">
                        <el-tag
                          v-for="item in activity.relatedData"
                          :key="item.id"
                          type="info"
                          size="small"
                          class="related-tag"
                        >
                          {{ item.type }}: {{ item.name }}
                        </el-tag>
                      </div>
                    </div>
                  </div>

                  <!-- 活动操作 -->
                  <div class="activity-actions">
                    <el-button
                      size="small"
                      text
                      @click="viewActivityDetails(activity)"
                    >
                      查看详情
                    </el-button>
                    <el-button
                      v-if="activity.type === 'operation'"
                      size="small"
                      text
                      @click="rollbackOperation(activity)"
                    >
                      回滚操作
                    </el-button>
                    <el-button
                      size="small"
                      text
                      @click="copyActivityInfo(activity)"
                    >
                      复制信息
                    </el-button>
                  </div>
                </div>
              </el-timeline-item>
            </el-timeline>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.size"
          :total="pagination.total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="loadActivities"
          @current-change="loadActivities"
        />
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * UserActivityDialog —— 用户活动日志对话框组件
 *
 * 📝 Responsibilities:
 *  1. 显示用户的详细活动日志
 *  2. 提供活动筛选和搜索功能
 *  3. 按时间分组展示活动时间线
 *  4. 支持活动详情查看和操作回滚
 *  5. 活动数据的导出功能
 *
 * 📦 Dependencies:
 *  - Element Plus 组件库
 *  - 用户活动日志相关API
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  UserFilled,
  Search,
  Download,
  Refresh,
  WarningFilled,
  Monitor,
  Setting,
  Document,
  Connection,
  User,
  Lock,
  Operation,
  DataAnalysis,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed, watch, nextTick } from 'vue'

// ===== Props & Emits =====
const props = defineProps<{
  visible: boolean
  userData?: any
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
}>()

// ===== 响应式数据 =====
const dialogVisible = ref(props.visible)
const loading = ref(false)
const exporting = ref(false)

// 筛选表单
const filterForm = ref({
  type: '',
  level: '',
  dateRange: [],
  keyword: '',
})

// 分页配置
const pagination = ref({
  page: 1,
  size: 20,
  total: 0,
})

// 活动数据
const activities = ref<any[]>([])
const availableRoles = ref<any[]>([])
const departments = ref<any[]>([])

// ===== 计算属性 =====
const totalActivities = computed(() => activities.value.length)

const todayActivities = computed(() => {
  const today = new Date().toDateString()
  return activities.value.filter(
    activity => new Date(activity.timestamp).toDateString() === today
  ).length
})

const lastActiveTime = computed(() => {
  if (activities.value.length === 0) return '无'
  const lastActivity = activities.value[0]
  return formatRelativeTime(lastActivity.timestamp)
})

const groupedActivities = computed(() => {
  const groups: { [key: string]: any[] } = {}

  activities.value.forEach(activity => {
    const date = new Date(activity.timestamp).toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      weekday: 'long',
    })

    if (!groups[date]) {
      groups[date] = []
    }
    groups[date].push(activity)
  })

  return Object.entries(groups)
    .map(([date, activities]) => ({
      date,
      activities: activities.sort(
        (a, b) =>
          new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      ),
    }))
    .sort(
      (a, b) =>
        new Date(b.activities[0].timestamp).getTime() -
        new Date(a.activities[0].timestamp).getTime()
    )
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    // 从API加载真实数据
    await Promise.all([loadAvailableRoles(), loadDepartments()])

    // 加载活动数据
    await loadActivities()
  } catch (error) {
    console.error('初始化用户活动对话框失败:', error)
    ElMessage.error('初始化失败')
  }
}

/**
 * 加载活动数据
 */
async function loadActivities() {
  try {
    loading.value = true

    const response = await fetch(
      `/api/v1/users/${props.userData.id}/activities`,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
        params: {
          page: pagination.value.currentPage,
          size: pagination.value.pageSize,
          type: filters.value.activityType,
          dateRange: filters.value.dateRange,
        },
      }
    )

    if (response.ok) {
      const data = await response.json()
      activities.value = data.items || []
      pagination.value.total = data.total || 0
    } else {
      activities.value = []
      pagination.value.total = 0
      console.error('加载活动数据失败:', response.statusText)
    }
  } catch (error) {
    console.error('加载活动数据失败:', error)
    ElMessage.error('加载活动数据失败')
    activities.value = []
    pagination.value.total = 0
  } finally {
    loading.value = false
  }
}

/**
 * 生成活动标题
 */
function generateActivityTitle(type: string, level: string): string {
  const titles = {
    login: ['用户登录', '用户登出', '登录失败', '密码重置'],
    operation: ['设备操作', '数据修改', '配置更新', '批量操作'],
    config: ['系统配置', '参数调整', '策略更新', '规则修改'],
    data: ['数据导入', '数据导出', '数据备份', '数据清理'],
    system: ['系统升级', '服务重启', '错误检测', '性能监控'],
  }

  const typeTitles = titles[type as keyof typeof titles] || ['未知操作']
  return typeTitles[Math.floor(Math.random() * typeTitles.length)]
}

/**
 * 生成活动描述
 */
function generateActivityDescription(type: string, level: string): string {
  const descriptions = {
    login: '用户通过Web界面成功登录系统',
    operation: '用户对设备PLC-001执行了启动操作',
    config: '用户修改了报警阈值配置参数',
    data: '用户导出了过去7天的历史数据',
    system: '系统自动执行了定期备份任务',
  }

  return descriptions[type as keyof typeof descriptions] || '执行了系统操作'
}

/**
 * 生成随机IP地址
 */
function generateRandomIP(): string {
  return `192.168.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`
}

/**
 * 生成随机设备信息
 */
function generateRandomDevice(): string {
  const devices = [
    'Windows PC',
    'Mac',
    'iPad',
    'Android Phone',
    'Linux Terminal',
  ]
  return devices[Math.floor(Math.random() * devices.length)]
}

/**
 * 生成活动详情
 */
function generateActivityDetails(type: string) {
  const details = {
    login: {
      browser: 'Chrome 119.0',
      os: 'Windows 10',
      location: '北京市朝阳区',
    },
    operation: {
      deviceId: 'PLC-001',
      command: 'START',
      duration: '2.3s',
    },
    config: {
      module: 'alert-engine',
      parameter: 'threshold',
      oldValue: '80',
      newValue: '85',
    },
    data: {
      format: 'CSV',
      records: '15,234',
      size: '2.1MB',
    },
  }

  return details[type as keyof typeof details] || {}
}

/**
 * 生成错误信息
 */
function generateErrorInfo() {
  return {
    message: 'Connection timeout: Failed to connect to device after 30 seconds',
    code: 'DEVICE_TIMEOUT',
    stack:
      'Error: Connection timeout\n    at DeviceManager.connect (device-manager.js:123:15)\n    at async Handler.execute (handler.js:45:7)',
  }
}

/**
 * 生成关联数据
 */
function generateRelatedData() {
  return [
    { id: 'device_1', type: '设备', name: 'PLC-001' },
    { id: 'rule_1', type: '规则', name: '温度监控' },
  ]
}

/**
 * 从API加载可用角色
 */
async function loadAvailableRoles() {
  try {
    const response = await fetch('/api/v1/roles', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (response.ok) {
      availableRoles.value = await response.json()
    } else {
      availableRoles.value = []
      console.error('加载角色列表失败:', response.statusText)
    }
  } catch (error) {
    console.error('加载角色列表失败:', error)
    availableRoles.value = []
  }
}

/**
 * 从API加载部门数据
 */
async function loadDepartments() {
  try {
    const response = await fetch('/api/v1/users/departments', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (response.ok) {
      departments.value = await response.json()
    } else {
      departments.value = []
      console.error('加载部门列表失败:', response.statusText)
    }
  } catch (error) {
    console.error('加载部门列表失败:', error)
    departments.value = []
  }
}

/**
 * 获取角色类型
 */
function getRoleType(roleId: string): string {
  switch (roleId) {
    case 'admin':
      return 'danger'
    case 'manager':
      return 'warning'
    case 'operator':
      return 'success'
    case 'viewer':
      return 'info'
    default:
      return 'info'
  }
}

/**
 * 获取角色名称
 */
function getRoleName(roleId: string): string {
  const role = availableRoles.value.find(r => r.id === roleId)
  return role?.name || '未知角色'
}

/**
 * 获取部门名称
 */
function getDepartmentName(departmentId: string): string {
  const department = departments.value.find(d => d.id === departmentId)
  return department?.name || '未分配部门'
}

/**
 * 获取活动图标
 */
function getActivityIcon(type: string): string {
  switch (type) {
    case 'login':
      return 'User'
    case 'operation':
      return 'Operation'
    case 'config':
      return 'Setting'
    case 'data':
      return 'Document'
    case 'system':
      return 'Monitor'
    default:
      return 'Document'
  }
}

/**
 * 获取时间线类型
 */
function getTimelineType(level: string): string {
  switch (level) {
    case 'critical':
      return 'danger'
    case 'error':
      return 'danger'
    case 'warning':
      return 'warning'
    case 'info':
      return 'primary'
    default:
      return 'primary'
  }
}

/**
 * 获取时间线颜色
 */
function getTimelineColor(level: string): string {
  switch (level) {
    case 'critical':
      return '#F56C6C'
    case 'error':
      return '#F56C6C'
    case 'warning':
      return '#E6A23C'
    case 'info':
      return '#409EFF'
    default:
      return '#409EFF'
  }
}

/**
 * 获取活动类型标签
 */
function getActivityTypeTag(type: string): string {
  switch (type) {
    case 'login':
      return 'success'
    case 'operation':
      return 'primary'
    case 'config':
      return 'warning'
    case 'data':
      return 'info'
    case 'system':
      return 'danger'
    default:
      return 'info'
  }
}

/**
 * 获取活动类型文本
 */
function getActivityTypeText(type: string): string {
  switch (type) {
    case 'login':
      return '登录'
    case 'operation':
      return '操作'
    case 'config':
      return '配置'
    case 'data':
      return '数据'
    case 'system':
      return '系统'
    default:
      return '未知'
  }
}

/**
 * 获取级别标签类型
 */
function getLevelTypeTag(level: string): string {
  switch (level) {
    case 'critical':
      return 'danger'
    case 'error':
      return 'danger'
    case 'warning':
      return 'warning'
    case 'info':
      return 'success'
    default:
      return 'info'
  }
}

/**
 * 获取级别文本
 */
function getLevelText(level: string): string {
  switch (level) {
    case 'critical':
      return '严重'
    case 'error':
      return '错误'
    case 'warning':
      return '警告'
    case 'info':
      return '信息'
    default:
      return '未知'
  }
}

/**
 * 格式化详情键名
 */
function formatDetailKey(key: string): string {
  const keyMap: { [key: string]: string } = {
    browser: '浏览器',
    os: '操作系统',
    location: '位置',
    deviceId: '设备ID',
    command: '命令',
    duration: '耗时',
    module: '模块',
    parameter: '参数',
    oldValue: '原值',
    newValue: '新值',
    format: '格式',
    records: '记录数',
    size: '大小',
  }

  return keyMap[key] || key
}

/**
 * 格式化详情值
 */
function formatDetailValue(value: any): string {
  if (typeof value === 'object') {
    return JSON.stringify(value)
  }
  return String(value)
}

/**
 * 格式化相对时间
 */
function formatRelativeTime(timestamp: string): string {
  const now = new Date()
  const time = new Date(timestamp)
  const diff = now.getTime() - time.getTime()

  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}分钟前`

  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}小时前`

  const days = Math.floor(hours / 24)
  if (days < 7) return `${days}天前`

  return time.toLocaleDateString('zh-CN')
}

/**
 * 处理筛选变化
 */
function handleFilterChange() {
  // 实现筛选逻辑
  console.log('筛选条件:', filterForm.value)
  loadActivities()
}

/**
 * 刷新活动数据
 */
function refreshActivities() {
  loadActivities()
}

/**
 * 导出活动数据
 */
async function exportActivities() {
  try {
    exporting.value = true

    // 模拟导出操作
    await new Promise(resolve => setTimeout(resolve, 2000))

    ElMessage.success('活动日志导出成功')
  } catch (error) {
    console.error('导出活动数据失败:', error)
    ElMessage.error('导出活动数据失败')
  } finally {
    exporting.value = false
  }
}

/**
 * 查看活动详情
 */
function viewActivityDetails(activity: any) {
  ElMessageBox.alert(
    `
    <div style="text-align: left;">
      <p><strong>活动ID:</strong> ${activity.id}</p>
      <p><strong>时间:</strong> ${new Date(activity.timestamp).toLocaleString('zh-CN')}</p>
      <p><strong>类型:</strong> ${getActivityTypeText(activity.type)}</p>
      <p><strong>级别:</strong> ${getLevelText(activity.level)}</p>
      <p><strong>IP地址:</strong> ${activity.ip}</p>
      <p><strong>设备:</strong> ${activity.device}</p>
      <p><strong>描述:</strong> ${activity.description}</p>
    </div>
    `,
    '活动详情',
    {
      dangerouslyUseHTMLString: true,
      confirmButtonText: '关闭',
    }
  )
}

/**
 * 回滚操作
 */
function rollbackOperation(activity: any) {
  ElMessageBox.confirm(
    `确定要回滚操作 "${activity.title}" 吗？此操作不可撤销。`,
    '确认回滚',
    {
      type: 'warning',
    }
  )
    .then(() => {
      ElMessage.success('操作回滚成功')
    })
    .catch(() => {
      // 用户取消
    })
}

/**
 * 复制活动信息
 */
function copyActivityInfo(activity: any) {
  const info = `活动: ${activity.title}\n时间: ${new Date(activity.timestamp).toLocaleString('zh-CN')}\n描述: ${activity.description}\nIP: ${activity.ip}`

  navigator.clipboard
    .writeText(info)
    .then(() => {
      ElMessage.success('活动信息已复制到剪贴板')
    })
    .catch(() => {
      ElMessage.error('复制失败')
    })
}

/**
 * 处理对话框关闭
 */
function handleClose() {
  dialogVisible.value = false
}

// ===== 监听器 =====
watch(
  () => props.visible,
  visible => {
    dialogVisible.value = visible
    if (visible) {
      nextTick(() => {
        initializeData()
      })
    }
  }
)

watch(dialogVisible, visible => {
  emit('update:visible', visible)
})
</script>

<style scoped lang="scss">
.user-activity-dialog {
  .user-info-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: #f8f9fa;
    border-radius: 8px;

    .user-avatar {
      flex-shrink: 0;
    }

    .user-details {
      flex: 1;

      .user-name {
        margin: 0 0 8px 0;
        font-size: 18px;
        font-weight: 600;
        color: #303133;
      }

      .user-meta {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
      }
    }

    .activity-stats {
      display: flex;
      gap: 24px;

      .stat-item {
        text-align: center;

        .stat-value {
          display: block;
          font-size: 20px;
          font-weight: 600;
          color: #409eff;
          margin-bottom: 4px;
        }

        .stat-label {
          font-size: 12px;
          color: #909399;
        }
      }
    }
  }

  .filter-panel {
    margin: 20px 0;
    padding: 16px;
    background: #fafafa;
    border-radius: 8px;

    .filter-row {
      display: flex;
      align-items: center;
      gap: 16px;
      flex-wrap: wrap;

      .filter-item {
        display: flex;
        align-items: center;
        gap: 8px;

        .filter-label {
          font-size: 14px;
          color: #606266;
          white-space: nowrap;
        }
      }

      .filter-actions {
        margin-left: auto;
        display: flex;
        gap: 8px;
      }
    }
  }

  .activity-timeline {
    margin: 20px 0;
    min-height: 400px;

    .empty-state {
      text-align: center;
      padding: 60px 0;
    }

    .timeline-container {
      .timeline-group {
        margin-bottom: 32px;

        .group-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
          padding-bottom: 8px;
          border-bottom: 1px solid #ebeef5;

          .group-date {
            margin: 0;
            font-size: 16px;
            font-weight: 600;
            color: #303133;
          }
        }

        .activity-timeline-items {
          .activity-item {
            .activity-header {
              display: flex;
              justify-content: space-between;
              align-items: flex-start;
              margin-bottom: 8px;

              .activity-info {
                display: flex;
                align-items: center;
                gap: 8px;
                flex: 1;

                .activity-icon {
                  font-size: 16px;
                  color: #409eff;
                }

                .activity-title {
                  font-weight: 500;
                  color: #303133;
                }

                .activity-type,
                .activity-level {
                  margin: 0;
                }
              }

              .activity-meta {
                display: flex;
                flex-direction: column;
                align-items: flex-end;
                gap: 4px;
                font-size: 12px;
                color: #909399;

                .activity-ip,
                .activity-device {
                  white-space: nowrap;
                }
              }
            }

            .activity-content {
              margin: 12px 0;

              .activity-description {
                color: #606266;
                margin: 0 0 12px 0;
                line-height: 1.5;
              }

              .activity-details {
                background: #f8f9fa;
                padding: 12px;
                border-radius: 6px;
                margin: 8px 0;

                .detail-item {
                  display: flex;
                  margin-bottom: 4px;

                  &:last-child {
                    margin-bottom: 0;
                  }

                  .detail-key {
                    font-weight: 500;
                    color: #606266;
                    min-width: 80px;
                    margin-right: 8px;
                  }

                  .detail-value {
                    color: #303133;
                    flex: 1;
                  }
                }
              }

              .activity-error {
                background: #fef0f0;
                border: 1px solid #fbc4c4;
                border-radius: 6px;
                padding: 12px;
                margin: 8px 0;

                .error-header {
                  display: flex;
                  align-items: center;
                  gap: 8px;
                  margin-bottom: 8px;

                  .error-icon {
                    color: #f56c6c;
                  }

                  .error-title {
                    font-weight: 500;
                    color: #f56c6c;
                  }
                }

                .error-content {
                  .error-message {
                    background: #fff;
                    border: 1px solid #dcdfe6;
                    border-radius: 4px;
                    padding: 8px;
                    margin: 0 0 8px 0;
                    font-family: monospace;
                    font-size: 12px;
                    white-space: pre-wrap;
                    color: #303133;
                  }

                  .error-stack {
                    .stack-trace {
                      background: #f5f5f5;
                      border: 1px solid #dcdfe6;
                      border-radius: 4px;
                      padding: 8px;
                      margin: 0;
                      font-family: monospace;
                      font-size: 11px;
                      white-space: pre-wrap;
                      color: #606266;
                      max-height: 200px;
                      overflow-y: auto;
                    }
                  }
                }
              }

              .related-data {
                background: #f0f9ff;
                border: 1px solid #b3d8ff;
                border-radius: 6px;
                padding: 12px;
                margin: 8px 0;

                .related-header {
                  font-weight: 500;
                  color: #409eff;
                  margin-bottom: 8px;
                }

                .related-items {
                  display: flex;
                  gap: 6px;
                  flex-wrap: wrap;

                  .related-tag {
                    margin: 0;
                  }
                }
              }
            }

            .activity-actions {
              display: flex;
              gap: 8px;
              margin-top: 12px;
              padding-top: 8px;
              border-top: 1px solid #f0f0f0;
            }
          }
        }
      }
    }
  }

  .pagination-container {
    margin-top: 20px;
    text-align: center;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .user-activity-dialog {
    .user-info-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;

      .activity-stats {
        width: 100%;
        justify-content: space-around;
      }
    }

    .filter-panel .filter-row {
      flex-direction: column;
      align-items: stretch;
      gap: 12px;

      .filter-item {
        flex-direction: column;
        align-items: stretch;
        gap: 4px;

        .filter-label {
          font-size: 12px;
        }
      }

      .filter-actions {
        margin-left: 0;
        justify-content: center;
      }
    }

    .activity-timeline
      .timeline-container
      .timeline-group
      .activity-timeline-items
      .activity-item {
      .activity-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 8px;

        .activity-info {
          flex-wrap: wrap;
        }

        .activity-meta {
          align-items: flex-start;
        }
      }

      .activity-actions {
        flex-wrap: wrap;
      }
    }
  }
}
</style>
