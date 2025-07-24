<template>
  <div class="login-container">
    <div class="login-wrapper">
      <div class="login-left">
        <div class="brand-section">
          <div class="logo">
            <img src="/logo.svg" alt="Edge Gateway" />
          </div>
          <h1 class="brand-title">工控物联网边缘网关</h1>
          <p class="brand-subtitle">Industrial IoT Edge Gateway Management System</p>
          
          <div class="features-list">
            <div class="feature-item">
              <el-icon><Connection /></el-icon>
              <span>多协议设备接入</span>
            </div>
            <div class="feature-item">
              <el-icon><Monitor /></el-icon>
              <span>实时数据监控</span>
            </div>
            <div class="feature-item">
              <el-icon><Setting /></el-icon>
              <span>智能配置管理</span>
            </div>
            <div class="feature-item">
              <el-icon><DataAnalysis /></el-icon>
              <span>数据分析与预测</span>
            </div>
          </div>
        </div>
      </div>
      
      <div class="login-right">
        <div class="login-form-container">
          <div class="login-header">
            <h2>用户登录</h2>
            <p>请输入您的账号和密码</p>
          </div>
          
          <el-form
            ref="loginFormRef"
            :model="loginForm"
            :rules="loginRules"
            size="large"
            @keyup.enter="handleLogin"
          >
            <el-form-item prop="username">
              <el-input
                v-model="loginForm.username"
                placeholder="请输入用户名"
                prefix-icon="User"
                :disabled="loading"
              />
            </el-form-item>
            
            <el-form-item prop="password">
              <el-input
                v-model="loginForm.password"
                type="password"
                placeholder="请输入密码"
                prefix-icon="Lock"
                :disabled="loading"
                show-password
              />
            </el-form-item>
            
            <el-form-item>
              <div class="login-options">
                <el-checkbox v-model="rememberMe">记住我</el-checkbox>
                <el-link type="primary" :underline="false">
                  忘记密码？
                </el-link>
              </div>
            </el-form-item>
            
            <el-form-item>
              <el-button
                type="primary"
                size="large"
                style="width: 100%"
                :loading="loading"
                @click="handleLogin"
              >
                {{ loading ? '登录中...' : '登录' }}
              </el-button>
            </el-form-item>
          </el-form>
          
          <div class="demo-accounts" v-if="isDevelopment">
            <el-divider>演示账号</el-divider>
            <div class="demo-account-list">
              <el-button
                text
                size="small"
                @click="fillDemoAccount('admin', 'admin123')"
              >
                管理员 (admin/admin123)
              </el-button>
              <el-button
                text
                size="small"
                @click="fillDemoAccount('operator', 'operator123')"
              >
                操作员 (operator/operator123)
              </el-button>
              <el-button
                text
                size="small"
                @click="fillDemoAccount('viewer', 'viewer123')"
              >
                查看者 (viewer/viewer123)
              </el-button>
            </div>
          </div>
        </div>
        
        <div class="login-footer">
          <p>&copy; 2024 工控物联网边缘网关系统. All rights reserved.</p>
        </div>
      </div>
    </div>
    
    <!-- Background Animation -->
    <div class="background-animation">
      <div class="particle" v-for="n in 50" :key="n" :style="getParticleStyle(n)"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import type { FormInstance, FormRules } from 'element-plus'

const router = useRouter()
const authStore = useAuthStore()

// Form ref
const loginFormRef = ref<FormInstance>()

// Form data
const loginForm = reactive({
  username: '',
  password: '',
})

// Form validation rules
const loginRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 3, max: 20, message: '用户名长度应为3-20个字符', trigger: 'blur' },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码长度不能少于6个字符', trigger: 'blur' },
  ],
}

// State
const loading = ref(false)
const rememberMe = ref(false)

// Computed
const isDevelopment = computed(() => {
  return import.meta.env.MODE === 'development'
})

