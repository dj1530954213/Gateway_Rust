<template>
  <el-dialog
    v-model="dialogVisible"
    title="用户会话管理"
    width="1000px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="user-session-dialog">
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
        <div class="session-stats">
          <div class="stat-item">
            <span class="stat-value">{{ activeSessions.length }}</span>
            <span class="stat-label">活跃会话</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ totalSessions }}</span>
            <span class="stat-label">总会话数</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ onlineTime }}</span>
            <span class="stat-label">在线时长</span>
          </div>
        </div>
      </div>

      <el-divider />

      <!-- 会话控制面板 -->
      <div class="session-controls">
        <div class="control-row">
          <div class="session-actions">
            <el-button
              type="danger"
              :disabled="activeSessions.length === 0"
              :loading="terminating"
              @click="terminateAllSessions"
            >
              <el-icon><CircleClose /></el-icon>
              终止所有会话
            </el-button>
            <el-button
              type="warning"
              :disabled="activeSessions.length === 0"
              @click="kickOutUser"
            >
              <el-icon><SwitchButton /></el-icon>
              强制下线
            </el-button>
            <el-button :loading="loading" @click="refreshSessions">
              <el-icon><Refresh /></el-icon>
              刷新数据
            </el-button>
          </div>

          <div class="view-controls">
            <el-radio-group v-model="viewMode" @change="handleViewModeChange">
              <el-radio-button label="active">活跃会话</el-radio-button>
              <el-radio-button label="history">历史会话</el-radio-button>
              <el-radio-button label="all">全部会话</el-radio-button>
            </el-radio-group>
          </div>
        </div>
      </div>

      <!-- 会话列表 -->
      <div v-loading="loading" class="sessions-container">
        <div v-if="filteredSessions.length === 0" class="empty-state">
          <el-empty :description="getEmptyDescription()" />
        </div>

        <div v-else class="sessions-grid">
          <div
            v-for="session in filteredSessions"
            :key="session.id"
            :class="[
              'session-card',
              {
                active: session.status === 'active',
                expired: session.status === 'expired',
                terminated: session.status === 'terminated',
              },
            ]"
          >
            <!-- 会话头部 -->
            <div class="session-header">
              <div class="session-info">
                <div class="session-title">
                  <el-icon class="device-icon">
                    <component :is="getDeviceIcon(session.device?.type)" />
                  </el-icon>
                  <span class="session-name">{{
                    session.device?.name || '未知设备'
                  }}</span>
                  <el-tag
                    :type="getStatusType(session.status)"
                    size="small"
                    class="session-status"
                  >
                    {{ getStatusText(session.status) }}
                  </el-tag>
                </div>
                <div class="session-meta">
                  <span class="session-id">{{ session.id }}</span>
                  <span class="session-location">{{
                    session.location || '未知位置'
                  }}</span>
                </div>
              </div>

              <div class="session-actions">
                <el-dropdown
                  v-if="session.status === 'active'"
                  @command="handleSessionAction"
                >
                  <el-button size="small" text>
                    <el-icon><MoreFilled /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item :command="`terminate_${session.id}`">
                        终止会话
                      </el-dropdown-item>
                      <el-dropdown-item :command="`refresh_${session.id}`">
                        刷新令牌
                      </el-dropdown-item>
                      <el-dropdown-item
                        :command="`details_${session.id}`"
                        divided
                      >
                        查看详情
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
                <el-button
                  v-else
                  size="small"
                  text
                  @click="viewSessionDetails(session)"
                >
                  查看详情
                </el-button>
              </div>
            </div>

            <!-- 会话详情 -->
            <div class="session-details">
              <div class="detail-grid">
                <div class="detail-item">
                  <div class="detail-icon">
                    <el-icon><Monitor /></el-icon>
                  </div>
                  <div class="detail-content">
                    <div class="detail-label">设备信息</div>
                    <div class="detail-value">
                      {{ session.device?.os }} · {{ session.device?.browser }}
                    </div>
                  </div>
                </div>

                <div class="detail-item">
                  <div class="detail-icon">
                    <el-icon><Connection /></el-icon>
                  </div>
                  <div class="detail-content">
                    <div class="detail-label">IP地址</div>
                    <div class="detail-value">{{ session.ip }}</div>
                  </div>
                </div>

                <div class="detail-item">
                  <div class="detail-icon">
                    <el-icon><Clock /></el-icon>
                  </div>
                  <div class="detail-content">
                    <div class="detail-label">登录时间</div>
                    <div class="detail-value">
                      {{ formatDateTime(session.loginTime) }}
                    </div>
                  </div>
                </div>

                <div class="detail-item">
                  <div class="detail-icon">
                    <el-icon><Timer /></el-icon>
                  </div>
                  <div class="detail-content">
                    <div class="detail-label">持续时长</div>
                    <div class="detail-value">
                      {{ formatDuration(session.duration) }}
                    </div>
                  </div>
                </div>
              </div>

              <!-- 会话活动 -->
              <div v-if="session.status === 'active'" class="session-activity">
                <div class="activity-header">
                  <span class="activity-title">会话活动</span>
                  <span class="last-activity">
                    最后活动: {{ formatRelativeTime(session.lastActivity) }}
                  </span>
                </div>
                <div class="activity-chart">
                  <div class="activity-bars">
                    <div
                      v-for="(activity, index) in session.activityChart"
                      :key="index"
                      :class="[
                        'activity-bar',
                        {
                          high: activity.level === 'high',
                          medium: activity.level === 'medium',
                          low: activity.level === 'low',
                        },
                      ]"
                      :style="{ height: `${activity.value}%` }"
                      :title="`${activity.time}: ${activity.count} 次活动`"
                    ></div>
                  </div>
                </div>
              </div>

              <!-- 安全信息 -->
              <div class="security-info">
                <div class="security-items">
                  <div class="security-item">
                    <el-icon class="security-icon success">
                      <CircleCheck />
                    </el-icon>
                    <span class="security-text">IP地址验证通过</span>
                  </div>
                  <div class="security-item">
                    <el-icon class="security-icon success">
                      <CircleCheck />
                    </el-icon>
                    <span class="security-text">设备指纹匹配</span>
                  </div>
                  <div
                    v-if="session.security?.mfaEnabled"
                    class="security-item"
                  >
                    <el-icon class="security-icon success">
                      <CircleCheck />
                    </el-icon>
                    <span class="security-text">双因子认证</span>
                  </div>
                  <div
                    v-if="session.security?.risks?.length"
                    class="security-item"
                  >
                    <el-icon class="security-icon warning">
                      <Warning />
                    </el-icon>
                    <span class="security-text"
                      >检测到 {{ session.security.risks.length }} 个风险</span
                    >
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 会话历史图表 -->
      <div
        v-if="viewMode === 'history' || viewMode === 'all'"
        class="session-history"
      >
        <el-divider />
        <div class="history-header">
          <h4>会话历史趋势</h4>
          <div class="time-range-selector">
            <el-radio-group v-model="historyRange" @change="loadSessionHistory">
              <el-radio-button label="7d">7天</el-radio-button>
              <el-radio-button label="30d">30天</el-radio-button>
              <el-radio-button label="90d">90天</el-radio-button>
            </el-radio-group>
          </div>
        </div>
        <div class="history-chart">
          <div class="chart-placeholder">
            <el-icon><TrendCharts /></el-icon>
            <span>会话历史图表（模拟）</span>
          </div>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * UserSessionDialog —— 用户会话管理对话框组件
 *
 * 📝 Responsibilities:
 *  1. 显示用户的活跃和历史会话信息
 *  2. 提供会话管理功能（终止、刷新等）
 *  3. 展示会话安全状态和风险检测
 *  4. 显示会话活动图表和趋势分析
 *  5. 支持批量会话操作
 *
 * 📦 Dependencies:
 *  - Element Plus 组件库
 *  - 用户会话管理相关API
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  UserFilled,
  CircleClose,
  SwitchButton,
  Refresh,
  MoreFilled,
  Monitor,
  Connection,
  Clock,
  Timer,
  CircleCheck,
  Warning,
  TrendCharts,
  Iphone,
  Monitor as MonitorIcon,
  TabletButton,
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
const terminating = ref(false)
const viewMode = ref('active')
const historyRange = ref('7d')

