<template>
  <div class="system-config">
    <el-card class="header-card">
      <div class="config-header">
        <h2>系统配置管理</h2>
        <div class="header-actions">
          <el-button 
            type="primary" 
            :icon="Refresh" 
            @click="loadConfig"
            :loading="loading"
          >
            刷新配置
          </el-button>
          <el-button 
            type="success" 
            :icon="Check" 
            @click="saveConfig"
            :loading="saving"
            :disabled="!hasChanges"
          >
            保存配置
          </el-button>
          <el-button 
            type="warning" 
            :icon="RefreshRight" 
            @click="resetConfig"
            :disabled="!hasChanges"
          >
            重置更改
          </el-button>
        </div>
      </div>
    </el-card>

    <el-row :gutter="20">
      <!-- 系统基础配置 -->
      <el-col :span="12">
        <el-card title="系统基础配置" class="config-card">
          <template #header>
            <span>系统基础配置</span>
          </template>
          
          <el-form :model="config.system" label-width="120px">
            <el-form-item label="系统名称">
              <el-input 
                v-model="config.system.name" 
                placeholder="请输入系统名称"
                @input="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="系统描述">
              <el-input 
                v-model="config.system.description" 
                type="textarea"
                placeholder="请输入系统描述"
                :rows="3"
                @input="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="日志级别">
              <el-select v-model="config.system.log_level" @change="markChanged">
                <el-option label="TRACE" value="trace" />
                <el-option label="DEBUG" value="debug" />
                <el-option label="INFO" value="info" />
                <el-option label="WARN" value="warn" />
                <el-option label="ERROR" value="error" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="线程池大小">
              <el-input-number 
                v-model="config.system.thread_pool_size"
                :min="1" 
                :max="32"
                @change="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="最大连接数">
              <el-input-number 
                v-model="config.system.max_connections"
                :min="10" 
                :max="10000"
                @change="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="API端口">
              <el-input-number 
                v-model="config.system.api_port"
                :min="1024" 
                :max="65535"
                @change="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="Web端口">
              <el-input-number 
                v-model="config.system.web_port"
                :min="1024" 
                :max="65535"
                @change="markChanged"
              />
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <!-- 安全配置 -->
      <el-col :span="12">
        <el-card title="安全配置" class="config-card">
          <template #header>
            <span>安全配置</span>
          </template>
          
          <el-form :model="config.security" label-width="120px">
            <el-form-item label="启用身份验证">
              <el-switch 
                v-model="config.security.auth_enabled"
                @change="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="JWT密钥" v-if="config.security.auth_enabled">
              <el-input 
                v-model="config.security.jwt_secret" 
                type="password"
                placeholder="JWT签名密钥"
                show-password
                @input="markChanged"
              />
              <div class="form-tip">
                建议使用32位以上的随机字符串
              </div>
            </el-form-item>
            
            <el-form-item label="Token有效期" v-if="config.security.auth_enabled">
              <el-input-number 
                v-model="config.security.token_expiry"
                :min="300" 
                :max="86400"
                @change="markChanged"
              />
              <span class="input-suffix">秒</span>
            </el-form-item>
            
            <el-form-item label="启用TLS">
              <el-switch 
                v-model="config.security.tls_enabled"
                @change="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="证书文件" v-if="config.security.tls_enabled">
              <el-input 
                v-model="config.security.cert_file" 
                placeholder="证书文件路径"
                @input="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="私钥文件" v-if="config.security.tls_enabled">
              <el-input 
                v-model="config.security.key_file" 
                type="password"
                placeholder="私钥文件路径"
                show-password
                @input="markChanged"
              />
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20" class="mt-20">
      <!-- 监控配置 -->
      <el-col :span="12">
        <el-card title="监控配置" class="config-card">
          <template #header>
            <span>监控配置</span>
          </template>
          
          <el-form :model="config.monitoring" label-width="120px">
            <el-form-item label="启用监控">
              <el-switch 
                v-model="config.monitoring.enabled"
                @change="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="指标端口" v-if="config.monitoring.enabled">
              <el-input-number 
                v-model="config.monitoring.metrics_port"
                :min="1024" 
                :max="65535"
                @change="markChanged"
              />
            </el-form-item>
            
            <el-form-item label="健康检查间隔" v-if="config.monitoring.enabled">
              <el-input-number 
                v-model="config.monitoring.health_check_interval"
                :min="5" 
                :max="300"
                @change="markChanged"
              />
              <span class="input-suffix">秒</span>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <!-- 高级配置 -->
      <el-col :span="12">
        <el-card title="高级功能配置" class="config-card">
          <template #header>
            <span>高级功能配置</span>
          </template>
          
          <el-form :model="config.advanced" label-width="120px">
            <el-form-item label="机器学习">
              <el-switch 
                v-model="config.advanced.ml_enabled"
                @change="markChanged"
              />
              <div class="form-tip">启用边缘机器学习功能</div>
            </el-form-item>
            
            <el-form-item label="数据分析">
              <el-switch 
                v-model="config.advanced.analytics_enabled"
                @change="markChanged"
              />
              <div class="form-tip">启用实时数据分析</div>
            </el-form-item>
            
            <el-form-item label="边缘计算">
              <el-switch 
                v-model="config.advanced.edge_compute_enabled"
                @change="markChanged"
              />
              <div class="form-tip">启用边缘计算运行时</div>
            </el-form-item>
            
            <el-form-item label="性能优化">
              <el-switch 
                v-model="config.advanced.optimization_enabled"
                @change="markChanged"
              />
              <div class="form-tip">启用自动性能优化</div>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>
    </el-row>

    <!-- 配置预览和导入导出 -->
    <el-card class="config-preview mt-20">
      <template #header>
        <div class="preview-header">
          <span>配置预览</span>
          <div class="preview-actions">
            <el-button 
              type="info" 
              :icon="Download" 
              @click="exportConfig"
            >
              导出配置
            </el-button>
            <el-button 
              type="warning" 
              :icon="Upload" 
              @click="$refs.fileInput.click()"
            >
              导入配置
            </el-button>
            <input 
              ref="fileInput" 
              type="file" 
              accept=".json,.yaml,.yml" 
              style="display: none"
              @change="importConfig"
            />
          </div>
        </div>
      </template>
      
      <el-tabs v-model="previewTab">
        <el-tab-pane label="JSON格式" name="json">
          <div class="config-editor">
            <pre>{{ formatConfig('json') }}</pre>
          </div>
        </el-tab-pane>
        <el-tab-pane label="YAML格式" name="yaml">
          <div class="config-editor">
            <pre>{{ formatConfig('yaml') }}</pre>
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { 
  Refresh, 
  Check, 
  RefreshRight, 
  Download, 
  Upload 
} from '@element-plus/icons-vue'
import { systemApi } from '@/services/api'
import type { GatewayConfig } from '@/types'

