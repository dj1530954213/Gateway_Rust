<template>
  <div class="not-found-container">
    <div class="not-found-content">
      <div class="error-image">
        <el-icon :size="120" color="#c6e2ff">
          <WarningFilled />
        </el-icon>
      </div>

      <h1 class="error-title">404</h1>
      <h2 class="error-subtitle">页面未找到</h2>
      <p class="error-description">抱歉，您访问的页面不存在或已被移除。</p>

      <div class="error-actions">
        <el-button type="primary" @click="goHome">
          <el-icon><House /></el-icon>
          返回首页
        </el-button>
        <el-button @click="goBack">
          <el-icon><Back /></el-icon>
          返回上页
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const goHome = () => {
  router.push('/')
}

const goBack = () => {
  router.go(-1)
}
</script>

<style scoped lang="scss">
.not-found-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--el-bg-color-page);
}

.not-found-content {
  text-align: center;
  max-width: 500px;
  padding: 40px;

  .error-image {
    margin-bottom: 30px;
  }

  .error-title {
    font-size: 72px;
    font-weight: 300;
    color: var(--el-color-primary);
    margin: 0 0 20px 0;
    line-height: 1;
  }

  .error-subtitle {
    font-size: 24px;
    color: var(--el-text-color-primary);
    margin: 0 0 16px 0;
  }

  .error-description {
    color: var(--el-text-color-secondary);
    margin-bottom: 40px;
    line-height: 1.6;
  }

  .error-actions {
    display: flex;
    gap: 16px;
    justify-content: center;
  }
}
</style>
