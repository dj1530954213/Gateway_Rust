<template>
  <div class="test-mode-page">
    <el-card class="test-card">
      <template #header>
        <div class="card-header">
          <h2>🔧 测试模式验证</h2>
          <el-tag type="warning" size="large">
            测试环境
          </el-tag>
        </div>
      </template>

      <div class="test-content">
        <el-alert
          title="测试模式已启用"
          type="success"
          :closable="false"
          show-icon
        >
          <p>当前系统运行在测试模式下，权限验证已被绕过，所有API调用将被模拟。</p>
        </el-alert>

        <div class="test-sections">
          <!-- 认证状态测试 -->
          <el-card class="test-section">
            <template #header>
              <h3>认证状态检查</h3>
            </template>
            <div class="test-items">
              <div class="test-item">
                <span>是否已认证:</span>
                <el-tag :type="isAuthenticated ? 'success' : 'danger'">
                  {{ isAuthenticated ? '是' : '否' }}
                </el-tag>
              </div>
              <div class="test-item">
                <span>当前用户:</span>
                <el-tag>{{ currentUser?.username || '无' }}</el-tag>
              </div>
              <div class="test-item">
                <span>用户角色:</span>
                <el-tag>{{ currentUser?.role || '无' }}</el-tag>
              </div>
            </div>
          </el-card>

          <!-- 权限测试 -->
          <el-card class="test-section">
            <template #header>
              <h3>权限检查测试</h3>
            </template>
            <div class="test-items">
              <div class="test-item">
                <span>设备管理权限:</span>
                <el-tag :type="hasDevicePermission.value ? 'success' : 'danger'">
                  {{ hasDevicePermission.value ? '有权限' : '无权限' }}
                </el-tag>
              </div>
              <div class="test-item">
                <span>用户管理权限:</span>
                <el-tag :type="hasUserPermission.value ? 'success' : 'danger'">
                  {{ hasUserPermission.value ? '有权限' : '无权限' }}
                </el-tag>
              </div>
              <div class="test-item">
                <span>系统配置权限:</span>
                <el-tag :type="hasSystemPermission.value ? 'success' : 'danger'">
                  {{ hasSystemPermission.value ? '有权限' : '无权限' }}
                </el-tag>
              </div>
              <div class="test-item">
                <span>是否为管理员:</span>
                <el-tag :type="isAdminRole.value ? 'success' : 'danger'">
                  {{ isAdminRole.value ? '是' : '否' }}
                </el-tag>
              </div>
            </div>
          </el-card>

          <!-- API 调用测试 -->
          <el-card class="test-section">
            <template #header>
              <h3>API 调用测试</h3>
            </template>
            <div class="test-actions">
              <el-button @click="testLogActivity" type="primary">
                测试活动记录
              </el-button>
              <el-button @click="testGetSessions" type="primary">
                测试会话获取
              </el-button>
              <el-button @click="testGetActivities" type="primary">
                测试活动获取
              </el-button>
            </div>
            <div class="test-results" v-if="testResults.length > 0">
              <h4>测试结果:</h4>
              <div v-for="result in testResults" :key="result.id" class="test-result">
                <el-tag :type="result.success ? 'success' : 'danger'">
                  {{ result.action }}: {{ result.success ? '成功' : '失败' }}
                </el-tag>
                <span class="result-message">{{ result.message }}</span>
              </div>
            </div>
          </el-card>

          <!-- 系统信息 -->
          <el-card class="test-section">
            <template #header>
              <h3>系统信息</h3>
            </template>
            <div class="system-info">
              <div class="info-item">
                <span>测试模式:</span>
                <el-tag type="warning">{{ isTestMode ? '已启用' : '已禁用' }}</el-tag>
              </div>
              <div class="info-item">
                <span>环境变量 VITE_BYPASS_AUTH:</span>
                <el-tag>{{ bypassAuthEnv }}</el-tag>
              </div>
              <div class="info-item">
                <span>当前权限列表:</span>
                <el-tag v-for="permission in userPermissions" :key="permission">
                  {{ permission }}
                </el-tag>
              </div>
            </div>
          </el-card>
        </div>

        <div class="test-footer">
          <el-alert
            title="注意"
            type="warning"
            :closable="false"
          >
            <p>测试模式仅用于开发和调试，请确保在生产环境中禁用此模式。</p>
            <p>要禁用测试模式，请在 .env.development 中设置 VITE_BYPASS_AUTH=false</p>
          </el-alert>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