// 状态管理
const loading = ref(false)
const saving = ref(false)
const hasChanges = ref(false)
const previewTab = ref('json')

// 原始配置和当前配置
const originalConfig = ref<GatewayConfig | null>(null)
const config = reactive<GatewayConfig>({
  system: {
    name: '',
    description: '',
    log_level: 'info',
    thread_pool_size: 4,
    max_connections: 1000,
    api_port: 8080,
    web_port: 3000
  },
  drivers: {},
  connectors: {},
  security: {
    auth_enabled: true,
    jwt_secret: '',
    token_expiry: 3600,
    tls_enabled: false,
    cert_file: '',
    key_file: ''
  },
  monitoring: {
    enabled: true,
    metrics_port: 9090,
    health_check_interval: 30,
    alert_rules: []
  },
  advanced: {
    ml_enabled: false,
    analytics_enabled: true,
    edge_compute_enabled: false,
    optimization_enabled: true
  }
})

// 计算属性
const formatConfig = computed(() => {
  return (format: 'json' | 'yaml') => {
    if (format === 'json') {
      return JSON.stringify(config, null, 2)
    } else {
      // 简化的YAML格式
      return `system:
  name: "${config.system.name}"
  description: "${config.system.description}"
  log_level: ${config.system.log_level}
  thread_pool_size: ${config.system.thread_pool_size}
  max_connections: ${config.system.max_connections}
  api_port: ${config.system.api_port}
  web_port: ${config.system.web_port}

security:
  auth_enabled: ${config.security.auth_enabled}
  jwt_secret: "${config.security.jwt_secret}"
  token_expiry: ${config.security.token_expiry}
  tls_enabled: ${config.security.tls_enabled}
  cert_file: "${config.security.cert_file || ''}"
  key_file: "${config.security.key_file || ''}"

monitoring:
  enabled: ${config.monitoring.enabled}
  metrics_port: ${config.monitoring.metrics_port}
  health_check_interval: ${config.monitoring.health_check_interval}

advanced:
  ml_enabled: ${config.advanced.ml_enabled}
  analytics_enabled: ${config.advanced.analytics_enabled}
  edge_compute_enabled: ${config.advanced.edge_compute_enabled}
  optimization_enabled: ${config.advanced.optimization_enabled}`
    }
  }
})

