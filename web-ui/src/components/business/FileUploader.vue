<template>
  <div class="file-uploader" :class="containerClass">
    <!-- 上传器头部 -->
    <div v-if="showHeader" class="uploader-header">
      <div class="header-title">
        <el-icon :size="18">
          <UploadFilled />
        </el-icon>
        <span>{{ title }}</span>
        <el-tag v-if="files.length > 0" type="info" size="small">
          {{ files.length }} 个文件
        </el-tag>
      </div>
      
      <div class="header-actions">
        <!-- 上传类型选择 -->
        <el-select
          v-if="uploadTypes.length > 1"
          v-model="selectedUploadType"
          size="small"
          style="width: 150px"
          @change="handleUploadTypeChange"
        >
          <el-option
            v-for="type in uploadTypes"
            :key="type.value"
            :label="type.label"
            :value="type.value"
          />
        </el-select>
        
        <!-- 清空文件 -->
        <el-button
          v-if="files.length > 0"
          type="text"
          size="small"
          :icon="Delete"
          @click="handleClearAll"
        >
          清空
        </el-button>
        
        <!-- 上传历史 -->
        <el-button
          v-if="showHistory"
          type="text"
          size="small"
          :icon="Clock"
          @click="historyVisible = true"
        >
          历史
        </el-button>
      </div>
    </div>
    
    <!-- 上传区域 -->
    <div class="upload-area">
      <el-upload
        ref="uploadRef"
        :action="uploadUrl"
        :headers="uploadHeaders"
        :data="uploadData"
        :file-list="files"
        :multiple="multiple"
        :accept="currentAccept"
        :limit="maxFiles"
        :show-file-list="false"
        :auto-upload="autoUpload"
        :drag="enableDrag"
        :before-upload="handleBeforeUpload"
        :on-exceed="handleExceed"
        :on-success="handleSuccess"
        :on-error="handleError"
        :on-progress="handleProgress"
        :http-request="customRequest"
        class="upload-component"
      >
        <div class="upload-content" :class="{ 'is-dragover': isDragover }">
          <!-- 拖拽提示 -->
          <div v-if="enableDrag && files.length === 0" class="drag-area">
            <el-icon :size="48" class="upload-icon">
              <UploadFilled />
            </el-icon>
            <div class="upload-text">
              <div class="primary-text">点击或拖拽文件到此处上传</div>
              <div class="secondary-text">{{ uploadTip }}</div>
            </div>
          </div>
          
          <!-- 选择文件按钮 -->
          <div v-else-if="files.length === 0" class="select-area">
            <el-button type="primary" :icon="Plus">
              选择文件
            </el-button>
            <div class="upload-tip">{{ uploadTip }}</div>
          </div>
          
          <!-- 文件列表预览 -->
          <div v-else class="file-preview">
            <div class="file-summary">
              <span>已选择 {{ files.length }} 个文件</span>
              <el-button type="text" :icon="Plus" @click="handleAddMore">
                继续添加
              </el-button>
            </div>
          </div>
        </div>
      </el-upload>
    </div>
    
    <!-- 文件列表 -->
    <div v-if="files.length > 0" class="file-list">
      <div
        v-for="(file, index) in files"
        :key="file.uid || index"
        class="file-item"
        :class="getFileItemClass(file)"
      >
        <div class="file-info">
          <div class="file-icon">
            <el-icon :size="24">
              <component :is="getFileIcon(file)" />
            </el-icon>
          </div>
          
          <div class="file-details">
            <div class="file-name" :title="file.name">
              {{ file.name }}
            </div>
            <div class="file-meta">
              <span class="file-size">{{ formatFileSize(file.size) }}</span>
              <span v-if="file.lastModified" class="file-time">
                {{ formatFileTime(file.lastModified) }}
              </span>
            </div>
          </div>
        </div>
        
        <div class="file-status">
          <!-- 上传进度 -->
          <div v-if="file.status === 'uploading'" class="upload-progress">
            <el-progress
              :percentage="file.percentage || 0"
              :stroke-width="4"
              :show-text="false"
            />
            <span class="progress-text">{{ Math.round(file.percentage || 0) }}%</span>
          </div>
          
          <!-- 状态标签 -->
          <StatusTag
            v-else
            :status="file.status"
            size="small"
            :mappings="fileStatusMappings"
          />
        </div>
        
        <div class="file-actions">
          <!-- 预览按钮 -->
          <el-button
            v-if="canPreview(file)"
            type="text"
            size="small"
            :icon="View"
            @click="handlePreview(file)"
          />
          
          <!-- 重新上传 -->
          <el-button
            v-if="file.status === 'error'"
            type="text"
            size="small"
            :icon="Refresh"
            @click="handleRetry(file)"
          />
          
          <!-- 删除文件 -->
          <el-button
            type="text"
            size="small"
            :icon="Delete"
            @click="handleRemove(file, index)"
          />
        </div>
      </div>
    </div>
    
    <!-- 批量操作栏 -->
    <div v-if="files.length > 0 && showBatchActions" class="batch-actions">
      <div class="batch-info">
        <span>{{ files.length }} 个文件，总大小 {{ totalSize }}</span>
      </div>
      
      <div class="batch-buttons">
        <el-button
          v-if="!autoUpload"
          type="primary"
          size="small"
          :loading="uploading"
          :disabled="!hasValidFiles"
          @click="handleUploadAll"
        >
          {{ uploading ? '上传中...' : '开始上传' }}
        </el-button>
        
        <el-button
          size="small"
          @click="handleClearAll"
        >
          清空列表
        </el-button>
      </div>
    </div>
    
    <!-- 上传配置 -->
    <div v-if="showConfig" class="upload-config">
      <el-collapse v-model="configCollapsed">
        <el-collapse-item title="上传设置" name="config">
          <BaseForm
            v-model="uploadConfig"
            :fields="configFields"
            label-width="100px"
            @change="handleConfigChange"
          />
        </el-collapse-item>
      </el-collapse>
    </div>
    
    <!-- 文件预览对话框 -->
    <el-dialog
      v-model="previewVisible"
      :title="`预览 - ${previewFile?.name}`"
      width="800px"
      :before-close="handlePreviewClose"
    >
      <div v-if="previewFile" class="file-preview-content">
        <!-- 图片预览 -->
        <div v-if="isImageFile(previewFile)" class="image-preview">
          <img
            :src="previewFile.url || getFileURL(previewFile)"
            :alt="previewFile.name"
            style="max-width: 100%; max-height: 500px;"
          />
        </div>
        
        <!-- 文本文件预览 -->
        <div v-else-if="isTextFile(previewFile)" class="text-preview">
          <el-input
            v-model="previewContent"
            type="textarea"
            :rows="20"
            readonly
            placeholder="加载中..."
          />
        </div>
        
        <!-- 其他文件信息 -->
        <div v-else class="file-info-preview">
          <el-descriptions border column="2">
            <el-descriptions-item label="文件名">
              {{ previewFile.name }}
            </el-descriptions-item>
            <el-descriptions-item label="文件大小">
              {{ formatFileSize(previewFile.size) }}
            </el-descriptions-item>
            <el-descriptions-item label="文件类型">
              {{ previewFile.type || '未知' }}
            </el-descriptions-item>
            <el-descriptions-item label="最后修改">
              {{ formatFileTime(previewFile.lastModified) }}
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="previewVisible = false">关闭</el-button>
      </template>
    </el-dialog>
    
    <!-- 上传历史对话框 -->
    <el-dialog
      v-model="historyVisible"
      title="上传历史"
      width="700px"
      :before-close="handleHistoryClose"
    >
      <BaseTable
        :data="uploadHistory"
        :columns="historyColumns"
        @action="handleHistoryAction"
      />
      
      <template #footer>
        <el-button @click="historyVisible = false">关闭</el-button>
        <el-button type="danger" @click="handleClearHistory">
          清空历史
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, watch } from 'vue'
import {
  UploadFilled,
  Plus,
  Delete,
  Clock,
  View,
  Refresh,
  Document,
  Picture,
  Files,
  VideoPlay,
  Download
} from '@element-plus/icons-vue'
import { BaseForm, BaseTable, StatusTag } from '../base'
import { ElMessage } from 'element-plus'