// 会话数据
const sessions = ref<any[]>([])
const availableRoles = ref<any[]>([])
const departments = ref<any[]>([])

// ===== 计算属性 =====
const activeSessions = computed(() =>
  sessions.value.filter(session => session.status === 'active')
)

const totalSessions = computed(() => sessions.value.length)

const onlineTime = computed(() => {
  if (activeSessions.value.length === 0) return '离线'

  const totalMinutes = activeSessions.value.reduce((total, session) => {
    return total + session.duration
  }, 0)

  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60

  if (hours > 0) {
    return `${hours}小时${minutes}分钟`
  } else {
    return `${minutes}分钟`
  }
})

const filteredSessions = computed(() => {
  switch (viewMode.value) {
    case 'active':
      return sessions.value.filter(session => session.status === 'active')
    case 'history':
      return sessions.value.filter(session => session.status !== 'active')
    case 'all':
    default:
      return sessions.value
  }
})

// ===== 方法 =====

/**
 * 初始化数据
 */
async function initializeData() {
  try {
    // 生成模拟数据
    availableRoles.value = generateMockRoles()
    departments.value = generateMockDepartments()

    // 加载会话数据
    await loadSessions()
  } catch (error) {
    console.error('初始化用户会话对话框失败:', error)
    ElMessage.error('初始化失败')
  }
}

