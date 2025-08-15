<template>
  <div class="alert-notification-settings">
    <!-- 通知渠道配置 -->
    <div class="notification-channels">
      <div class="section-header">
        <h4 class="section-title">通知渠道</h4>
        <el-button type="primary" size="small" @click="addNotificationChannel">
          <el-icon><Plus /></el-icon>
          添加渠道
        </el-button>
      </div>

      <div v-if="notifications.length === 0" class="empty-state">
        <el-empty description="暂未配置通知渠道" :image-size="80">
          <el-button type="primary" @click="addNotificationChannel">
            添加第一个通知渠道
          </el-button>
        </el-empty>
      </div>

      <div v-else class="channels-list">
        <div
          v-for="(notification, index) in notifications"
          :key="index"
          class="channel-item"
        >
          <div class="channel-header">
            <div class="channel-info">
              <el-icon class="channel-icon">
                <Message v-if="notification.type === 'email'" />
                <ChatDotRound v-else-if="notification.type === 'sms'" />
                <Connection v-else-if="notification.type === 'webhook'" />
                <Bell v-else-if="notification.type === 'system'" />
                <ChatLineRound v-else-if="notification.type === 'dingtalk'" />
                <Comment v-else />
              </el-icon>
              <span class="channel-name">{{
                getChannelTypeName(notification.type)
              }}</span>
              <el-tag
                :type="notification.enabled ? 'success' : 'warning'"
                size="small"
              >
                {{ notification.enabled ? '已启用' : '已禁用' }}
              </el-tag>
            </div>

            <div class="channel-actions">
              <el-switch
                v-model="notification.enabled"
                @change="handleChannelEnabledChange(index)"
              />
              <el-button
                type="text"
                :disabled="notifications.length <= 1"
                @click="removeNotificationChannel(index)"
              >
                <el-icon><Delete /></el-icon>
              </el-button>
            </div>
          </div>

          <div v-if="notification.enabled" class="channel-config">
            <!-- 邮件配置 -->
            <div v-if="notification.type === 'email'" class="config-section">
              <div class="config-row">
                <div class="config-item">
                  <label class="config-label">收件人</label>
                  <el-select
                    v-model="notification.config.recipients"
                    multiple
                    filterable
                    allow-create
                    placeholder="输入邮箱地址"
                    style="width: 100%"
                  >
                    <el-option
                      v-for="email in commonEmails"
                      :key="email"
                      :label="email"
                      :value="email"
                    />
                  </el-select>
                </div>

                <div class="config-item">
                  <label class="config-label">邮件模板</label>
                  <el-select
                    v-model="notification.config.template"
                    placeholder="选择邮件模板"
                  >
                    <el-option label="默认模板" value="default" />
                    <el-option label="简洁模板" value="simple" />
                    <el-option label="详细模板" value="detailed" />
                  </el-select>
                </div>
              </div>

              <div class="config-row">
                <div class="config-item">
                  <el-checkbox v-model="notification.config.includeSnapshot">
                    包含数据快照
                  </el-checkbox>
                </div>
                <div class="config-item">
                  <el-checkbox v-model="notification.config.htmlFormat">
                    使用HTML格式
                  </el-checkbox>
                </div>
              </div>
            </div>

            <!-- 短信配置 -->
            <div v-else-if="notification.type === 'sms'" class="config-section">
              <div class="config-row">
                <div class="config-item">
                  <label class="config-label">手机号码</label>
                  <el-select
                    v-model="notification.config.phoneNumbers"
                    multiple
                    filterable
                    allow-create
                    placeholder="输入手机号码"
                    style="width: 100%"
                  >
                    <el-option
                      v-for="phone in commonPhones"
                      :key="phone"
                      :label="phone"
                      :value="phone"
                    />
                  </el-select>
                </div>

                <div class="config-item">
                  <label class="config-label">短信模板</label>
                  <el-select
                    v-model="notification.config.template"
                    placeholder="选择短信模板"
                  >
                    <el-option label="报警通知" value="alert" />
                    <el-option label="故障通知" value="fault" />
                    <el-option label="恢复通知" value="recovery" />
                  </el-select>
                </div>
              </div>
            </div>

            <!-- Webhook配置 -->
            <div
              v-else-if="notification.type === 'webhook'"
              class="config-section"
            >
              <div class="config-row">
                <div class="config-item">
                  <label class="config-label">Webhook URL</label>
                  <el-input
                    v-model="notification.config.url"
                    placeholder="https://example.com/webhook"
                  />
                </div>

                <div class="config-item">
                  <label class="config-label">HTTP方法</label>
                  <el-select
                    v-model="notification.config.method"
                    placeholder="选择HTTP方法"
                  >
                    <el-option label="POST" value="POST" />
                    <el-option label="PUT" value="PUT" />
                    <el-option label="PATCH" value="PATCH" />
                  </el-select>
                </div>
              </div>

              <div class="config-row">
                <div class="config-item full-width">
                  <label class="config-label">请求头</label>
                  <div class="headers-config">
                    <div
                      v-for="(header, headerIndex) in notification.config
                        .headers"
                      :key="headerIndex"
                      class="header-item"
                    >
                      <el-input
                        v-model="header.key"
                        placeholder="Header Name"
                        style="width: 200px"
                      />
                      <el-input
                        v-model="header.value"
                        placeholder="Header Value"
                        style="width: 300px"
                      />
                      <el-button
                        type="text"
                        :disabled="notification.config.headers.length <= 1"
                        @click="removeHeader(index, headerIndex)"
                      >
                        <el-icon><Delete /></el-icon>
                      </el-button>
                    </div>
                    <el-button type="text" @click="addHeader(index)">
                      <el-icon><Plus /></el-icon>
                      添加请求头
                    </el-button>
                  </div>
                </div>
              </div>

              <div class="config-row">
                <div class="config-item">
                  <el-checkbox v-model="notification.config.verifySSL">
                    验证SSL证书
                  </el-checkbox>
                </div>
                <div class="config-item">
                  <label class="config-label">超时时间</label>
                  <div class="timeout-input">
                    <el-input-number
                      v-model="notification.config.timeout"
                      :min="1"
                      :max="300"
                      placeholder="秒"
                    />
                    <span>秒</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 钉钉配置 -->
            <div
              v-else-if="notification.type === 'dingtalk'"
              class="config-section"
            >
              <div class="config-row">
                <div class="config-item">
                  <label class="config-label">机器人Token</label>
                  <el-input
                    v-model="notification.config.token"
                    placeholder="钉钉机器人Token"
                    show-password
                  />
                </div>

                <div class="config-item">
                  <label class="config-label">消息类型</label>
                  <el-select
                    v-model="notification.config.messageType"
                    placeholder="选择消息类型"
                  >
                    <el-option label="文本消息" value="text" />
                    <el-option label="Markdown" value="markdown" />
                    <el-option label="卡片消息" value="actionCard" />
                  </el-select>
                </div>
              </div>

              <div class="config-row">
                <div class="config-item">
                  <label class="config-label">@手机号</label>
                  <el-input
                    v-model="notification.config.atMobiles"
                    placeholder="多个号码用逗号分隔"
                  />
                </div>
                <div class="config-item">
                  <el-checkbox v-model="notification.config.atAll">
                    @所有人
                  </el-checkbox>
                </div>
              </div>
            </div>

            <!-- 系统通知配置 -->
            <div
              v-else-if="notification.type === 'system'"
              class="config-section"
            >
              <div class="config-row">
                <div class="config-item">
                  <label class="config-label">通知级别</label>
                  <el-select
                    v-model="notification.config.level"
                    placeholder="选择通知级别"
                  >
                    <el-option label="信息" value="info" />
                    <el-option label="警告" value="warning" />
                    <el-option label="错误" value="error" />
                  </el-select>
                </div>

                <div class="config-item">
                  <label class="config-label">持续时间</label>
                  <div class="duration-input">
                    <el-input-number
                      v-model="notification.config.duration"
                      :min="1"
                      :max="300"
                      placeholder="秒"
                    />
                    <span>秒</span>
                  </div>
                </div>
              </div>

              <div class="config-row">
                <div class="config-item">
                  <el-checkbox v-model="notification.config.showDesktop">
                    桌面通知
                  </el-checkbox>
                </div>
                <div class="config-item">
                  <el-checkbox v-model="notification.config.playSound">
                    播放提示音
                  </el-checkbox>
                </div>
              </div>
            </div>

            <!-- 通知条件 -->
            <div class="notification-conditions">
              <h5 class="condition-title">通知条件</h5>
              <div class="condition-config">
                <div class="condition-row">
                  <div class="condition-item">
                    <label class="condition-label">触发级别</label>
                    <el-select
                      v-model="notification.conditions.severityLevels"
                      multiple
                      placeholder="选择要通知的严重级别"
                    >
                      <el-option label="信息" value="info" />
                      <el-option label="警告" value="warning" />
                      <el-option label="严重" value="critical" />
                    </el-select>
                  </div>

                  <div class="condition-item">
                    <label class="condition-label">通知时间</label>
                    <el-select
                      v-model="notification.conditions.timeRange"
                      placeholder="选择通知时间"
                    >
                      <el-option label="24小时" value="always" />
                      <el-option label="工作时间" value="business" />
                      <el-option label="自定义" value="custom" />
                    </el-select>
                  </div>
                </div>

                <div
                  v-if="notification.conditions.timeRange === 'custom'"
                  class="condition-row"
                >
                  <div class="condition-item">
                    <label class="condition-label">开始时间</label>
                    <el-time-picker
                      v-model="notification.conditions.startTime"
                      placeholder="选择开始时间"
                      format="HH:mm"
                      value-format="HH:mm"
                    />
                  </div>

                  <div class="condition-item">
                    <label class="condition-label">结束时间</label>
                    <el-time-picker
                      v-model="notification.conditions.endTime"
                      placeholder="选择结束时间"
                      format="HH:mm"
                      value-format="HH:mm"
                    />
                  </div>
                </div>

                <div class="condition-row">
                  <div class="condition-item">
                    <el-checkbox v-model="notification.conditions.onTrigger">
                      报警触发时通知
                    </el-checkbox>
                  </div>
                  <div class="condition-item">
                    <el-checkbox v-model="notification.conditions.onRecovery">
                      报警恢复时通知
                    </el-checkbox>
                  </div>
                </div>

                <div class="condition-row">
                  <div class="condition-item">
                    <label class="condition-label">重复通知间隔</label>
                    <div class="interval-input">
                      <el-input-number
                        v-model="notification.conditions.repeatInterval"
                        :min="0"
                        :max="1440"
                        placeholder="分钟"
                      />
                      <span>分钟 (0表示不重复)</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 测试按钮 -->
            <div class="channel-test">
              <el-button
                type="primary"
                size="small"
                :loading="testingChannels[index]"
                @click="testNotification(index)"
              >
                <el-icon><VideoPlay /></el-icon>
                测试通知
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 高级设置 -->
    <div class="advanced-settings">
      <h4 class="section-title">高级设置</h4>

      <div class="settings-grid">
        <div class="setting-item">
          <label class="setting-label">通知去重</label>
          <el-switch v-model="advancedSettings.deduplication" />
          <div class="setting-tip">相同内容的通知在指定时间内只发送一次</div>
        </div>

        <div class="setting-item">
          <label class="setting-label">通知限流</label>
          <el-switch v-model="advancedSettings.rateLimit" />
          <div class="setting-tip">限制通知发送频率，防止通知风暴</div>
        </div>

        <div v-if="advancedSettings.rateLimit" class="setting-item">
          <label class="setting-label">限流阈值</label>
          <div class="rate-limit-config">
            <span>每</span>
            <el-input-number
              v-model="advancedSettings.rateLimitWindow"
              :min="1"
              :max="60"
              style="width: 80px"
            />
            <span>分钟最多</span>
            <el-input-number
              v-model="advancedSettings.rateLimitCount"
              :min="1"
              :max="1000"
              style="width: 80px"
            />
            <span>条通知</span>
          </div>
        </div>

        <div class="setting-item">
          <label class="setting-label">故障转移</label>
          <el-switch v-model="advancedSettings.failover" />
          <div class="setting-tip">当主要通知渠道失败时，使用备用渠道</div>
        </div>
      </div>
    </div>

    <!-- 添加通知渠道对话框 -->
    <el-dialog
      v-model="showAddChannelDialog"
      title="添加通知渠道"
      width="500px"
      :close-on-click-modal="false"
    >
      <div class="channel-types">
        <div
          v-for="channelType in availableChannelTypes"
          :key="channelType.value"
          class="channel-type-item"
          :class="{ selected: selectedChannelType === channelType.value }"
          @click="selectChannelType(channelType.value)"
        >
          <el-icon class="type-icon">
            <component :is="channelType.icon" />
          </el-icon>
          <div class="type-info">
            <div class="type-name">{{ channelType.label }}</div>
            <div class="type-desc">{{ channelType.description }}</div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="showAddChannelDialog = false">取消</el-button>
          <el-button
            type="primary"
            :disabled="!selectedChannelType"
            @click="confirmAddChannel"
          >
            添加
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * AlertNotificationSettings —— 报警通知设置组件
 *
 * 📝 Responsibilities:
 *  1. 配置多种通知渠道（邮件、短信、Webhook、钉钉等）
 *  2. 设置通知条件和时间范围
 *  3. 配置高级通知功能（去重、限流、故障转移）
 *  4. 提供通知测试功能
 *
 * 📦 Dependencies:
 *  - Element Plus 组件库
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import {
  Plus,
  Delete,
  Message,
  ChatDotRound,
  Connection,
  Bell,
  ChatLineRound,
  Comment,
  VideoPlay,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, watch } from 'vue'