/**
 * TestMode.vue —— 测试模式验证页面
 * 
 * 📝 Responsibilities:
 *  1. 验证测试模式是否正确启用
 *  2. 测试权限检查功能
 *  3. 测试API调用模拟
 *  4. 显示系统状态信息
 *
 * 📦 Dependencies:
 *  - useAuthStore
 *  - usePermission
 *
 * 🔄 Update Log:
 *  - 2025-07-30  创建测试模式验证页面
 */

import { ref, computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { usePermission } from '@/composables/usePermission'

const authStore = useAuthStore()
const {
  hasPermission,
  hasRole,
  isAuthenticated,
  currentUser,
  userPermissions
} = usePermission()

// 测试模式检查
const isTestMode = import.meta.env.VITE_BYPASS_AUTH === 'true'
const bypassAuthEnv = import.meta.env.VITE_BYPASS_AUTH

// 权限测试
const hasDevicePermission = hasPermission('devices:read')
const hasUserPermission = hasPermission('users:read')
const hasSystemPermission = hasPermission('system:config')
const isAdminRole = hasRole('admin')

// 测试结果
const testResults = ref<Array<{
  id: string
  action: string
  success: boolean
  message: string
}>>([])

// 测试API调用
const testLogActivity = async () => {
  try {
    await authStore.logActivity({
      type: 'test',
      action: '测试活动记录',
      target: 'test-page',
      details: { message: '这是一个测试活动记录' }
    })
    
    testResults.value.push({
      id: Date.now().toString(),
      action: '活动记录',
      success: true,
      message: '成功记录测试活动'
    })
  } catch (error: any) {
    testResults.value.push({
      id: Date.now().toString(),
      action: '活动记录',
      success: false,
      message: error.message || '记录失败'
    })
  }
}

const testGetSessions = async () => {
  try {
    await authStore.getSessions()
    
    testResults.value.push({
      id: Date.now().toString(),
      action: '会话获取',
      success: true,
      message: `成功获取 ${authStore.sessions.length} 个会话`
    })
  } catch (error: any) {
    testResults.value.push({
      id: Date.now().toString(),
      action: '会话获取',
      success: false,
      message: error.message || '获取失败'
    })
  }
}

const testGetActivities = async () => {
  try {
    await authStore.getActivities()
    
    testResults.value.push({
      id: Date.now().toString(),
      action: '活动获取',
      success: true,
      message: `成功获取 ${authStore.activities.length} 条活动记录`
    })
  } catch (error: any) {
    testResults.value.push({
      id: Date.now().toString(),
      action: '活动获取',
      success: false,
      message: error.message || '获取失败'
    })
  }
}
</script>

<style scoped>
.test-mode-page {
  padding: 20px;
}

.test-card {
  max-width: 1200px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  margin: 0;
  color: #409eff;
}

.test-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.test-sections {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
  gap: 20px;
}

.test-section {
  height: fit-content;
}

.test-items {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.test-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  background: #f5f7fa;
  border-radius: 4px;
}

.test-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.test-results {
  margin-top: 20px;
  padding: 15px;
  background: #f5f7fa;
  border-radius: 4px;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.result-message {
  color: #606266;
  font-size: 14px;
}

.system-info {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  background: #f5f7fa;
  border-radius: 4px;
}

.test-footer {
  margin-top: 20px;
}
</style>