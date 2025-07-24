<template>
  <div class="diagnostic-page">
    <el-card header="前端诊断页面">
      <h2>系统状态检查</h2>
      
      <el-descriptions title="环境信息" border :column="2">
        <el-descriptions-item label="Vue版本">{{ vueVersion }}</el-descriptions-item>
        <el-descriptions-item label="Router状态">{{ routerStatus }}</el-descriptions-item>
        <el-descriptions-item label="Store状态">{{ storeStatus }}</el-descriptions-item>
        <el-descriptions-item label="API连接">{{ apiStatus }}</el-descriptions-item>
      </el-descriptions>

      <h3>路由测试</h3>
      <div class="route-tests">
        <el-button 
          v-for="route in testRoutes" 
          :key="route.path"
          @click="navigateTo(route.path)"
          :type="route.tested ? 'success' : 'primary'"
          size="small"
          style="margin: 4px;"
        >
          {{ route.name }} {{ route.tested ? '✓' : '' }}
        </el-button>
      </div>

      <h3>组件测试</h3>
      <div class="component-tests">
        <el-alert title="图标测试" type="info">
          <el-icon><Check /></el-icon>
          <el-icon><Warning /></el-icon>
          <el-icon><Connection /></el-icon>
          <el-icon><Monitor /></el-icon>
        </el-alert>
      </div>

      <h3>API测试</h3>
      <el-button @click="testApiCalls" :loading="apiTesting">测试API调用</el-button>
      <div v-if="apiResults.length > 0" class="api-results">
        <el-table :data="apiResults" style="width: 100%; margin-top: 16px;">
          <el-table-column prop="endpoint" label="端点" />
          <el-table-column prop="status" label="状态" />
          <el-table-column prop="message" label="结果" />
        </el-table>
      </div>

      <h3>错误日志</h3>
      <el-button @click="clearErrors">清除错误</el-button>
      <div v-if="errors.length > 0" class="error-log">
        <el-alert 
          v-for="(error, index) in errors" 
          :key="index"
          :title="error.message"
          type="error"
          :description="error.stack"
          :closable="false"
          style="margin-bottom: 8px;"
        />
      </div>
      <el-empty v-else description="暂无错误" />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { systemApi, driversApi, alertsApi } from '@/services/api'
import { ElMessage } from 'element-plus'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const vueVersion = ref('3.4.0')
const routerStatus = ref('未知')
const storeStatus = ref('未知')
const apiStatus = ref('未知')
const apiTesting = ref(false)
const errors = ref<Array<{message: string, stack?: string}>>([])
const apiResults = ref<Array<{endpoint: string, status: string, message: string}>>([])

const testRoutes = ref([
  { path: '/dashboard', name: '仪表板', tested: false },
  { path: '/realtime', name: '实时数据', tested: false },
  { path: '/drivers', name: '驱动管理', tested: false },
  { path: '/connectors', name: '连接器', tested: false },
  { path: '/data-points', name: '数据点', tested: false },
  { path: '/alerts', name: '告警', tested: false },
  { path: '/analytics', name: '分析', tested: false },
  { path: '/system', name: '系统', tested: false },
  { path: '/monitoring', name: '监控', tested: false }
])

const navigateTo = (path: string) => {
  try {
    router.push(path)
    const routeItem = testRoutes.value.find(r => r.path === path)
    if (routeItem) {
      routeItem.tested = true
    }
    ElMessage.success(`导航到 ${path} 成功`)
  } catch (error: any) {
    ElMessage.error(`导航失败: ${error.message}`)
    errors.value.push({
      message: `导航到 ${path} 失败`,
      stack: error.stack
    })
  }
}

const testApiCalls = async () => {
  apiTesting.value = true
  apiResults.value = []

  const testApis = [
    { name: '健康检查', call: () => systemApi.getSystemMetrics() },
    { name: '系统信息', call: () => systemApi.getSystemInfo() },
    { name: '驱动列表', call: () => driversApi.getDrivers() },
    { name: '告警列表', call: () => alertsApi.getAlerts() }
  ]

  for (const api of testApis) {
    try {
      const response = await api.call()
      apiResults.value.push({
        endpoint: api.name,
        status: 'success',
        message: `状态: ${response.success ? '成功' : '失败'}`
      })
    } catch (error: any) {
      apiResults.value.push({
        endpoint: api.name,
        status: 'error',
        message: error.message
      })
    }
  }

  apiTesting.value = false
}

const clearErrors = () => {
  errors.value = []
}

// 捕获全局错误
window.addEventListener('error', (event) => {
  errors.value.push({
    message: event.message,
    stack: event.error?.stack
  })
})

// 捕获未处理的Promise拒绝
window.addEventListener('unhandledrejection', (event) => {
  errors.value.push({
    message: `未处理的Promise拒绝: ${event.reason}`,
    stack: event.reason?.stack
  })
})

onMounted(async () => {
  // 检查路由状态
  try {
    routerStatus.value = router.currentRoute.value ? '正常' : '异常'
  } catch (error) {
    routerStatus.value = '错误'
  }

  // 检查store状态
  try {
    storeStatus.value = authStore ? '正常' : '异常'
  } catch (error) {
    storeStatus.value = '错误'
  }

  // 检查API状态
  try {
    await systemApi.getSystemMetrics()
    apiStatus.value = '连接正常'
  } catch (error) {
    apiStatus.value = 'API连接失败'
  }
})
</script>

<style scoped lang="scss">
.diagnostic-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;

  .route-tests {
    margin: 16px 0;
  }

  .component-tests {
    margin: 16px 0;
  }

  .api-results {
    margin-top: 16px;
  }

  .error-log {
    margin-top: 16px;
    max-height: 400px;
    overflow-y: auto;
  }
}
</style>