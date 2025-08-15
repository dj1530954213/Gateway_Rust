<template>
  <div class="drivers-view">
    <div class="drivers-tabs">
      <el-tabs v-model="activeTab" @tab-click="handleTabClick">
        <el-tab-pane label="驱动配置" name="configs">
          <template #label>
            <span class="tab-label">
              <el-icon><Setting /></el-icon>
              驱动配置
            </span>
          </template>
        </el-tab-pane>
        <el-tab-pane label="驱动文件" name="list">
          <template #label>
            <span class="tab-label">
              <el-icon><Files /></el-icon>
              驱动文件
            </span>
          </template>
        </el-tab-pane>
      </el-tabs>
    </div>
    <div class="drivers-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Setting, Files } from '@element-plus/icons-vue'
import { ref, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const activeTab = ref('configs')

// 根据当前路由设置活动标签
watch(
  () => route.name,
  newName => {
    if (newName === 'DriversList') {
      activeTab.value = 'list'
    } else if (newName === 'DriverConfigs') {
      activeTab.value = 'configs'
    }
  },
  { immediate: true }
)

// 处理标签切换
const handleTabClick = (tab: any) => {
  const tabName = tab.paneName
  if (tabName === 'configs') {
    router.push('/drivers/configs')
  } else if (tabName === 'list') {
    router.push('/drivers/list')
  }
}
</script>

<style scoped lang="scss">
.drivers-view {
  padding: 16px;
  background: var(--el-bg-color-page);
  min-height: 100vh;
}

.drivers-tabs {
  margin-bottom: 16px;

  .tab-label {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  :deep(.el-tabs__header) {
    margin-bottom: 0;
    background: var(--el-bg-color);
    padding: 0 16px;
    border-radius: 8px 8px 0 0;
  }
}

.drivers-content {
  background: var(--el-bg-color);
  border-radius: 0 0 8px 8px;
  min-height: calc(100vh - 120px);
}
</style>
