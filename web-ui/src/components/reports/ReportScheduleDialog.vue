<template>
  <el-dialog
    v-model="visible"
    title="定时生成报表"
    width="700px"
    :close-on-click-modal="false"
  >
    <div class="report-schedule">
      <!-- 基础设置 -->
      <el-card class="schedule-section" shadow="never">
        <template #header>
          <span>基础设置</span>
        </template>

        <el-form :model="scheduleConfig" label-width="120px">
          <el-form-item label="任务名称">
            <el-input
              v-model="scheduleConfig.name"
              placeholder="请输入任务名称"
              maxlength="50"
              show-word-limit
            />
          </el-form-item>

          <el-form-item label="任务状态">
            <el-switch
              v-model="scheduleConfig.enabled"
              active-text="启用"
              inactive-text="禁用"
            />
          </el-form-item>

          <el-form-item label="报表描述">
            <el-input
              v-model="scheduleConfig.description"
              type="textarea"
              :rows="3"
              placeholder="报表描述（可选）"
              maxlength="200"
              show-word-limit
            />
          </el-form-item>
        </el-form>
      </el-card>

      <!-- 时间调度 -->
      <el-card class="schedule-section" shadow="never">
        <template #header>
          <span>时间调度</span>
        </template>

        <el-form :model="scheduleConfig" label-width="120px">
          <el-form-item label="调度频率">
            <el-radio-group
              v-model="scheduleConfig.frequency"
              @change="handleFrequencyChange"
            >
              <el-radio label="once">一次性</el-radio>
              <el-radio label="daily">每日</el-radio>
              <el-radio label="weekly">每周</el-radio>
              <el-radio label="monthly">每月</el-radio>
              <el-radio label="custom">自定义</el-radio>
            </el-radio-group>
          </el-form-item>

          <!-- 一次性任务 -->
          <el-form-item
            v-if="scheduleConfig.frequency === 'once'"
            label="执行时间"
          >
            <el-date-picker
              v-model="scheduleConfig.executeTime"
              type="datetime"
              placeholder="选择执行时间"
              format="YYYY-MM-DD HH:mm:ss"
              value-format="YYYY-MM-DD HH:mm:ss"
            />
          </el-form-item>

          <!-- 每日任务 -->
          <el-form-item
            v-if="scheduleConfig.frequency === 'daily'"
            label="执行时间"
          >
            <el-time-picker
              v-model="scheduleConfig.dailyTime"
              placeholder="选择时间"
              format="HH:mm"
              value-format="HH:mm"
            />
          </el-form-item>

          <!-- 每周任务 -->
          <template v-if="scheduleConfig.frequency === 'weekly'">
            <el-form-item label="星期">
              <el-checkbox-group v-model="scheduleConfig.weekDays">
                <el-checkbox label="1">周一</el-checkbox>
                <el-checkbox label="2">周二</el-checkbox>
                <el-checkbox label="3">周三</el-checkbox>
                <el-checkbox label="4">周四</el-checkbox>
                <el-checkbox label="5">周五</el-checkbox>
                <el-checkbox label="6">周六</el-checkbox>
                <el-checkbox label="0">周日</el-checkbox>
              </el-checkbox-group>
            </el-form-item>
            <el-form-item label="执行时间">
              <el-time-picker
                v-model="scheduleConfig.weeklyTime"
                placeholder="选择时间"
                format="HH:mm"
                value-format="HH:mm"
              />
            </el-form-item>
          </template>

          <!-- 每月任务 -->
          <template v-if="scheduleConfig.frequency === 'monthly'">
            <el-form-item label="执行日期">
              <el-radio-group v-model="scheduleConfig.monthlyType">
                <el-radio label="date">指定日期</el-radio>
                <el-radio label="last">月末</el-radio>
              </el-radio-group>
            </el-form-item>
            <el-form-item
              v-if="scheduleConfig.monthlyType === 'date'"
              label="日期"
            >
              <el-select
                v-model="scheduleConfig.monthlyDate"
                placeholder="选择日期"
              >
                <el-option
                  v-for="day in 31"
                  :key="day"
                  :label="`${day}号`"
                  :value="day"
                />
              </el-select>
            </el-form-item>
            <el-form-item label="执行时间">
              <el-time-picker
                v-model="scheduleConfig.monthlyTime"
                placeholder="选择时间"
                format="HH:mm"
                value-format="HH:mm"
              />
            </el-form-item>
          </template>

          <!-- 自定义Cron表达式 -->
          <el-form-item
            v-if="scheduleConfig.frequency === 'custom'"
            label="Cron表达式"
          >
            <el-input
              v-model="scheduleConfig.cronExpression"
              placeholder="例如: 0 9 * * 1-5 (工作日每天9点)"
            >
              <template #append>
                <el-button @click="showCronHelper = true">帮助</el-button>
              </template>
            </el-input>
            <div class="cron-preview">
              <span class="cron-text">{{
                getCronDescription(scheduleConfig.cronExpression)
              }}</span>
            </div>
          </el-form-item>

          <!-- 有效期设置 -->
          <el-form-item label="有效期">
            <el-radio-group v-model="scheduleConfig.validityType">
              <el-radio label="forever">永久有效</el-radio>
              <el-radio label="until">到期失效</el-radio>
              <el-radio label="count">指定次数</el-radio>
            </el-radio-group>
          </el-form-item>

          <el-form-item
            v-if="scheduleConfig.validityType === 'until'"
            label="失效时间"
          >
            <el-date-picker
              v-model="scheduleConfig.validUntil"
              type="datetime"
              placeholder="选择失效时间"
              format="YYYY-MM-DD HH:mm:ss"
              value-format="YYYY-MM-DD HH:mm:ss"
            />
          </el-form-item>

          <el-form-item
            v-if="scheduleConfig.validityType === 'count'"
            label="执行次数"
          >
            <el-input-number
              v-model="scheduleConfig.maxExecutions"
              :min="1"
              :max="1000"
              placeholder="最大执行次数"
            />
          </el-form-item>
        </el-form>
      </el-card>

      <!-- 报表配置 -->
      <el-card class="schedule-section" shadow="never">
        <template #header>
          <span>报表配置</span>
        </template>

        <el-form :model="scheduleConfig" label-width="120px">
          <el-form-item label="数据范围">
            <el-select
              v-model="scheduleConfig.dataRange"
              placeholder="选择数据范围"
            >
              <el-option label="最近1小时" value="1h" />
              <el-option label="最近24小时" value="24h" />
              <el-option label="最近7天" value="7d" />
              <el-option label="最近30天" value="30d" />
              <el-option label="上个月" value="last_month" />
              <el-option label="自定义" value="custom" />
            </el-select>
          </el-form-item>

          <el-form-item label="导出格式">
            <el-checkbox-group v-model="scheduleConfig.formats">
              <el-checkbox label="pdf" border>PDF</el-checkbox>
              <el-checkbox label="excel" border>Excel</el-checkbox>
              <el-checkbox label="csv" border>CSV</el-checkbox>
            </el-checkbox-group>
          </el-form-item>

          <el-form-item label="自动发送">
            <el-switch v-model="scheduleConfig.autoSend" />
          </el-form-item>

          <el-form-item v-if="scheduleConfig.autoSend" label="收件人">
            <el-select
              v-model="scheduleConfig.recipients"
              multiple
              filterable
              allow-create
              placeholder="输入邮箱地址"
              style="width: 100%"
            >
              <el-option
                v-for="contact in emailContacts"
                :key="contact.email"
                :label="`${contact.name} <${contact.email}>`"
                :value="contact.email"
              />
            </el-select>
          </el-form-item>
        </el-form>
      </el-card>

      <!-- 任务列表 -->
      <el-card class="schedule-section" shadow="never">
        <template #header>
          <span>已创建的定时任务</span>
        </template>

        <el-table :data="existingTasks" style="width: 100%">
          <el-table-column prop="name" label="任务名称" />
          <el-table-column prop="frequency" label="频率" width="100" />
          <el-table-column prop="nextRun" label="下次执行" width="150" />
          <el-table-column prop="status" label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="getTaskStatusType(row.status)" size="small">
                {{ row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="150">
            <template #default="{ row }">
              <el-button size="small" text @click="editTask(row)"
                >编辑</el-button
              >
              <el-button
                size="small"
                text
                :type="row.status === '已启用' ? 'warning' : 'success'"
                @click="toggleTask(row)"
              >
                {{ row.status === '已启用' ? '禁用' : '启用' }}
              </el-button>
              <el-button
                size="small"
                text
                type="danger"
                @click="deleteTask(row)"
              >
                删除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>

    <!-- 对话框底部 -->
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleCancel">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">
          {{ saving ? '保存中...' : '保存任务' }}
        </el-button>
      </div>
    </template>

    <!-- Cron帮助对话框 -->
    <el-dialog v-model="showCronHelper" title="Cron表达式帮助" width="600px">
      <div class="cron-help">
        <h4>格式说明</h4>
        <p>Cron表达式由5个字段组成：分钟 小时 日 月 星期</p>

        <h4>特殊字符</h4>
        <ul>
          <li><code>*</code> - 匹配任意值</li>
          <li><code>?</code> - 匹配任意值（仅用于日和星期）</li>
          <li><code>-</code> - 范围，如 1-5</li>
          <li><code>,</code> - 列表，如 1,3,5</li>
          <li><code>/</code> - 间隔，如 */5 表示每5分钟</li>
        </ul>

        <h4>常用示例</h4>
        <el-table :data="cronExamples" style="width: 100%">
          <el-table-column prop="expression" label="表达式" width="150" />
          <el-table-column prop="description" label="说明" />
          <el-table-column label="操作" width="80">
            <template #default="{ row }">
              <el-button
                size="small"
                text
                @click="useCronExample(row.expression)"
              >
                使用
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-dialog>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * ReportScheduleDialog —— 报表定时生成对话框
 *
 * 📝 Responsibilities:
 *  1. 定时任务创建和配置
 *  2. 多种调度频率支持
 *  3. Cron表达式设置
 *  4. 任务管理和监控
 *
 * 📦 Dependencies:
 *  - Element Plus UI组件
 *  - Cron表达式解析
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, computed } from 'vue'

// ===== Props =====
const props = defineProps<{
  modelValue: boolean
  report?: any
}>()

// ===== Emits =====
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

// ===== 响应式数据 =====
const visible = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})

