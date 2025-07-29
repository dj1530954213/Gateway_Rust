<template>
  <div class="route-test-container">
    <el-card>
      <template #header>
        <h2>🔧 路由测试工具</h2>
        <p>用于验证所有路由是否正确配置</p>
      </template>

      <!-- 测试结果 -->
      <div class="test-results">
        <h3>路由测试结果：</h3>
        <el-table :data="routeTests" style="width: 100%">
          <el-table-column prop="path" label="路由路径" width="200" />
          <el-table-column prop="name" label="页面名称" width="150" />
          <el-table-column label="状态" width="120">
            <template #default="scope">
              <el-tag :type="scope.row.working ? 'success' : 'danger'">
                {{ scope.row.working ? '✅ 正常' : '❌ 404' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作">
            <template #default="scope">
              <el-button type="primary" size="small" @click="testRoute(scope.row.path)">
                测试访问
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 当前路由信息 -->
      <el-divider />
      <div class="current-route-info">
        <h3>当前路由信息：</h3>
        <p><strong>路径:</strong> {{ $route.path }}</p>
        <p><strong>名称:</strong> {{ $route.name }}</p>
        <p><strong>参数:</strong> {{ JSON.stringify($route.params) }}</p>
        <p><strong>查询:</strong> {{ JSON.stringify($route.query) }}</p>
        <p><strong>认证状态:</strong> {{ authStore.isAuthenticated ? '✅ 已登录' : '❌ 未登录' }}</p>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { ElMessage } from 'element-plus'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

// 路由测试数据
const routeTests = ref([
  { path: '/dashboard', name: '仪表板', working: false },
  { path: '/realtime', name: '实时数据', working: false },
  { path: '/drivers', name: '驱动管理', working: false },
  { path: '/devices', name: '设备管理', working: false },
  { path: '/connectors', name: '连接器管理', working: false },
  { path: '/connectors/list', name: '连接器列表', working: false },
  { path: '/tags', name: '点位管理', working: false },
  { path: '/alerts', name: '告警管理', working: false },
  { path: '/alerts/list', name: '告警列表', working: false },
  { path: '/alerts/rules', name: '告警规则', working: false },
  { path: '/analytics', name: '数据分析', working: false },
  { path: '/system', name: '系统管理', working: false },
  { path: '/system/config', name: '系统配置', working: false },
  { path: '/monitoring', name: '监控中心', working: false },
  { path: '/monitoring/health', name: '健康状态', working: false },
])

// 测试路由访问
const testRoute = async (path: string) => {
  try {
    await router.push(path)
    ElMessage.success(`成功访问路由: ${path}`)
    
    // 更新测试状态
    const testItem = routeTests.value.find(item => item.path === path)
    if (testItem) {
      testItem.working = true
    }
  } catch (error) {
    ElMessage.error(`路由访问失败: ${path}`)
    console.error('Route test failed:', error)
    
    // 更新测试状态
    const testItem = routeTests.value.find(item => item.path === path)
    if (testItem) {
      testItem.working = false
    }
  }
}

// 批量测试所有路由
const testAllRoutes = async () => {
  const currentPath = route.path
  
  for (const test of routeTests.value) {
    try {
      await router.push(test.path)
      await new Promise(resolve => setTimeout(resolve, 100)) // 短暂延迟
      test.working = !route.name?.toString().includes('NotFound')
    } catch (error) {
      test.working = false
    }
  }
  
  // 返回到原始路径
  await router.push(currentPath)
  ElMessage.success('路由测试完成')
}

onMounted(() => {
  // 页面加载时进行基本检查
  console.log('Route test page loaded')
  console.log('Current route:', route)
  console.log('Auth status:', authStore.isAuthenticated)
})
</script>

<style scoped>
.route-test-container {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.test-results {
  margin-bottom: 20px;
}

.current-route-info {
  background: #f5f7fa;
  padding: 15px;
  border-radius: 8px;
}

.current-route-info p {
  margin: 8px 0;
}
</style>