// ===== Props & Emits =====
const props = defineProps<{
  modelValue: any[]
  severity: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: any[]]
}>()

// ===== 响应式数据 =====
const notifications = ref(props.modelValue || [])
const showAddChannelDialog = ref(false)
const selectedChannelType = ref('')
const testingChannels = ref<{ [key: number]: boolean }>({})

// 高级设置
const advancedSettings = ref({
  deduplication: true,
  rateLimit: false,
  rateLimitWindow: 5,
  rateLimitCount: 10,
  failover: false,
})

// 从后端API加载的常用联系方式
const commonEmails = ref<string[]>([])
const commonPhones = ref<string[]>([])

// 可用的通知渠道类型
const availableChannelTypes = [
  {
    value: 'email',
    label: '邮件通知',
    description: '发送邮件到指定邮箱',
    icon: 'Message',
  },
  {
    value: 'sms',
    label: '短信通知',
    description: '发送短信到指定手机号',
    icon: 'ChatDotRound',
  },
  {
    value: 'webhook',
    label: 'Webhook',
    description: '发送HTTP请求到指定URL',
    icon: 'Connection',
  },
  {
    value: 'dingtalk',
    label: '钉钉通知',
    description: '发送消息到钉钉群',
    icon: 'ChatLineRound',
  },
  {
    value: 'system',
    label: '系统通知',
    description: '显示系统内通知',
    icon: 'Bell',
  },
]