const saving = ref(false)
const showCronHelper = ref(false)

// 调度配置
const scheduleConfig = ref({
  name: '',
  description: '',
  enabled: true,
  frequency: 'daily',
  executeTime: '',
  dailyTime: '09:00',
  weekDays: ['1', '2', '3', '4', '5'],
  weeklyTime: '09:00',
  monthlyType: 'date',
  monthlyDate: 1,
  monthlyTime: '09:00',
  cronExpression: '0 9 * * 1-5',
  validityType: 'forever',
  validUntil: '',
  maxExecutions: 30,
  dataRange: '24h',
  formats: ['pdf'],
  autoSend: false,
  recipients: [],
})

// 邮件联系人
const emailContacts = ref([
  { name: '张三', email: 'zhangsan@example.com' },
  { name: '李四', email: 'lisi@example.com' },
  { name: '王五', email: 'wangwu@example.com' },
])

// 现有任务
const existingTasks = ref([
  {
    id: '1',
    name: '设备状态日报',
    frequency: '每日',
    nextRun: '2025-07-28 09:00',
    status: '已启用',
  },
  {
    id: '2',
    name: '性能分析周报',
    frequency: '每周',
    nextRun: '2025-07-29 10:00',
    status: '已启用',
  },
  {
    id: '3',
    name: '数据质量月报',
    frequency: '每月',
    nextRun: '2025-08-01 08:00',
    status: '已禁用',
  },
])