/**
 * 加载会话数据
 */
async function loadSessions() {
  try {
    loading.value = true

    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 500))

    sessions.value = generateMockSessions()
  } catch (error) {
    console.error('加载会话数据失败:', error)
    ElMessage.error('加载会话数据失败')
  } finally {
    loading.value = false
  }
}

/**
 * 生成模拟会话数据
 */
function generateMockSessions() {
  const mockSessions = []
  const statuses = ['active', 'expired', 'terminated']
  const deviceTypes = ['desktop', 'mobile', 'tablet']
  const browsers = ['Chrome', 'Firefox', 'Safari', 'Edge']
  const oses = ['Windows 10', 'macOS', 'iOS', 'Android', 'Linux']

  for (let i = 0; i < 8; i++) {
    const status =
      i < 2 ? 'active' : statuses[Math.floor(Math.random() * statuses.length)]
    const deviceType =
      deviceTypes[Math.floor(Math.random() * deviceTypes.length)]
    const browser = browsers[Math.floor(Math.random() * browsers.length)]
    const os = oses[Math.floor(Math.random() * oses.length)]

    const loginTime = new Date(
      Date.now() - Math.random() * 7 * 24 * 60 * 60 * 1000
    )
    const duration = Math.floor(Math.random() * 480) + 10 // 10-490 minutes
    const lastActivity = new Date(loginTime.getTime() + duration * 60 * 1000)

    mockSessions.push({
      id: `session_${i + 1}`,
      status,
      loginTime: loginTime.toISOString(),
      lastActivity: lastActivity.toISOString(),
      duration,
      ip: generateRandomIP(),
      location: generateRandomLocation(),
      device: {
        type: deviceType,
        name: `${browser} on ${os}`,
        browser,
        os,
        fingerprint: generateDeviceFingerprint(),
      },
      security: {
        mfaEnabled: Math.random() > 0.5,
        ipVerified: true,
        deviceTrusted: Math.random() > 0.3,
        risks: Math.random() > 0.7 ? generateSecurityRisks() : [],
      },
      activityChart: generateActivityChart(),
    })
  }

  return mockSessions.sort(
    (a, b) => new Date(b.loginTime).getTime() - new Date(a.loginTime).getTime()
  )
}

/**
 * 生成随机IP地址
 */
function generateRandomIP(): string {
  return `192.168.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`
}

/**
 * 生成随机位置
 */
function generateRandomLocation(): string {
  const locations = [
    '北京市',
    '上海市',
    '广州市',
    '深圳市',
    '杭州市',
    '成都市',
    '西安市',
  ]
  return locations[Math.floor(Math.random() * locations.length)]
}

/**
 * 生成设备指纹
 */
function generateDeviceFingerprint(): string {
  return Math.random().toString(36).substring(2, 15)
}

/**
 * 生成安全风险
 */