// ===== 计算属性 =====
const hasEnabledChannels = computed(() => {
  return notifications.value.some(n => n.enabled)
})

// ===== 方法 =====

/**
 * 获取通知渠道类型名称
 */
function getChannelTypeName(type: string): string {
  const channelType = availableChannelTypes.find(t => t.value === type)
  return channelType ? channelType.label : type
}

/**
 * 添加通知渠道
 */
function addNotificationChannel() {
  showAddChannelDialog.value = true
  selectedChannelType.value = ''
}

/**
 * 选择渠道类型
 */
function selectChannelType(type: string) {
  selectedChannelType.value = type
}

/**
 * 确认添加渠道
 */
function confirmAddChannel() {
  if (!selectedChannelType.value) return

  const newNotification = createNotificationTemplate(selectedChannelType.value)
  notifications.value.push(newNotification)

  showAddChannelDialog.value = false
  selectedChannelType.value = ''

  ElMessage.success('通知渠道已添加')
}

/**
 * 创建通知模板
 */
function createNotificationTemplate(type: string) {
  const baseTemplate = {
    type,
    enabled: true,
    conditions: {
      severityLevels: [props.severity],
      timeRange: 'always',
      startTime: '09:00',
      endTime: '18:00',
      onTrigger: true,
      onRecovery: true,
      repeatInterval: 0,
    },
  }

  switch (type) {
    case 'email':
      return {
        ...baseTemplate,
        config: {
          recipients: [],
          template: 'default',
          includeSnapshot: true,
          htmlFormat: true,
        },
      }
    case 'sms':
      return {
        ...baseTemplate,
        config: {
          phoneNumbers: [],
          template: 'alert',
        },
      }
    case 'webhook':
      return {
        ...baseTemplate,
        config: {
          url: '',
          method: 'POST',
          headers: [{ key: 'Content-Type', value: 'application/json' }],
          timeout: 30,
          verifySSL: true,
        },
      }
    case 'dingtalk':
      return {
        ...baseTemplate,
        config: {
          token: '',
          messageType: 'text',
          atMobiles: '',
          atAll: false,
        },
      }
    case 'system':
      return {
        ...baseTemplate,
        config: {
          level: 'warning',
          duration: 5,
          showDesktop: true,
          playSound: true,
        },
      }
    default:
      return baseTemplate
  }
}