// Cron示例
const cronExamples = ref([
  { expression: '0 9 * * 1-5', description: '工作日每天9点' },
  { expression: '0 */2 * * *', description: '每2小时' },
  { expression: '0 0 1 * *', description: '每月1号零点' },
  { expression: '0 0 * * 0', description: '每周日零点' },
  { expression: '30 8 * * 1-5', description: '工作日每天8:30' },
  { expression: '0 12 1,15 * *', description: '每月1号和15号中午12点' },
])

// ===== 方法 =====

/**
 * 处理频率变化
 */
function handleFrequencyChange(frequency: string) {
  // 根据频率设置默认的cron表达式
  const cronMap: { [key: string]: string } = {
    daily: '0 9 * * *',
    weekly: '0 9 * * 1',
    monthly: '0 9 1 * *',
  }

  if (cronMap[frequency]) {
    scheduleConfig.value.cronExpression = cronMap[frequency]
  }
}

/**
 * 获取Cron描述
 */
function getCronDescription(cron: string): string {
  if (!cron) return '无效的表达式'

  // 简单的cron表达式解析（实际项目中可使用专门的库）
  const descriptions: { [key: string]: string } = {
    '0 9 * * *': '每天上午9点',
    '0 9 * * 1-5': '工作日每天上午9点',
    '0 */2 * * *': '每2小时',
    '0 0 1 * *': '每月1号零点',
    '0 0 * * 0': '每周日零点',
    '30 8 * * 1-5': '工作日每天上午8:30',
    '0 12 1,15 * *': '每月1号和15号中午12点',
  }

  return descriptions[cron] || '自定义表达式'
}

/**
 * 使用Cron示例
 */
function useCronExample(expression: string) {
  scheduleConfig.value.cronExpression = expression
  showCronHelper.value = false
  ElMessage.success('已应用示例表达式')
}

/**
 * 获取任务状态类型
 */
function getTaskStatusType(status: string): string {
  if (status === '已启用') return 'success'
  if (status === '已禁用') return 'warning'
  if (status === '已完成') return 'info'
  if (status === '失败') return 'danger'
  return 'info'
}

/**
 * 编辑任务
 */
function editTask(task: any) {
  // 加载任务配置到表单
  ElMessage.info(`编辑任务: ${task.name}`)
}

/**
 * 切换任务状态
 */
function toggleTask(task: any) {
  const newStatus = task.status === '已启用' ? '已禁用' : '已启用'
  task.status = newStatus
  ElMessage.success(`任务已${newStatus}`)
}