export type UploadStatus = 'ready' | 'uploading' | 'success' | 'error'

export interface FileItem {
  uid?: string
  name: string
  size: number
  type: string
  lastModified?: number
  status: UploadStatus
  percentage?: number
  url?: string
  response?: any
  raw?: File
}

export interface UploadType {
  value: string
  label: string
  accept: string
  maxSize: number
  description: string
}

export interface UploadHistoryRecord {
  id: string
  fileName: string
  fileSize: number
  uploadTime: Date
  status: UploadStatus
  type: string
}

interface Props {
  title?: string
  
  // 上传配置
  uploadUrl?: string
  uploadTypes?: UploadType[]
  multiple?: boolean
  autoUpload?: boolean
  enableDrag?: boolean
  maxFiles?: number
  chunkSize?: number
  
  // 功能控制
  showHeader?: boolean
  showBatchActions?: boolean
  showConfig?: boolean
  showHistory?: boolean
  
  // 外观配置
  size?: 'small' | 'default' | 'large'
  
  // 自定义样式
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: '文件上传',
  uploadUrl: '/api/upload',
  uploadTypes: () => [
    {
      value: 'config',
      label: '配置文件',
      accept: '.json,.xml,.yaml,.yml',
      maxSize: 10 * 1024 * 1024, // 10MB
      description: '支持 JSON、XML、YAML 格式，最大 10MB'
    },
    {
      value: 'firmware',
      label: '固件文件',
      accept: '.bin,.hex,.elf',
      maxSize: 100 * 1024 * 1024, // 100MB
      description: '支持 BIN、HEX、ELF 格式，最大 100MB'
    },
    {
      value: 'data',
      label: '数据文件',
      accept: '.csv,.xlsx,.json',
      maxSize: 50 * 1024 * 1024, // 50MB
      description: '支持 CSV、Excel、JSON 格式，最大 50MB'
    }
  ],
  multiple: true,
  autoUpload: false,
  enableDrag: true,
  maxFiles: 10,
  chunkSize: 1024 * 1024, // 1MB
  showHeader: true,
  showBatchActions: true,
  showConfig: false,
  showHistory: true,
  size: 'default',
})