/**
 * 移除通知渠道
 */
function removeNotificationChannel(index: number) {
  notifications.value.splice(index, 1)
  ElMessage.success('通知渠道已移除')
}

/**
 * 处理渠道启用状态变化
 */
function handleChannelEnabledChange(index: number) {
  const notification = notifications.value[index]
  if (notification.enabled) {
    ElMessage.success(`${getChannelTypeName(notification.type)} 已启用`)
  } else {
    ElMessage.info(`${getChannelTypeName(notification.type)} 已禁用`)
  }
}

/**
 * 添加请求头
 */
function addHeader(notificationIndex: number) {
  const notification = notifications.value[notificationIndex]
  if (notification.config.headers) {
    notification.config.headers.push({ key: '', value: '' })
  }
}

/**
 * 移除请求头
 */
function removeHeader(notificationIndex: number, headerIndex: number) {
  const notification = notifications.value[notificationIndex]
  if (notification.config.headers) {
    notification.config.headers.splice(headerIndex, 1)
  }
}

/**
 * 测试通知
 */
async function testNotification(index: number) {
  const notification = notifications.value[index]
  testingChannels.value[index] = true

  try {
    // 调用真实API测试通知渠道
    const response = await fetch('/api/v1/notifications/test', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        type: notification.type,
        config: notification.config,
        testMessage: `这是一条${getChannelTypeName(notification.type)}测试消息`,
      }),
    })

    if (response.ok) {
      const result = await response.json()
      ElMessage.success(
        `${getChannelTypeName(notification.type)} 测试通知已发送`
      )
    } else {
      const errorData = await response.json()
      throw new Error(errorData.message || '测试失败')
    }
  } catch (error) {
    console.error('测试通知失败:', error)
    ElMessage.error(`测试通知失败: ${error.message}`)
  } finally {
    testingChannels.value[index] = false
  }
}

