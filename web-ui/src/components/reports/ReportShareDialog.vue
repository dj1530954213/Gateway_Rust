<template>
  <el-dialog
    v-model="visible"
    title="分享报表"
    width="600px"
    :close-on-click-modal="false"
  >
    <div class="report-share">
      <!-- 分享方式选择 -->
      <div class="share-methods">
        <h4>选择分享方式</h4>
        <el-tabs v-model="activeTab" class="share-tabs">
          <el-tab-pane label="链接分享" name="link">
            <div class="link-share">
              <div class="share-settings">
                <el-form label-width="100px">
                  <el-form-item label="访问权限">
                    <el-radio-group v-model="linkConfig.permission">
                      <el-radio label="public">公开访问</el-radio>
                      <el-radio label="protected">密码保护</el-radio>
                      <el-radio label="private">仅限邀请</el-radio>
                    </el-radio-group>
                  </el-form-item>

                  <el-form-item
                    v-if="linkConfig.permission === 'protected'"
                    label="访问密码"
                  >
                    <el-input
                      v-model="linkConfig.password"
                      type="password"
                      placeholder="设置访问密码"
                      maxlength="20"
                      show-password
                    />
                  </el-form-item>

                  <el-form-item label="有效期">
                    <el-select
                      v-model="linkConfig.expiry"
                      placeholder="选择有效期"
                    >
                      <el-option label="永久有效" value="never" />
                      <el-option label="1小时" value="1h" />
                      <el-option label="1天" value="1d" />
                      <el-option label="7天" value="7d" />
                      <el-option label="30天" value="30d" />
                      <el-option label="自定义" value="custom" />
                    </el-select>
                  </el-form-item>

                  <el-form-item
                    v-if="linkConfig.expiry === 'custom'"
                    label="自定义时间"
                  >
                    <el-date-picker
                      v-model="linkConfig.customExpiry"
                      type="datetime"
                      placeholder="选择过期时间"
                      format="YYYY-MM-DD HH:mm:ss"
                      value-format="YYYY-MM-DD HH:mm:ss"
                    />
                  </el-form-item>

                  <el-form-item label="允许下载">
                    <el-switch v-model="linkConfig.allowDownload" />
                  </el-form-item>
                </el-form>
              </div>

              <div class="link-result">
                <el-input
                  v-model="shareLink"
                  readonly
                  placeholder="点击生成分享链接"
                >
                  <template #append>
                    <el-button @click="generateShareLink">
                      <el-icon><Refresh /></el-icon>
                      生成链接
                    </el-button>
                  </template>
                </el-input>

                <div class="link-actions">
                  <el-button :disabled="!shareLink" @click="copyLink">
                    <el-icon><CopyDocument /></el-icon>
                    复制链接
                  </el-button>
                  <el-button :disabled="!shareLink" @click="openQRCode">
                    <el-icon><Picture /></el-icon>
                    生成二维码
                  </el-button>
                </div>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="邮件发送" name="email">
            <div class="email-share">
              <el-form :model="emailConfig" label-width="80px">
                <el-form-item label="收件人">
                  <el-select
                    v-model="emailConfig.recipients"
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

                <el-form-item label="抄送">
                  <el-select
                    v-model="emailConfig.cc"
                    multiple
                    filterable
                    allow-create
                    placeholder="抄送邮箱（可选）"
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

                <el-form-item label="主题">
                  <el-input
                    v-model="emailConfig.subject"
                    placeholder="邮件主题"
                  />
                </el-form-item>

                <el-form-item label="内容">
                  <el-input
                    v-model="emailConfig.message"
                    type="textarea"
                    :rows="5"
                    placeholder="邮件内容（可选）"
                  />
                </el-form-item>

                <el-form-item label="附件格式">
                  <el-checkbox-group v-model="emailConfig.attachments">
                    <el-checkbox
                      v-for="format in report?.formats || []"
                      :key="format"
                      :label="format"
                    >
                      {{ format.toUpperCase() }}
                    </el-checkbox>
                  </el-checkbox-group>
                </el-form-item>
              </el-form>
            </div>
          </el-tab-pane>

          <el-tab-pane label="团队协作" name="team">
            <div class="team-share">
              <div class="team-members">
                <h5>选择团队成员</h5>
                <el-checkbox-group v-model="teamConfig.members">
                  <div
                    v-for="member in teamMembers"
                    :key="member.id"
                    class="member-item"
                  >
                    <el-checkbox :label="member.id">
                      <div class="member-info">
                        <el-avatar :size="32" :src="member.avatar">
                          {{ member.name.charAt(0) }}
                        </el-avatar>
                        <div class="member-details">
                          <div class="member-name">{{ member.name }}</div>
                          <div class="member-role">{{ member.role }}</div>
                        </div>
                      </div>
                    </el-checkbox>
                  </div>
                </el-checkbox-group>
              </div>

              <div class="team-permissions">
                <h5>权限设置</h5>
                <el-form label-width="100px">
                  <el-form-item label="查看权限">
                    <el-switch v-model="teamConfig.permissions.view" />
                  </el-form-item>
                  <el-form-item label="下载权限">
                    <el-switch v-model="teamConfig.permissions.download" />
                  </el-form-item>
                  <el-form-item label="评论权限">
                    <el-switch v-model="teamConfig.permissions.comment" />
                  </el-form-item>
                  <el-form-item label="分享权限">
                    <el-switch v-model="teamConfig.permissions.share" />
                  </el-form-item>
                </el-form>
              </div>

              <div class="notification-settings">
                <h5>通知设置</h5>
                <el-checkbox-group v-model="teamConfig.notifications">
                  <el-checkbox label="email">邮件通知</el-checkbox>
                  <el-checkbox label="system">系统消息</el-checkbox>
                  <el-checkbox label="mobile">手机推送</el-checkbox>
                </el-checkbox-group>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>

      <!-- 分享历史 -->
      <div class="share-history">
        <h4>分享历史</h4>
        <el-table :data="shareHistory" style="width: 100%" max-height="200">
          <el-table-column prop="method" label="方式" width="80" />
          <el-table-column prop="target" label="分享对象" />
          <el-table-column prop="createTime" label="创建时间" width="150" />
          <el-table-column prop="status" label="状态" width="80">
            <template #default="{ row }">
              <el-tag :type="getStatusType(row.status)" size="small">
                {{ row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="100">
            <template #default="{ row }">
              <el-button
                size="small"
                text
                :disabled="row.status === '已撤销'"
                @click="revokeShare(row)"
              >
                撤销
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- 对话框底部 -->
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleCancel">取消</el-button>
        <el-button type="primary" :loading="sharing" @click="handleShare">
          {{ sharing ? '分享中...' : '确认分享' }}
        </el-button>
      </div>
    </template>

    <!-- 二维码对话框 -->
    <el-dialog v-model="showQRCode" title="分享二维码" width="400px">
      <div class="qr-code-content">
        <div ref="qrCodeRef" class="qr-code"></div>
        <div class="qr-info">
          <p>扫描二维码查看报表</p>
          <p class="qr-link">{{ shareLink }}</p>
        </div>
        <div class="qr-actions">
          <el-button @click="downloadQRCode">下载二维码</el-button>
          <el-button type="primary" @click="copyLink">复制链接</el-button>
        </div>
      </div>
    </el-dialog>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * ReportShareDialog —— 报表分享对话框
 *
 * 📝 Responsibilities:
 *  1. 多种分享方式支持
 *  2. 权限控制和有效期设置
 *  3. 分享历史管理
 *  4. 二维码生成
 *
 * 📦 Dependencies:
 *  - Element Plus UI组件
 *  - QRCode 二维码生成库
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { Refresh, CopyDocument, Picture } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, nextTick } from 'vue'

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

const activeTab = ref('link')
const sharing = ref(false)
const showQRCode = ref(false)
const qrCodeRef = ref<HTMLDivElement>()

// 分享链接配置
const linkConfig = ref({
  permission: 'public',
  password: '',
  expiry: '7d',
  customExpiry: '',
  allowDownload: true,
})

const shareLink = ref('')

// 邮件分享配置
const emailConfig = ref({
  recipients: [],
  cc: [],
  subject: '',
  message: '',
  attachments: ['pdf'],
})

// 团队分享配置
const teamConfig = ref({
  members: [],
  permissions: {
    view: true,
    download: true,
    comment: false,
    share: false,
  },
  notifications: ['email', 'system'],
})

// 邮件联系人
const emailContacts = ref([
  { name: '张三', email: 'zhangsan@example.com' },
  { name: '李四', email: 'lisi@example.com' },
  { name: '王五', email: 'wangwu@example.com' },
])

// 团队成员
const teamMembers = ref([
  {
    id: '1',
    name: '张三',
    role: '系统管理员',
    avatar: '',
  },
  {
    id: '2',
    name: '李四',
    role: '数据分析师',
    avatar: '',
  },
  {
    id: '3',
    name: '王五',
    role: '设备工程师',
    avatar: '',
  },
  {
    id: '4',
    name: '赵六',
    role: '运维工程师',
    avatar: '',
  },
])

// 分享历史
const shareHistory = ref([
  {
    id: '1',
    method: '链接',
    target: '公开链接',
    createTime: '2025-07-27 10:30',
    status: '有效',
  },
  {
    id: '2',
    method: '邮件',
    target: 'zhangsan@example.com',
    createTime: '2025-07-26 15:20',
    status: '已发送',
  },
  {
    id: '3',
    method: '团队',
    target: '开发团队',
    createTime: '2025-07-25 09:15',
    status: '已撤销',
  },
])

// ===== 方法 =====

/**
 * 生成分享链接
 */
function generateShareLink() {
  const baseUrl = window.location.origin
  const reportId = props.report?.id || 'demo'
  const token = Math.random().toString(36).substr(2, 16)

  shareLink.value = `${baseUrl}/shared/report/${reportId}?token=${token}`

  ElMessage.success('分享链接已生成')
}

/**
 * 复制链接
 */
async function copyLink() {
  try {
    await navigator.clipboard.writeText(shareLink.value)
    ElMessage.success('链接已复制到剪贴板')
  } catch (error) {
    console.error('复制失败:', error)
    ElMessage.error('复制失败')
  }
}

/**
 * 打开二维码
 */
function openQRCode() {
  if (!shareLink.value) {
    ElMessage.warning('请先生成分享链接')
    return
  }

  showQRCode.value = true

  nextTick(() => {
    generateQRCode()
  })
}

/**
 * 生成二维码
 */
function generateQRCode() {
  if (!qrCodeRef.value) return

  // 这里使用简单的二维码模拟
  // 实际项目中可以使用 qrcode 库
  qrCodeRef.value.innerHTML = `
    <div style="
      width: 200px; 
      height: 200px; 
      background: #f0f0f0; 
      display: flex; 
      align-items: center; 
      justify-content: center; 
      border: 1px solid #ddd;
      margin: 0 auto;
    ">
      <div style="font-size: 12px; color: #666; text-align: center;">
        二维码<br/>
        (模拟显示)
      </div>
    </div>
  `
}

/**
 * 下载二维码
 */
function downloadQRCode() {
  ElMessage.success('二维码图片已下载')
  // 这里实现二维码下载逻辑
}

/**
 * 处理分享
 */
async function handleShare() {
  sharing.value = true

  try {
    if (activeTab.value === 'link') {
      await handleLinkShare()
    } else if (activeTab.value === 'email') {
      await handleEmailShare()
    } else if (activeTab.value === 'team') {
      await handleTeamShare()
    }

    ElMessage.success('分享成功')

    // 更新分享历史
    updateShareHistory()

    // 关闭对话框
    visible.value = false
  } catch (error) {
    console.error('分享失败:', error)
    ElMessage.error('分享失败')
  } finally {
    sharing.value = false
  }
}

/**
 * 处理链接分享
 */
async function handleLinkShare() {
  if (!shareLink.value) {
    throw new Error('请先生成分享链接')
  }

  // 模拟API调用
  await new Promise(resolve => setTimeout(resolve, 1000))

  console.log('链接分享配置:', linkConfig.value)
}

/**
 * 处理邮件分享
 */
async function handleEmailShare() {
  if (!emailConfig.value.recipients.length) {
    throw new Error('请至少选择一个收件人')
  }

  // 模拟邮件发送
  await new Promise(resolve => setTimeout(resolve, 2000))

  console.log('邮件分享配置:', emailConfig.value)
}

/**
 * 处理团队分享
 */
async function handleTeamShare() {
  if (!teamConfig.value.members.length) {
    throw new Error('请至少选择一个团队成员')
  }

  // 模拟团队分享
  await new Promise(resolve => setTimeout(resolve, 1500))

  console.log('团队分享配置:', teamConfig.value)
}

/**
 * 更新分享历史
 */
function updateShareHistory() {
  const now = new Date().toLocaleString('zh-CN').slice(0, 16)
  const newRecord: any = {
    id: Date.now().toString(),
    createTime: now,
    status: '有效',
  }

  if (activeTab.value === 'link') {
    newRecord.method = '链接'
    newRecord.target =
      linkConfig.value.permission === 'public' ? '公开链接' : '密码保护链接'
  } else if (activeTab.value === 'email') {
    newRecord.method = '邮件'
    newRecord.target = emailConfig.value.recipients.slice(0, 2).join(', ')
    newRecord.status = '已发送'
  } else if (activeTab.value === 'team') {
    newRecord.method = '团队'
    newRecord.target = `${teamConfig.value.members.length} 位成员`
  }

  shareHistory.value.unshift(newRecord)
}

/**
 * 撤销分享
 */
function revokeShare(record: any) {
  record.status = '已撤销'
  ElMessage.success('分享已撤销')
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): string {
  if (status === '有效') return 'success'
  if (status === '已发送') return 'info'
  if (status === '已撤销') return 'danger'
  return 'info'
}

/**
 * 处理取消
 */
function handleCancel() {
  visible.value = false
}

// ===== 生命周期 =====
// 初始化默认值
if (props.report) {
  emailConfig.value.subject = `分享报表: ${props.report.name}`
  emailConfig.value.message = `我分享了一个报表给您: ${props.report.name}\n\n${props.report.description || ''}`
}
</script>

<style scoped lang="scss">
.report-share {
  .share-methods {
    margin-bottom: 24px;

    h4 {
      margin: 0 0 16px 0;
      color: #303133;
      font-size: 16px;
    }

    .share-tabs {
      :deep(.el-tabs__content) {
        padding-top: 16px;
      }
    }

    .link-share {
      .share-settings {
        margin-bottom: 20px;
        padding: 16px;
        background: #fafafa;
        border-radius: 6px;
      }

      .link-result {
        .link-actions {
          margin-top: 12px;
          display: flex;
          gap: 8px;
        }
      }
    }

    .email-share {
      :deep(.el-form-item__label) {
        font-weight: 500;
      }
    }

    .team-share {
      .team-members {
        margin-bottom: 20px;

        h5 {
          margin: 0 0 12px 0;
          color: #303133;
          font-size: 14px;
        }

        .member-item {
          margin-bottom: 12px;

          :deep(.el-checkbox) {
            width: 100%;

            .el-checkbox__label {
              width: 100%;
            }
          }

          .member-info {
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 8px;
            border-radius: 6px;
            transition: background 0.2s;

            &:hover {
              background: #f0f9ff;
            }

            .member-details {
              flex: 1;

              .member-name {
                font-size: 14px;
                color: #303133;
                font-weight: 500;
              }

              .member-role {
                font-size: 12px;
                color: #909399;
                margin-top: 2px;
              }
            }
          }
        }
      }

      .team-permissions,
      .notification-settings {
        margin-bottom: 20px;

        h5 {
          margin: 0 0 12px 0;
          color: #303133;
          font-size: 14px;
        }
      }
    }
  }

  .share-history {
    border-top: 1px solid #ebeef5;
    padding-top: 16px;

    h4 {
      margin: 0 0 12px 0;
      color: #303133;
      font-size: 16px;
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.qr-code-content {
  text-align: center;

  .qr-code {
    margin-bottom: 16px;
  }

  .qr-info {
    margin-bottom: 20px;

    p {
      margin: 8px 0;
      color: #606266;

      &.qr-link {
        font-size: 12px;
        color: #909399;
        word-break: break-all;
      }
    }
  }

  .qr-actions {
    display: flex;
    justify-content: center;
    gap: 12px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .report-share {
    .team-share .team-members .member-item .member-info {
      flex-direction: column;
      text-align: center;
      gap: 8px;
    }

    .link-share .link-result .link-actions {
      flex-direction: column;
    }
  }

  .qr-code-content .qr-actions {
    flex-direction: column;
  }
}
</style>