// Methods
const handleLogin = async () => {
  if (!loginFormRef.value) return
  
  try {
    await loginFormRef.value.validate()
    
    loading.value = true
    
    await authStore.login({
      username: loginForm.username,
      password: loginForm.password,
    })
    
    // Save remember me preference
    if (rememberMe.value) {
      localStorage.setItem('remember-username', loginForm.username)
    } else {
      localStorage.removeItem('remember-username')
    }
    
    // Redirect to dashboard
    router.push('/')
    
  } catch (error) {
    console.error('Login failed:', error)
  } finally {
    loading.value = false
  }
}

const fillDemoAccount = (username: string, password: string) => {
  loginForm.username = username
  loginForm.password = password
}

const getParticleStyle = (index: number) => {
  const delay = Math.random() * 20
  const duration = 15 + Math.random() * 10
  const left = Math.random() * 100
  const size = 2 + Math.random() * 4
  
  return {
    left: `${left}%`,
    animationDelay: `${delay}s`,
    animationDuration: `${duration}s`,
    width: `${size}px`,
    height: `${size}px`,
  }
}

// Initialize form with remembered username
onMounted(() => {
  const rememberedUsername = localStorage.getItem('remember-username')
  if (rememberedUsername) {
    loginForm.username = rememberedUsername
    rememberMe.value = true
  }
})
</script>

<style scoped lang="scss">
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  position: relative;
  overflow: hidden;
}

.login-wrapper {
  width: 100%;
  max-width: 1000px;
  background: white;
  border-radius: 16px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  display: flex;
  min-height: 600px;
  position: relative;
  z-index: 1;
}

.login-left {
  flex: 1;
  background: linear-gradient(135deg, #409eff 0%, #67c23a 100%);
  border-radius: 16px 0 0 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: white;
  
  .brand-section {
    text-align: center;
    
    .logo {
      margin-bottom: 24px;
      
      img {
        width: 80px;
        height: 80px;
        filter: brightness(0) invert(1);
      }
    }
    
    .brand-title {
      font-size: 28px;
      font-weight: 600;
      margin-bottom: 8px;
      line-height: 1.2;
    }
    
    .brand-subtitle {
      font-size: 14px;
      opacity: 0.9;
      margin-bottom: 40px;
    }
    
    .features-list {
      .feature-item {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 16px;
        font-size: 16px;
        
        .el-icon {
          font-size: 20px;
        }
      }
    }
  }
}

.login-right {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 40px;
}

.login-form-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  max-width: 360px;
  margin: 0 auto;
  width: 100%;
}

.login-header {
  text-align: center;
  margin-bottom: 32px;
  
  h2 {
    font-size: 24px;
    color: #303133;
    margin-bottom: 8px;
  }
  
  p {
    color: #909399;
    font-size: 14px;
  }
}

.login-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.demo-accounts {
  margin-top: 20px;
  
  .demo-account-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    
    .el-button {
      justify-content: flex-start;
      padding: 4px 0;
    }
  }
}

.login-footer {
  text-align: center;
  margin-top: 20px;
  
  p {
    color: #909399;
    font-size: 12px;
  }
}

.background-animation {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  overflow: hidden;
  
  .particle {
    position: absolute;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 50%;
    animation: float infinite linear;
  }
}

@keyframes float {
  0% {
    transform: translateY(100vh) rotate(0deg);
    opacity: 0;
  }
  10% {
    opacity: 1;
  }
  90% {
    opacity: 1;
  }
  100% {
    transform: translateY(-100px) rotate(360deg);
    opacity: 0;
  }
}

// Responsive design
@media (max-width: 768px) {
  .login-wrapper {
    max-width: 400px;
    flex-direction: column;
    min-height: auto;
  }
  
  .login-left {
    border-radius: 16px 16px 0 0;
    padding: 30px 20px;
    
    .brand-section {
      .brand-title {
        font-size: 20px;
      }
      
      .features-list {
        display: none;
      }
    }
  }
  
  .login-right {
    padding: 30px 20px;
  }
}

@media (max-width: 480px) {
  .login-container {
    padding: 20px;
  }
  
  .login-wrapper {
    width: 100%;
    max-width: none;
  }
  
  .login-left {
    padding: 20px;
    
    .brand-section {
      .logo img {
        width: 60px;
        height: 60px;
      }
      
      .brand-title {
        font-size: 18px;
      }
    }
  }
  
  .login-right {
    padding: 20px;
  }
}
</style>