interface Emits {
  success: [file: FileItem, response: any]
  error: [file: FileItem, error: any]
  progress: [file: FileItem, percentage: number]
  'upload-complete': [files: FileItem[]]
}

const emit = defineEmits<Emits>()

// 状态
const uploadRef = ref()
const files = ref<FileItem[]>([])
const uploading = ref(false)
const isDragover = ref(false)
const selectedUploadType = ref(props.uploadTypes[0]?.value || '')

// 对话框状态
const previewVisible = ref(false)
const historyVisible = ref(false)
const previewFile = ref<FileItem | null>(null)
const previewContent = ref('')

// 配置
const configCollapsed = ref(['config'])
const uploadConfig = reactive({
  chunkSize: props.chunkSize,
  timeout: 60000,
  retryCount: 3,
  autoRetry: true
})

// 上传历史
const uploadHistory = ref<UploadHistoryRecord[]>([
  {
    id: '1',
    fileName: 'gateway_config.json',
    fileSize: 2048,
    uploadTime: new Date(Date.now() - 3600000),
    status: 'success',
    type: 'config'
  }
])

// 文件状态映射
const fileStatusMappings = {
  ready: { type: 'info', text: '准备中' },
  uploading: { type: 'warning', text: '上传中' },
  success: { type: 'success', text: '成功' },
  error: { type: 'danger', text: '失败' }
}

// 配置表单字段
const configFields = [
  {
    key: 'chunkSize',
    label: '分片大小',
    type: 'select',
    options: [
      { label: '512KB', value: 512 * 1024 },
      { label: '1MB', value: 1024 * 1024 },
      { label: '2MB', value: 2 * 1024 * 1024 },
      { label: '5MB', value: 5 * 1024 * 1024 }
    ]
  },
  {
    key: 'timeout',
    label: '超时时间',
    type: 'number',
    unit: '毫秒',
    min: 10000,
    max: 300000
  },
  {
    key: 'retryCount',
    label: '重试次数',
    type: 'number',
    min: 0,
    max: 10
  },
  {
    key: 'autoRetry',
    label: '自动重试',
    type: 'switch'
  }
]

// 历史记录表格列
const historyColumns = [
  { key: 'fileName', label: '文件名', width: 200 },
  { key: 'fileSize', label: '大小', width: 100, formatter: (row: any) => formatFileSize(row.fileSize) },
  { key: 'type', label: '类型', width: 100 },
  { key: 'uploadTime', label: '上传时间', type: 'datetime', width: 160 },
  { key: 'status', label: '状态', width: 100, type: 'tag' },
  { key: 'actions', label: '操作', width: 100, type: 'action' }
]

// 计算属性
const containerClass = computed(() => {
  const classes = []
  
  classes.push(`file-uploader--${props.size}`)
  
  if (props.customClass) {
    classes.push(props.customClass)
  }
  
  return classes.join(' ')
})