// ===== 监听器 =====
watch(
  notifications,
  newValue => {
    emit('update:modelValue', newValue)
  },
  { deep: true }
)

watch(
  () => props.modelValue,
  newValue => {
    if (newValue) {
      notifications.value = newValue
    }
  },
  { immediate: true }
)

// 根据严重程度自动调整默认通知级别
watch(
  () => props.severity,
  newSeverity => {
    notifications.value.forEach(notification => {
      if (!notification.conditions.severityLevels.includes(newSeverity)) {
        notification.conditions.severityLevels = [newSeverity]
      }
    })
  }
)
</script>

<style scoped lang="scss">
.alert-notification-settings {
  .notification-channels {
    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;

      .section-title {
        font-size: 16px;
        font-weight: 600;
        color: #303133;
        margin: 0;
      }
    }

    .empty-state {
      text-align: center;
      padding: 40px 20px;
      border: 2px dashed #e4e7ed;
      border-radius: 8px;
      background: #fafafa;
    }

    .channels-list {
      .channel-item {
        border: 1px solid #e4e7ed;
        border-radius: 8px;
        margin-bottom: 16px;
        overflow: hidden;

        &:last-child {
          margin-bottom: 0;
        }

        .channel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px;
          background: #f8f9fa;
          border-bottom: 1px solid #e4e7ed;

          .channel-info {
            display: flex;
            align-items: center;
            gap: 12px;

            .channel-icon {
              font-size: 18px;
              color: #409eff;
            }

            .channel-name {
              font-weight: 500;
              color: #303133;
            }
          }

          .channel-actions {
            display: flex;
            align-items: center;
            gap: 12px;
          }
        }

        .channel-config {
          padding: 20px;

          .config-section {
            margin-bottom: 24px;

            .config-row {
              display: flex;
              gap: 20px;
              margin-bottom: 16px;
              flex-wrap: wrap;

              &:last-child {
                margin-bottom: 0;
              }

              .config-item {
                flex: 1;
                min-width: 200px;

                &.full-width {
                  flex-basis: 100%;
                }

                .config-label {
                  display: block;
                  font-size: 13px;
                  color: #606266;
                  margin-bottom: 6px;
                  font-weight: 500;
                }
              }
            }

            .headers-config {
              .header-item {
                display: flex;
                align-items: center;
                gap: 8px;
                margin-bottom: 8px;
              }
            }

            .timeout-input,
            .duration-input,
            .interval-input {
              display: flex;
              align-items: center;
              gap: 8px;

              span {
                font-size: 13px;
                color: #606266;
              }
            }
          }

          .notification-conditions {
            margin-top: 24px;
            padding-top: 20px;
            border-top: 1px solid #e4e7ed;

            .condition-title {
              font-size: 14px;
              font-weight: 600;
              color: #303133;
              margin: 0 0 16px 0;
            }

            .condition-config {
              .condition-row {
                display: flex;
                gap: 20px;
                margin-bottom: 16px;
                flex-wrap: wrap;

                &:last-child {
                  margin-bottom: 0;
                }

                .condition-item {
                  flex: 1;
                  min-width: 200px;

                  .condition-label {
                    display: block;
                    font-size: 13px;
                    color: #606266;
                    margin-bottom: 6px;
                  }
                }
              }
            }
          }

          .channel-test {
            margin-top: 20px;
            padding-top: 16px;
            border-top: 1px solid #e4e7ed;
            text-align: right;
          }
        }
      }
    }
  }

  .advanced-settings {
    margin-top: 32px;
    padding-top: 24px;
    border-top: 1px solid #e4e7ed;

    .section-title {
      font-size: 16px;
      font-weight: 600;
      color: #303133;
      margin: 0 0 20px 0;
    }

    .settings-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: 20px;

      .setting-item {
        display: flex;
        flex-direction: column;
        gap: 8px;

        .setting-label {
          font-size: 14px;
          font-weight: 500;
          color: #303133;
        }

        .setting-tip {
          font-size: 12px;
          color: #909399;
        }

        .rate-limit-config {
          display: flex;
          align-items: center;
          gap: 8px;
          flex-wrap: wrap;

          span {
            font-size: 13px;
            color: #606266;
          }
        }
      }
    }
  }
}

// 添加渠道对话框样式
.channel-types {
  .channel-type-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    border: 1px solid #e4e7ed;
    border-radius: 8px;
    margin-bottom: 8px;
    cursor: pointer;
    transition: all 0.3s;

    &:hover {
      border-color: #409eff;
      background: #f0f9ff;
    }

    &.selected {
      border-color: #409eff;
      background: #e1f3fe;
    }

    &:last-child {
      margin-bottom: 0;
    }

    .type-icon {
      font-size: 24px;
      color: #409eff;
    }

    .type-info {
      .type-name {
        font-size: 14px;
        font-weight: 500;
        color: #303133;
        margin-bottom: 4px;
      }

      .type-desc {
        font-size: 12px;
        color: #909399;
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .alert-notification-settings {
    .channels-list .channel-item .channel-config {
      padding: 16px;

      .config-section .config-row {
        flex-direction: column;
        gap: 16px;

        .config-item {
          min-width: auto;
        }
      }

      .notification-conditions .condition-config .condition-row {
        flex-direction: column;
        gap: 16px;

        .condition-item {
          min-width: auto;
        }
      }
    }

    .advanced-settings .settings-grid {
      grid-template-columns: 1fr;
      gap: 16px;
    }
  }
}
</style>