/**
 * 删除任务
 */
function deleteTask(task: any) {
  ElMessageBox.confirm(
    `确定要删除任务 "${task.name}" 吗？此操作不可恢复。`,
    '删除任务',
    {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    }
  )
    .then(() => {
      const index = existingTasks.value.findIndex(t => t.id === task.id)
      if (index !== -1) {
        existingTasks.value.splice(index, 1)
        ElMessage.success('任务已删除')
      }
    })
    .catch(() => {
      // 取消操作
    })
}

/**
 * 保存任务
 */
async function handleSave() {
  // 验证表单
  if (!scheduleConfig.value.name) {
    ElMessage.warning('请输入任务名称')
    return
  }

  if (
    scheduleConfig.value.frequency === 'once' &&
    !scheduleConfig.value.executeTime
  ) {
    ElMessage.warning('请选择执行时间')
    return
  }

  if (
    scheduleConfig.value.frequency === 'weekly' &&
    !scheduleConfig.value.weekDays.length
  ) {
    ElMessage.warning('请选择星期')
    return
  }

  if (
    scheduleConfig.value.autoSend &&
    !scheduleConfig.value.recipients.length
  ) {
    ElMessage.warning('启用自动发送时请至少选择一个收件人')
    return
  }

  saving.value = true

  try {
    // 模拟保存
    await new Promise(resolve => setTimeout(resolve, 1500))

    // 添加到任务列表
    const newTask = {
      id: Date.now().toString(),
      name: scheduleConfig.value.name,
      frequency: getFrequencyText(scheduleConfig.value.frequency),
      nextRun: calculateNextRun(),
      status: scheduleConfig.value.enabled ? '已启用' : '已禁用',
    }

    existingTasks.value.unshift(newTask)

    ElMessage.success('定时任务创建成功')
    visible.value = false
  } catch (error) {
    console.error('保存失败:', error)
    ElMessage.error('保存失败')
  } finally {
    saving.value = false
  }
}

/**
 * 获取频率文本
 */
function getFrequencyText(frequency: string): string {
  const textMap: { [key: string]: string } = {
    once: '一次性',
    daily: '每日',
    weekly: '每周',
    monthly: '每月',
    custom: '自定义',
  }
  return textMap[frequency] || frequency
}

/**
 * 计算下次执行时间
 */
function calculateNextRun(): string {
  const now = new Date()
  const nextRun = new Date(now)

  if (scheduleConfig.value.frequency === 'daily') {
    const [hour, minute] = scheduleConfig.value.dailyTime.split(':')
    nextRun.setHours(parseInt(hour), parseInt(minute), 0, 0)

    if (nextRun <= now) {
      nextRun.setDate(nextRun.getDate() + 1)
    }
  } else if (scheduleConfig.value.frequency === 'weekly') {
    // 简化处理，设置为明天
    nextRun.setDate(nextRun.getDate() + 1)
    const [hour, minute] = scheduleConfig.value.weeklyTime.split(':')
    nextRun.setHours(parseInt(hour), parseInt(minute), 0, 0)
  } else {
    // 默认设置为明天
    nextRun.setDate(nextRun.getDate() + 1)
    nextRun.setHours(9, 0, 0, 0)
  }

  return nextRun.toLocaleString('zh-CN').slice(0, 16)
}

/**
 * 处理取消
 */
function handleCancel() {
  visible.value = false
}

// ===== 初始化 =====
if (props.report) {
  scheduleConfig.value.name = `${props.report.name} - 定时任务`
  scheduleConfig.value.description = `基于报表 "${props.report.name}" 的定时生成任务`
}
</script>

<style scoped lang="scss">
.report-schedule {
  .schedule-section {
    margin-bottom: 20px;

    :deep(.el-card__header) {
      padding: 12px 20px;
      background: #fafafa;

      span {
        font-weight: 600;
        color: #303133;
      }
    }

    :deep(.el-card__body) {
      padding: 20px;
    }
  }

  .cron-preview {
    margin-top: 8px;

    .cron-text {
      font-size: 12px;
      color: #67c23a;
      font-style: italic;
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.cron-help {
  h4 {
    margin: 16px 0 8px 0;
    color: #303133;
    font-size: 14px;
  }

  p {
    margin: 8px 0;
    color: #606266;
    line-height: 1.5;
  }

  ul {
    margin: 8px 0;
    padding-left: 20px;

    li {
      margin-bottom: 4px;
      color: #606266;
      line-height: 1.5;

      code {
        background: #f4f4f5;
        padding: 2px 4px;
        border-radius: 3px;
        font-family: monospace;
        color: #e6a23c;
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .report-schedule {
    .schedule-section {
      :deep(.el-form-item__label) {
        width: 100px !important;
      }
    }
  }
}
</style>