const currentUploadType = computed(() => {
  return props.uploadTypes.find(type => type.value === selectedUploadType.value)
})

const currentAccept = computed(() => {
  return currentUploadType.value?.accept || '*'
})

const uploadTip = computed(() => {
  return currentUploadType.value?.description || '选择要上传的文件'
})

const uploadHeaders = computed(() => {
  return {
    'Content-Type': 'multipart/form-data',
    'X-Upload-Type': selectedUploadType.value
  }
})

const uploadData = computed(() => {
  return {
    type: selectedUploadType.value,
    chunkSize: uploadConfig.chunkSize
  }
})

const totalSize = computed(() => {
  const total = files.value.reduce((sum, file) => sum + file.size, 0)
  return formatFileSize(total)
})

const hasValidFiles = computed(() => {
  return files.value.some(file => file.status === 'ready')
})

// 方法
const formatFileSize = (size: number) => {
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  if (size < 1024 * 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`
  return `${(size / 1024 / 1024 / 1024).toFixed(1)} GB`
}

const formatFileTime = (timestamp?: number) => {
  if (!timestamp) return '未知'
  return new Date(timestamp).toLocaleString('zh-CN')
}

const getFileIcon = (file: FileItem) => {
  const ext = file.name.split('.').pop()?.toLowerCase()
  
  if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp'].includes(ext || '')) {
    return Picture
  }
  
  if (['mp4', 'avi', 'mov', 'wmv', 'flv'].includes(ext || '')) {
    return VideoPlay
  }
  
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext || '')) {
    return Files
  }
  
  return Document
}

const getFileItemClass = (file: FileItem) => {
  return `file-item--${file.status}`
}

const getFileURL = (file: FileItem) => {
  if (file.raw) {
    return URL.createObjectURL(file.raw)
  }
  return file.url || ''
}

const isImageFile = (file: FileItem) => {
  const imageTypes = ['image/jpeg', 'image/png', 'image/gif', 'image/bmp', 'image/webp']
  return imageTypes.includes(file.type)
}

const isTextFile = (file: FileItem) => {
  const textTypes = ['text/plain', 'application/json', 'text/xml', 'text/yaml']
  return textTypes.includes(file.type) || file.name.endsWith('.txt') || file.name.endsWith('.log')
}

const canPreview = (file: FileItem) => {
  return isImageFile(file) || isTextFile(file) || file.size < 1024 * 1024 // 小于1MB的文件允许预览
}

// 事件处理
const handleUploadTypeChange = () => {
  // 清空当前文件列表（如果类型改变）
  if (files.value.length > 0) {
    ElMessage.warning('更改上传类型将清空当前文件列表')
    files.value = []
  }
}

const handleBeforeUpload = (file: File) => {
  const currentType = currentUploadType.value
  if (!currentType) return false
  
  // 检查文件大小
  if (file.size > currentType.maxSize) {
    ElMessage.error(`文件大小超出限制：${formatFileSize(currentType.maxSize)}`)
    return false
  }
  
  // 检查文件类型
  const acceptedTypes = currentType.accept.split(',').map(type => type.trim())
  const fileExt = '.' + file.name.split('.').pop()?.toLowerCase()
  const isValidType = acceptedTypes.includes(fileExt) || acceptedTypes.includes('*')
  
  if (!isValidType) {
    ElMessage.error(`不支持的文件类型：${fileExt}`)
    return false
  }
  
  // 添加到文件列表
  const fileItem: FileItem = {
    uid: Date.now() + Math.random().toString(),
    name: file.name,
    size: file.size,
    type: file.type,
    lastModified: file.lastModified,
    status: 'ready',
    percentage: 0,
    raw: file
  }
  
  files.value.push(fileItem)
  
  return !props.autoUpload // 如果不是自动上传，阻止默认上传行为
}

const handleExceed = (files: File[]) => {
  ElMessage.warning(`最多只能上传 ${props.maxFiles} 个文件`)
}

const handleSuccess = (response: any, file: FileItem) => {
  const fileItem = files.value.find(f => f.uid === file.uid)
  if (fileItem) {
    fileItem.status = 'success'
    fileItem.percentage = 100
    fileItem.response = response
    fileItem.url = response.url
  }
  
  // 添加到历史记录
  const historyRecord: UploadHistoryRecord = {
    id: Date.now().toString(),
    fileName: file.name,
    fileSize: file.size,
    uploadTime: new Date(),
    status: 'success',
    type: selectedUploadType.value
  }
  uploadHistory.value.unshift(historyRecord)
  
  emit('success', file, response)
  ElMessage.success(`${file.name} 上传成功`)
}

const handleError = (error: any, file: FileItem) => {
  const fileItem = files.value.find(f => f.uid === file.uid)
  if (fileItem) {
    fileItem.status = 'error'
    fileItem.percentage = 0
  }
  
  emit('error', file, error)
  ElMessage.error(`${file.name} 上传失败`)
}

const handleProgress = (event: any, file: FileItem) => {
  const fileItem = files.value.find(f => f.uid === file.uid)
  if (fileItem) {
    fileItem.status = 'uploading'
    fileItem.percentage = Math.round(event.percent)
  }
  
  emit('progress', file, event.percent)
}

const customRequest = (options: any) => {
  // 自定义上传实现
  const file = options.file
  const fileItem = files.value.find(f => f.name === file.name)
  
  if (!fileItem) return
  
  // 模拟分片上传
  const chunkSize = uploadConfig.chunkSize
  const chunks = Math.ceil(file.size / chunkSize)
  let uploadedChunks = 0
  
  const uploadChunk = (chunkIndex: number) => {
    const start = chunkIndex * chunkSize
    const end = Math.min(start + chunkSize, file.size)
    const chunk = file.slice(start, end)
    
    // 模拟上传请求
    setTimeout(() => {
      uploadedChunks++
      const percentage = Math.round((uploadedChunks / chunks) * 100)
      
      fileItem.status = 'uploading'
      fileItem.percentage = percentage
      
      options.onProgress({ percent: percentage })
      
      if (uploadedChunks === chunks) {
        // 上传完成
        const response = {
          url: `/uploads/${file.name}`,
          message: '上传成功'
        }
        options.onSuccess(response)
      } else {
        // 继续上传下一个分片
        uploadChunk(chunkIndex + 1)
      }
    }, 100) // 模拟网络延迟
  }
  
  uploadChunk(0)
}

const handleAddMore = () => {
  uploadRef.value?.$el.querySelector('input').click()
}

const handleRemove = (file: FileItem, index: number) => {
  files.value.splice(index, 1)
}

const handleClearAll = () => {
  files.value = []
}

const handleUploadAll = () => {
  if (!hasValidFiles.value) return
  
  uploading.value = true
  const readyFiles = files.value.filter(file => file.status === 'ready')
  
  // 逐个上传文件
  readyFiles.forEach(file => {
    if (file.raw) {
      uploadRef.value?.submit(file.raw)
    }
  })
  
  // 监听所有文件上传完成
  const checkComplete = () => {
    const allComplete = readyFiles.every(file => 
      file.status === 'success' || file.status === 'error'
    )
    
    if (allComplete) {
      uploading.value = false
      emit('upload-complete', files.value)
    } else {
      setTimeout(checkComplete, 500)
    }
  }
  
  checkComplete()
}

const handleRetry = (file: FileItem) => {
  file.status = 'ready'
  file.percentage = 0
  
  if (file.raw) {
    uploadRef.value?.submit(file.raw)
  }
}

const handlePreview = (file: FileItem) => {
  previewFile.value = file
  previewContent.value = ''
  previewVisible.value = true
  
  // 如果是文本文件，读取内容
  if (isTextFile(file) && file.raw) {
    const reader = new FileReader()
    reader.onload = (e) => {
      previewContent.value = e.target?.result as string
    }
    reader.readAsText(file.raw)
  }
}

const handlePreviewClose = () => {
  previewVisible.value = false
  previewFile.value = null
  previewContent.value = ''
}

const handleHistoryClose = () => {
  historyVisible.value = false
}

const handleHistoryAction = (action: string, row: UploadHistoryRecord) => {
  if (action === 'delete') {
    const index = uploadHistory.value.findIndex(h => h.id === row.id)
    if (index > -1) {
      uploadHistory.value.splice(index, 1)
      ElMessage.success('历史记录删除成功')
    }
  }
}

const handleClearHistory = () => {
  uploadHistory.value = []
  ElMessage.success('历史记录已清空')
}

const handleConfigChange = () => {
  // 配置变更处理
}

// 监听
watch(() => props.uploadTypes, (newTypes) => {
  if (newTypes.length > 0 && !selectedUploadType.value) {
    selectedUploadType.value = newTypes[0].value
  }
}, { immediate: true })
</script>

<style scoped lang="scss">
.file-uploader {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 6px;
  
  &.file-uploader--small {
    .uploader-header {
      padding: 8px 12px;
    }
  }
  
  &.file-uploader--large {
    .uploader-header {
      padding: 16px 20px;
    }
  }
}

.uploader-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  
  .header-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--el-text-color-primary);
  }
  
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
}

.upload-area {
  padding: 16px;
  
  :deep(.upload-component) {
    width: 100%;
    
    .el-upload {
      width: 100%;
    }
    
    .el-upload-dragger {
      width: 100%;
      height: auto;
      min-height: 180px;
      border: 2px dashed var(--el-border-color);
      border-radius: 6px;
      background: var(--el-fill-color-lighter);
      transition: all 0.3s;
      
      &:hover,
      &.is-dragover {
        border-color: var(--el-color-primary);
        background: var(--el-color-primary-light-9);
      }
    }
  }
  
  .upload-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 20px;
    min-height: 140px;
    
    &.is-dragover {
      .upload-icon {
        color: var(--el-color-primary);
      }
    }
    
    .drag-area,
    .select-area {
      text-align: center;
      
      .upload-icon {
        color: var(--el-text-color-placeholder);
        margin-bottom: 16px;
      }
      
      .upload-text {
        .primary-text {
          font-size: 16px;
          color: var(--el-text-color-primary);
          margin-bottom: 8px;
        }
        
        .secondary-text {
          font-size: 14px;
          color: var(--el-text-color-secondary);
        }
      }
      
      .upload-tip {
        font-size: 12px;
        color: var(--el-text-color-secondary);
        margin-top: 8px;
      }
    }
    
    .file-preview {
      width: 100%;
      
      .file-summary {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px;
        background: var(--el-fill-color-light);
        border-radius: 4px;
        
        span {
          color: var(--el-text-color-primary);
          font-size: 14px;
        }
      }
    }
  }
}

.file-list {
  max-height: 400px;
  overflow-y: auto;
  border-top: 1px solid var(--el-border-color-lighter);
  
  .file-item {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--el-border-color-lighter);
    transition: background-color 0.3s;
    
    &:hover {
      background-color: var(--el-fill-color-light);
    }
    
    &:last-child {
      border-bottom: none;
    }
    
    &.file-item--error {
      background-color: var(--el-color-danger-light-9);
    }
    
    &.file-item--success {
      background-color: var(--el-color-success-light-9);
    }
    
    .file-info {
      display: flex;
      align-items: center;
      flex: 1;
      min-width: 0;
      
      .file-icon {
        margin-right: 12px;
        color: var(--el-text-color-secondary);
      }
      
      .file-details {
        flex: 1;
        min-width: 0;
        
        .file-name {
          font-size: 14px;
          color: var(--el-text-color-primary);
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
          margin-bottom: 4px;
        }
        
        .file-meta {
          font-size: 12px;
          color: var(--el-text-color-secondary);
          
          .file-size,
          .file-time {
            margin-right: 12px;
          }
        }
      }
    }
    
    .file-status {
      margin: 0 16px;
      min-width: 100px;
      
      .upload-progress {
        display: flex;
        align-items: center;
        gap: 8px;
        
        .el-progress {
          flex: 1;
        }
        
        .progress-text {
          font-size: 12px;
          color: var(--el-text-color-secondary);
          white-space: nowrap;
        }
      }
    }
    
    .file-actions {
      display: flex;
      gap: 4px;
    }
  }
}

.batch-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  
  .batch-info {
    font-size: 13px;
    color: var(--el-text-color-secondary);
  }
  
  .batch-buttons {
    display: flex;
    gap: 8px;
  }
}

.upload-config {
  border-top: 1px solid var(--el-border-color-lighter);
  
  :deep(.el-collapse) {
    border: none;
    
    .el-collapse-item__header {
      padding: 0 16px;
    }
    
    .el-collapse-item__wrap {
      border: none;
      
      .el-collapse-item__content {
        padding: 16px;
      }
    }
  }
}

.file-preview-content {
  .image-preview {
    text-align: center;
    
    img {
      border-radius: 4px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }
  }
  
  .text-preview {
    :deep(.el-textarea__inner) {
      font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
      font-size: 12px;
      line-height: 1.5;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .file-uploader {
    .uploader-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
    
    .file-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
      
      .file-info {
        width: 100%;
      }
      
      .file-status {
        width: 100%;
        margin: 0;
      }
      
      .file-actions {
        width: 100%;
        justify-content: flex-end;
      }
    }
    
    .batch-actions {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
  }
}
</style>