// 方法
const markChanged = () => {
  hasChanges.value = true
}

const loadConfig = async () => {
  loading.value = true
  try {
    const response = await systemApi.getGatewayConfig()
    if (response.success && response.data) {
      originalConfig.value = response.data
      Object.assign(config, response.data)
      hasChanges.value = false
      ElMessage.success('配置加载成功')
    }
  } catch (error) {
    ElMessage.error(`配置加载失败: ${  error.message}`)
  } finally {
    loading.value = false
  }
}

const saveConfig = async () => {
  try {
    await ElMessageBox.confirm(
      '确认保存当前配置？这将重启系统服务。',
      '确认保存',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    saving.value = true
    const response = await systemApi.updateGatewayConfig(config)
    if (response.success) {
      originalConfig.value = { ...config }
      hasChanges.value = false
      ElMessage.success('配置保存成功，系统将在5秒后重启')
      
      // 提示系统重启
      setTimeout(() => {
        ElMessage.info('系统正在重启中，请稍候...')
      }, 2000)
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`配置保存失败: ${  error.message}`)
    }
  } finally {
    saving.value = false
  }
}

const resetConfig = () => {
  if (originalConfig.value) {
    Object.assign(config, originalConfig.value)
    hasChanges.value = false
    ElMessage.success('配置已重置')
  }
}

const exportConfig = () => {
  const dataStr = JSON.stringify(config, null, 2)
  const dataUri = `data:application/json;charset=utf-8,${ encodeURIComponent(dataStr)}`
  
  const exportFileDefaultName = `gateway-config-${new Date().toISOString().split('T')[0]}.json`
  
  const linkElement = document.createElement('a')
  linkElement.setAttribute('href', dataUri)
  linkElement.setAttribute('download', exportFileDefaultName)
  linkElement.click()
  
  ElMessage.success('配置导出成功')
}

const importConfig = (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  
  const reader = new FileReader()
  reader.onload = (e) => {
    try {
      const importedConfig = JSON.parse(e.target?.result as string)
      Object.assign(config, importedConfig)
      markChanged()
      ElMessage.success('配置导入成功')
    } catch (error) {
      ElMessage.error('配置文件格式错误')
    }
  }
  reader.readAsText(file)
}

// 生命周期
onMounted(() => {
  loadConfig()
})
</script>

<style scoped lang="scss">
.system-config {
  padding: 20px;
  
  .header-card {
    margin-bottom: 20px;
    
    .config-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      h2 {
        margin: 0;
        color: var(--el-text-color-primary);
      }
      
      .header-actions {
        display: flex;
        gap: 10px;
      }
    }
  }
  
  .config-card {
    height: fit-content;
    
    .form-tip {
      font-size: 12px;
      color: var(--el-text-color-placeholder);
      margin-top: 4px;
    }
    
    .input-suffix {
      margin-left: 8px;
      color: var(--el-text-color-regular);
    }
  }
  
  .config-preview {
    .preview-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .preview-actions {
        display: flex;
        gap: 10px;
      }
    }
    
    .config-editor {
      background: var(--el-fill-color-light);
      border-radius: 4px;
      padding: 16px;
      margin-top: 16px;
      max-height: 400px;
      overflow-y: auto;
      
      pre {
        margin: 0;
        font-family: 'Courier New', monospace;
        font-size: 14px;
        line-height: 1.4;
        color: var(--el-text-color-primary);
      }
    }
  }
  
  .mt-20 {
    margin-top: 20px;
  }
}

:deep(.el-card__header) {
  padding: 16px 20px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

:deep(.el-form-item__label) {
  font-weight: 500;
}

:deep(.el-input-number) {
  width: 100%;
}
</style>