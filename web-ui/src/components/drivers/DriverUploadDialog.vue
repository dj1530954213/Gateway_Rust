<template>
  <el-dialog
    v-model="dialogVisible"
    title="上传驱动文件"
    width="600px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @closed="handleDialogClosed"
  >
    <div class="upload-dialog">
      <!-- 上传说明 -->
      <el-alert
        title="上传要求"
        type="info"
        :closable="false"
        show-icon
        class="upload-notice"
      >
        <ul class="upload-rules">
          <li>
            仅支持 .so (Linux)、.dll (Windows)、.dylib (macOS)
            格式的动态链接库文件
          </li>
          <li>文件大小不能超过 100MB</li>
          <li>驱动文件必须实现规定的接口规范</li>
          <li>建议在上传前进行本地测试验证</li>
        </ul>
      </el-alert>

      <!-- 文件上传区域 -->
      <div class="upload-area">
        <ElUpload
          ref="uploadRef"
          class="driver-upload"
          drag
          :action="uploadAction"
          :headers="uploadHeaders"
          :multiple="false"
          :show-file-list="false"
          :before-upload="beforeUpload"
          :on-progress="handleProgress"
          :on-success="handleSuccess"
          :on-error="handleError"
          :auto-upload="false"
          accept=".so,.dll,.dylib"
        >
          <div class="upload-content">
            <el-icon class="upload-icon" :size="48">
              <Upload />
            </el-icon>
            <div class="upload-text">
              <p class="primary-text">点击选择文件或拖拽到此处</p>
              <p class="secondary-text">支持 .so / .dll / .dylib 格式</p>
            </div>
          </div>
        </ElUpload>
      </div>

      <!-- 选中的文件信息 -->
      <div v-if="selectedFile" class="file-info">
        <el-card shadow="never" class="file-card">
          <div class="file-details">
            <div class="file-icon">
              <el-icon :size="32">
                <Document />
              </el-icon>
            </div>
            <div class="file-meta">
              <div class="file-name">{{ selectedFile.name }}</div>
              <div class="file-size">
                {{ formatFileSize(selectedFile.size) }}
              </div>
              <div class="file-type">
                {{ getFileTypeDisplay(selectedFile.name) }}
              </div>
            </div>
            <div class="file-actions">
              <el-button
                type="danger"
                text
                :disabled="uploading"
                @click="removeFile"
              >
                移除
              </el-button>
            </div>
          </div>
        </el-card>
      </div>

      <!-- 上传进度 -->
      <div v-if="uploading || uploadProgress > 0" class="upload-progress">
        <div class="progress-info">
          <span class="progress-text">上传进度</span>
          <span class="progress-percentage"
            >{{ Math.round(uploadProgress) }}%</span
          >
        </div>
        <el-progress
          :percentage="uploadProgress"
          :status="uploadStatus"
          :stroke-width="8"
        />
      </div>

      <!-- 上传结果 -->
      <div v-if="uploadResult" class="upload-result">
        <el-alert
          :title="uploadResult.success ? '上传成功' : '上传失败'"
          :type="uploadResult.success ? 'success' : 'error'"
          :closable="false"
          show-icon
        >
          <template v-if="uploadResult.success && uploadResult.data">
            <div class="success-info">
              <p><strong>驱动文件:</strong> {{ uploadResult.data.filename }}</p>
              <p v-if="uploadResult.data.info">
                <strong>驱动信息:</strong> {{ uploadResult.data.info.name }} v{{
                  uploadResult.data.info.version
                }}
              </p>
              <p v-if="uploadResult.data.info?.protocol">
                <strong>支持协议:</strong> {{ uploadResult.data.info.protocol }}
              </p>
              <p>
                <strong>加载状态:</strong>
                {{ getStatusDisplayName(uploadResult.data.status) }}
              </p>
            </div>
          </template>
          <template v-else>
            <div class="error-info">
              <p>{{ uploadResult.message || '上传过程中发生未知错误' }}</p>
            </div>
          </template>
        </el-alert>
      </div>
    </div>

    <!-- 对话框底部按钮 -->
    <template #footer>
      <div class="dialog-footer">
        <el-button :disabled="uploading" @click="handleCancel">
          取消
        </el-button>
        <el-button
          type="primary"
          :loading="uploading"
          :disabled="!selectedFile"
          @click="handleUpload"
        >
          {{ uploading ? '上传中...' : '开始上传' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * DriverUploadDialog —— 驱动文件上传对话框
 *
 * 📝 Responsibilities:
 *  1. 提供拖拽上传界面
 *  2. 文件格式和大小验证
 *  3. 上传进度显示
 *  4. 上传结果展示
 *  5. 错误处理和用户提示
 *
 * 📦 Dependencies:
 *  - Element Plus Upload 组件
 *  - 文件格式化工具
 *  - 驱动 API
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { Upload, Document } from '@element-plus/icons-vue'
import { ElMessage, ElUpload } from 'element-plus'
import { ref, computed, watch } from 'vue'

import type { DriverUploadResponse, DriverVO } from '@/api/drivers'
import { formatFileSize } from '@/utils/format'

// ===== Props =====
const props = defineProps<{
  visible: boolean
}>()

// ===== Emits =====
const emit = defineEmits<{
  'update:visible': [visible: boolean]
  success: [result: DriverUploadResponse]
}>()

// ===== 响应式数据 =====
const uploadRef = ref<InstanceType<typeof ElUpload>>()
const selectedFile = ref<File | null>(null)
const uploading = ref(false)
const uploadProgress = ref(0)
const uploadStatus = ref<'success' | 'exception' | ''>('')
const uploadResult = ref<DriverUploadResponse | null>(null)

// ===== 计算属性 =====
const dialogVisible = computed({
  get: () => props.visible,
  set: value => emit('update:visible', value),
})

const uploadAction = computed(() => {
  return `${import.meta.env.VITE_API_BASE}/drivers/upload`
})

const uploadHeaders = computed(() => {
  return {
    Authorization: `Bearer ${localStorage.getItem('token') || ''}`,
  }
})

// ===== 方法 =====

/**
 * 上传前验证
 */
function beforeUpload(file: File): boolean {
  // 检查文件格式
  const allowedExtensions = ['.so', '.dll', '.dylib']
  const fileName = file.name.toLowerCase()
  const isValidFormat = allowedExtensions.some(ext => fileName.endsWith(ext))

  if (!isValidFormat) {
    ElMessage.error('文件格式不正确，仅支持 .so、.dll、.dylib 格式')
    return false
  }

  // 检查文件大小 (100MB)
  const maxSize = 100 * 1024 * 1024
  if (file.size > maxSize) {
    ElMessage.error('文件大小不能超过 100MB')
    return false
  }

  selectedFile.value = file
  uploadResult.value = null
  uploadProgress.value = 0
  uploadStatus.value = ''

  return false // 阻止自动上传
}

/**
 * 处理上传进度
 */
function handleProgress(event: any) {
  uploadProgress.value = event.percent || 0
}

/**
 * 处理上传成功
 */
function handleSuccess(response: any) {
  uploading.value = false
  uploadProgress.value = 100
  uploadStatus.value = 'success'

  if (response.success) {
    uploadResult.value = response
    ElMessage.success('驱动上传成功')

    // 延迟触发成功事件，让用户看到结果
    setTimeout(() => {
      emit('success', response)
    }, 1500)
  } else {
    uploadStatus.value = 'exception'
    uploadResult.value = response
    ElMessage.error(response.message || '上传失败')
  }
}

/**
 * 处理上传错误
 */
function handleError(error: any) {
  uploading.value = false
  uploadStatus.value = 'exception'
  uploadProgress.value = 0

  const errorMessage =
    error?.response?.data?.message || error.message || '上传失败'
  uploadResult.value = {
    success: false,
    message: errorMessage,
    data: null,
  }

  ElMessage.error(errorMessage)
}

/**
 * 开始上传
 */
function handleUpload() {
  if (!selectedFile.value) {
    ElMessage.warning('请先选择要上传的文件')
    return
  }

  uploading.value = true
  uploadProgress.value = 0
  uploadStatus.value = ''
  uploadResult.value = null

  // 使用 Upload 组件的 submit 方法
  uploadRef.value?.submit()
}

/**
 * 移除文件
 */
function removeFile() {
  selectedFile.value = null
  uploadProgress.value = 0
  uploadStatus.value = ''
  uploadResult.value = null
  uploadRef.value?.clearFiles()
}

/**
 * 处理取消
 */
function handleCancel() {
  if (uploading.value) {
    ElMessage.warning('上传中，无法关闭')
    return
  }

  dialogVisible.value = false
}

/**
 * 处理对话框关闭
 */
function handleDialogClosed() {
  // 重置状态
  selectedFile.value = null
  uploading.value = false
  uploadProgress.value = 0
  uploadStatus.value = ''
  uploadResult.value = null
  uploadRef.value?.clearFiles()
}

/**
 * 获取文件类型显示名称
 */
function getFileTypeDisplay(filename: string): string {
  const extension = filename.split('.').pop()?.toLowerCase()
  const typeMap: Record<string, string> = {
    so: 'Linux 动态链接库',
    dll: 'Windows 动态链接库',
    dylib: 'macOS 动态链接库',
  }
  return typeMap[extension || ''] || '未知类型'
}

/**
 * 获取状态显示名称
 */
function getStatusDisplayName(status: string): string {
  const nameMap: Record<string, string> = {
    Loaded: '已加载',
    Failed: '加载失败',
    Unloaded: '未加载',
  }
  return nameMap[status] || status
}

// ===== 监听器 =====
watch(
  () => props.visible,
  visible => {
    if (!visible) {
      handleDialogClosed()
    }
  }
)
</script>

<style scoped lang="scss">
.upload-dialog {
  .upload-notice {
    margin-bottom: 20px;

    .upload-rules {
      margin: 8px 0 0 0;
      padding-left: 20px;

      li {
        margin-bottom: 4px;
        font-size: 13px;
        line-height: 1.4;
      }
    }
  }

  .upload-area {
    margin-bottom: 20px;

    .driver-upload {
      width: 100%;

      :deep(.el-upload) {
        width: 100%;

        .el-upload-dragger {
          width: 100%;
          height: 160px;
          border: 2px dashed #d9d9d9;
          border-radius: 8px;
          background-color: #fafafa;
          transition: all 0.3s;

          &:hover {
            border-color: #409eff;
            background-color: #f0f9ff;
          }
        }
      }

      .upload-content {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding: 20px;

        .upload-icon {
          color: #409eff;
          margin-bottom: 16px;
        }

        .upload-text {
          text-align: center;

          .primary-text {
            font-size: 16px;
            color: #303133;
            margin: 0 0 8px 0;
            font-weight: 500;
          }

          .secondary-text {
            font-size: 14px;
            color: #909399;
            margin: 0;
          }
        }
      }
    }
  }

  .file-info {
    margin-bottom: 20px;

    .file-card {
      .file-details {
        display: flex;
        align-items: center;
        gap: 16px;

        .file-icon {
          color: #409eff;
          flex-shrink: 0;
        }

        .file-meta {
          flex: 1;
          min-width: 0;

          .file-name {
            font-size: 16px;
            font-weight: 500;
            color: #303133;
            margin-bottom: 4px;
            word-break: break-all;
          }

          .file-size {
            font-size: 14px;
            color: #606266;
            margin-bottom: 2px;
          }

          .file-type {
            font-size: 13px;
            color: #909399;
          }
        }

        .file-actions {
          flex-shrink: 0;
        }
      }
    }
  }

  .upload-progress {
    margin-bottom: 20px;

    .progress-info {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;

      .progress-text {
        font-size: 14px;
        color: #606266;
      }

      .progress-percentage {
        font-size: 14px;
        font-weight: 500;
        color: #409eff;
      }
    }
  }

  .upload-result {
    .success-info,
    .error-info {
      p {
        margin: 4px 0;
        font-size: 14px;
        line-height: 1.4;

        strong {
          color: #303133;
        }
      }
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

// 响应式设计
@media (max-width: 768px) {
  .upload-dialog {
    .upload-area {
      .driver-upload {
        :deep(.el-upload-dragger) {
          height: 120px;
        }

        .upload-content {
          padding: 16px;

          .upload-icon {
            font-size: 36px;
            margin-bottom: 12px;
          }

          .upload-text {
            .primary-text {
              font-size: 14px;
            }

            .secondary-text {
              font-size: 12px;
            }
          }
        }
      }
    }

    .file-info {
      .file-card {
        .file-details {
          flex-direction: column;
          align-items: flex-start;
          gap: 12px;

          .file-meta {
            .file-name {
              font-size: 14px;
            }
          }

          .file-actions {
            align-self: flex-end;
          }
        }
      }
    }
  }
}
</style>