function generateSecurityRisks() {
  const risks = [
    { type: 'ip_change', message: 'IP地址发生变化' },
    { type: 'unusual_activity', message: '检测到异常活动' },
    { type: 'device_change', message: '设备指纹不匹配' },
  ]

  return risks.slice(0, Math.floor(Math.random() * 2) + 1)
}

/**
 * 生成活动图表数据
 */
function generateActivityChart() {
  const chart = []
  const now = new Date()

  for (let i = 23; i >= 0; i--) {
    const time = new Date(now.getTime() - i * 60 * 60 * 1000)
    const count = Math.floor(Math.random() * 50)
    const level = count > 30 ? 'high' : count > 15 ? 'medium' : 'low'

    chart.push({
      time: `${time.getHours().toString().padStart(2, '0')}:00`,
      count,
      value: Math.min(count * 2, 100),
      level,
    })
  }

  return chart
}

/**
 * 生成模拟角色数据
 */
function generateMockRoles() {
  return [
    { id: 'admin', name: '系统管理员' },
    { id: 'operator', name: '操作员' },
    { id: 'viewer', name: '观察员' },
    { id: 'engineer', name: '工程师' },
    { id: 'manager', name: '管理员' },
  ]
}

/**
 * 生成模拟部门数据
 */
function generateMockDepartments() {
  return [
    { id: 'it', name: 'IT部门' },
    { id: 'production', name: '生产部门' },
    { id: 'maintenance', name: '维护部门' },
    { id: 'quality', name: '质量部门' },
    { id: 'management', name: '管理部门' },
  ]
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
 * 获取设备图标
 */
function getDeviceIcon(deviceType: string): string {
  switch (deviceType) {
    case 'mobile':
      return 'Iphone'
    case 'tablet':
      return 'TabletButton'
    case 'desktop':
    default:
      return 'Monitor'
  }
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): string {
  switch (status) {
    case 'active':
      return 'success'
    case 'expired':
      return 'warning'
    case 'terminated':
      return 'danger'
    default:
      return 'info'
  }
}

/**
 * 获取状态文本
 */
function getStatusText(status: string): string {
  switch (status) {
    case 'active':
      return '活跃'
    case 'expired':
      return '已过期'
    case 'terminated':
      return '已终止'
    default:
      return '未知'
  }
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateString: string): string {
  return new Date(dateString).toLocaleString('zh-CN')
}

/**
 * 格式化持续时长
 */
function formatDuration(minutes: number): string {
  if (minutes < 60) {
    return `${minutes} 分钟`
  }

  const hours = Math.floor(minutes / 60)
  const remainingMinutes = minutes % 60

  if (remainingMinutes === 0) {
    return `${hours} 小时`
  } else {
    return `${hours} 小时 ${remainingMinutes} 分钟`
  }
}

/**
 * 格式化相对时间
 */
function formatRelativeTime(dateString: string): string {
  const now = new Date()
  const time = new Date(dateString)
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
 * 获取空状态描述
 */
function getEmptyDescription(): string {
  switch (viewMode.value) {
    case 'active':
      return '当前没有活跃会话'
    case 'history':
      return '暂无历史会话记录'
    case 'all':
      return '暂无会话记录'
    default:
      return '暂无数据'
  }
}

/**
 * 处理视图模式变化
 */
function handleViewModeChange() {
  // 视图模式变化时的处理逻辑
  console.log('视图模式变更为:', viewMode.value)
}

/**
 * 刷新会话数据
 */
function refreshSessions() {
  loadSessions()
}

/**
 * 终止所有会话
 */
function terminateAllSessions() {
  ElMessageBox.confirm(
    `确定要终止用户 "${props.userData?.name}" 的所有活跃会话吗？用户将被强制登出。`,
    '确认终止',
    {
      type: 'warning',
      confirmButtonText: '确定终止',
      cancelButtonText: '取消',
    }
  )
    .then(async () => {
      try {
        terminating.value = true

        // 模拟终止操作
        await new Promise(resolve => setTimeout(resolve, 1000))

        // 更新会话状态
        sessions.value.forEach(session => {
          if (session.status === 'active') {
            session.status = 'terminated'
          }
        })

        ElMessage.success('所有会话已终止')
      } catch (error) {
        console.error('终止会话失败:', error)
        ElMessage.error('终止会话失败')
      } finally {
        terminating.value = false
      }
    })
    .catch(() => {
      // 用户取消
    })
}

/**
 * 强制用户下线
 */
function kickOutUser() {
  ElMessageBox.confirm(
    `确定要强制用户 "${props.userData?.name}" 下线吗？此操作会立即终止所有会话并禁止短时间内重新登录。`,
    '确认强制下线',
    {
      type: 'error',
      confirmButtonText: '确定下线',
      cancelButtonText: '取消',
    }
  )
    .then(async () => {
      try {
        // 模拟强制下线操作
        await new Promise(resolve => setTimeout(resolve, 1000))

        // 更新会话状态
        sessions.value.forEach(session => {
          if (session.status === 'active') {
            session.status = 'terminated'
          }
        })

        ElMessage.success('用户已被强制下线')
      } catch (error) {
        console.error('强制下线失败:', error)
        ElMessage.error('强制下线失败')
      }
    })
    .catch(() => {
      // 用户取消
    })
}

/**
 * 处理会话操作
 */
function handleSessionAction(command: string) {
  const [action, sessionId] = command.split('_')
  const session = sessions.value.find(s => s.id === sessionId)

  if (!session) return

  switch (action) {
    case 'terminate':
      terminateSession(session)
      break
    case 'refresh':
      refreshSessionToken(session)
      break
    case 'details':
      viewSessionDetails(session)
      break
  }
}

/**
 * 终止单个会话
 */
function terminateSession(session: any) {
  ElMessageBox.confirm(
    `确定要终止会话 "${session.device.name}" 吗？`,
    '确认终止',
    {
      type: 'warning',
    }
  )
    .then(async () => {
      try {
        // 模拟终止操作
        await new Promise(resolve => setTimeout(resolve, 500))

        session.status = 'terminated'
        ElMessage.success('会话已终止')
      } catch (error) {
        console.error('终止会话失败:', error)
        ElMessage.error('终止会话失败')
      }
    })
    .catch(() => {
      // 用户取消
    })
}

/**
 * 刷新会话令牌
 */
function refreshSessionToken(session: any) {
  ElMessage.success(`会话 "${session.device.name}" 的令牌已刷新`)
}

/**
 * 查看会话详情
 */
function viewSessionDetails(session: any) {
  const securityInfo =
    session.security.risks.length > 0
      ? `<p style="color: #E6A23C;"><strong>安全风险:</strong> ${session.security.risks.map((r: any) => r.message).join(', ')}</p>`
      : '<p style="color: #67C23A;"><strong>安全状态:</strong> 正常</p>'

  ElMessageBox.alert(
    `
    <div style="text-align: left;">
      <p><strong>会话ID:</strong> ${session.id}</p>
      <p><strong>设备:</strong> ${session.device.name}</p>
      <p><strong>IP地址:</strong> ${session.ip}</p>
      <p><strong>位置:</strong> ${session.location}</p>
      <p><strong>登录时间:</strong> ${formatDateTime(session.loginTime)}</p>
      <p><strong>持续时长:</strong> ${formatDuration(session.duration)}</p>
      <p><strong>状态:</strong> ${getStatusText(session.status)}</p>
      ${securityInfo}
    </div>
    `,
    '会话详情',
    {
      dangerouslyUseHTMLString: true,
      confirmButtonText: '关闭',
    }
  )
}

/**
 * 加载会话历史
 */
function loadSessionHistory() {
  console.log('加载会话历史:', historyRange.value)
  // 这里可以根据时间范围加载不同的历史数据
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
.user-session-dialog {
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

    .session-stats {
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

  .session-controls {
    margin: 20px 0;

    .control-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 16px;

      .session-actions {
        display: flex;
        gap: 8px;
      }

      .view-controls {
        display: flex;
        align-items: center;
        gap: 12px;
      }
    }
  }

  .sessions-container {
    margin: 20px 0;
    min-height: 400px;

    .empty-state {
      text-align: center;
      padding: 60px 0;
    }

    .sessions-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(450px, 1fr));
      gap: 16px;

      .session-card {
        border: 1px solid #ebeef5;
        border-radius: 8px;
        padding: 16px;
        transition: all 0.2s;

        &:hover {
          border-color: #409eff;
          box-shadow: 0 2px 12px rgba(64, 158, 255, 0.1);
        }

        &.active {
          border-color: #67c23a;
          background: #f0f9ff;
        }

        &.expired {
          border-color: #e6a23c;
          background: #fdf6ec;
        }

        &.terminated {
          border-color: #f56c6c;
          background: #fef0f0;
          opacity: 0.8;
        }

        .session-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 16px;

          .session-info {
            flex: 1;

            .session-title {
              display: flex;
              align-items: center;
              gap: 8px;
              margin-bottom: 4px;

              .device-icon {
                font-size: 16px;
                color: #409eff;
              }

              .session-name {
                font-weight: 500;
                color: #303133;
              }

              .session-status {
                margin: 0;
              }
            }

            .session-meta {
              display: flex;
              flex-direction: column;
              gap: 2px;
              font-size: 12px;
              color: #909399;

              .session-id {
                font-family: monospace;
              }
            }
          }

          .session-actions {
            flex-shrink: 0;
          }
        }

        .session-details {
          .detail-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 12px;
            margin-bottom: 16px;

            .detail-item {
              display: flex;
              align-items: center;
              gap: 8px;

              .detail-icon {
                font-size: 14px;
                color: #409eff;
                flex-shrink: 0;
              }

              .detail-content {
                flex: 1;
                min-width: 0;

                .detail-label {
                  font-size: 11px;
                  color: #909399;
                  line-height: 1.2;
                }

                .detail-value {
                  font-size: 13px;
                  color: #303133;
                  font-weight: 500;
                  line-height: 1.3;
                  word-break: break-all;
                }
              }
            }
          }

          .session-activity {
            background: #f8f9fa;
            border-radius: 6px;
            padding: 12px;
            margin-bottom: 12px;

            .activity-header {
              display: flex;
              justify-content: space-between;
              align-items: center;
              margin-bottom: 8px;

              .activity-title {
                font-size: 12px;
                font-weight: 500;
                color: #606266;
              }

              .last-activity {
                font-size: 11px;
                color: #909399;
              }
            }

            .activity-chart {
              .activity-bars {
                display: flex;
                gap: 1px;
                height: 30px;
                align-items: flex-end;

                .activity-bar {
                  flex: 1;
                  min-height: 2px;
                  border-radius: 1px;
                  transition: all 0.2s;

                  &.high {
                    background: #f56c6c;
                  }

                  &.medium {
                    background: #e6a23c;
                  }

                  &.low {
                    background: #67c23a;
                  }

                  &:hover {
                    opacity: 0.8;
                    transform: scaleY(1.1);
                  }
                }
              }
            }
          }

          .security-info {
            .security-items {
              display: flex;
              flex-direction: column;
              gap: 6px;

              .security-item {
                display: flex;
                align-items: center;
                gap: 6px;

                .security-icon {
                  font-size: 14px;

                  &.success {
                    color: #67c23a;
                  }

                  &.warning {
                    color: #e6a23c;
                  }
                }

                .security-text {
                  font-size: 12px;
                  color: #606266;
                }
              }
            }
          }
        }
      }
    }
  }

  .session-history {
    margin-top: 20px;

    .history-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;

      h4 {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: #303133;
      }
    }

    .history-chart {
      height: 200px;
      border: 1px solid #ebeef5;
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: #fafafa;

      .chart-placeholder {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        color: #909399;

        .el-icon {
          font-size: 32px;
        }

        span {
          font-size: 14px;
        }
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .user-session-dialog {
    .user-info-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;

      .session-stats {
        width: 100%;
        justify-content: space-around;
      }
    }

    .session-controls .control-row {
      flex-direction: column;
      align-items: stretch;
      gap: 12px;

      .session-actions {
        flex-wrap: wrap;
        justify-content: center;
      }

      .view-controls {
        justify-content: center;
      }
    }

    .sessions-container .sessions-grid {
      grid-template-columns: 1fr;

      .session-card .session-details .detail-grid {
        grid-template-columns: 1fr;
        gap: 8px;
      }
    }
  }
}
</style>
