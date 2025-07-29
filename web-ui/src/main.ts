import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

import App from './App.vue'
import router from './router'
import '@/assets/styles/main.scss'

// Permission management
import { createPermissionDirective, createPermissionGuard } from '@/composables/usePermission'
import { useAuthStore } from '@/stores/auth'

const app = createApp(App)
const pinia = createPinia()

// Global error handler
app.config.errorHandler = (error, instance, info) => {
  console.error('Global error:', error)
  console.error('Error info:', info)
  
  // 特别处理DOM相关错误
  if (error.message?.includes('parentNode') || 
      error.message?.includes('Cannot read properties of null') ||
      error.message?.includes('Cannot read properties of undefined')) {
    console.warn('DOM access error detected, component may need better cleanup')
  }
  
  // 在开发模式下不中断应用
  if (import.meta.env.MODE === 'development') {
    console.warn('Error caught by global handler, continuing...')
  }
}

// Register Element Plus icons
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

// Use Pinia first so stores are available
app.use(pinia)

// Initialize auth store
const authStore = useAuthStore()
authStore.init()

// Register permission directive
app.directive('permission', createPermissionDirective())

// Setup router with permission guard
router.beforeEach(createPermissionGuard())

app.use(router)
app.use(ElementPlus)

app.mount('#app')