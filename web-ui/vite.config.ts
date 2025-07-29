import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  
  return {
    plugins: [
      vue(),
      AutoImport({
        resolvers: [ElementPlusResolver()],
        imports: [
          'vue',
          'vue-router',
          'pinia',
          {
            'element-plus': ['ElMessage', 'ElMessageBox', 'ElNotification', 'ElLoading'],
            'axios': [['default', 'axios']],
            'dayjs': [['default', 'dayjs']],
            'lodash-es': ['debounce', 'throttle', 'cloneDeep', 'merge'],
          }
        ],
        dts: true,
        eslintrc: {
          enabled: true,
          filepath: './.eslintrc-auto-import.json',
          globalsPropValue: true,
        },
      }),
      Components({
        resolvers: [ElementPlusResolver({ 
          importStyle: 'sass',
        })],
        dts: true
      })
    ],
    
    define: {
      __VUE_OPTIONS_API__: false,
      __VUE_PROD_DEVTOOLS__: false,
    },
    
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
        '@components': resolve(__dirname, 'src/components'),
        '@views': resolve(__dirname, 'src/views'),
        '@stores': resolve(__dirname, 'src/stores'),
        '@services': resolve(__dirname, 'src/services'),
        '@utils': resolve(__dirname, 'src/utils'),
        '@types': resolve(__dirname, 'src/types'),
        '@assets': resolve(__dirname, 'src/assets'),
        '@api': resolve(__dirname, 'src/api'),
        '@composables': resolve(__dirname, 'src/composables'),
      }
    },
    
    server: {
      port: parseInt(env.VITE_DEV_PORT) || 5173,
      host: '0.0.0.0',
      open: false,
      cors: true,
      proxy: {
        // Gateway Rust API代理
        '/api': {
          target: env.VITE_API_BASE_URL || 'http://localhost:50010',
          changeOrigin: true,
          secure: false,
          rewrite: (path) => path.replace(/^\/api/, '/api/v1'),
          configure: (proxy, options) => {
            proxy.on('error', (err, req, res) => {
              console.error('API代理错误:', err.message);
            });
            proxy.on('proxyReq', (proxyReq, req, res) => {
              console.log('🚀 API请求:', req.method, req.url);
            });
            proxy.on('proxyRes', (proxyRes, req, res) => {
              console.log('✅ API响应:', proxyRes.statusCode, req.url);
            });
          }
        },
        // WebSocket代理
        '/ws': {
          target: env.VITE_WS_BASE_URL || 'ws://localhost:50013',
          ws: true,
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/ws/, '/ws')
        },
        // OpenAPI文档代理
        '/docs': {
          target: env.VITE_API_BASE_URL || 'http://localhost:50010',
          changeOrigin: true,
          secure: false,
        }
      }
    },
    
    build: {
      target: 'es2020',
      outDir: 'dist',
      assetsDir: 'assets',
      minify: 'terser',
      sourcemap: mode === 'development',
      chunkSizeWarningLimit: 1000,
      rollupOptions: {
        output: {
          manualChunks: {
            'element-plus': ['element-plus', '@element-plus/icons-vue'],
            'vue-vendor': ['vue', 'vue-router', 'pinia'],
            'chart-vendor': ['echarts', 'vue-echarts'],
            'utils-vendor': ['dayjs', 'lodash-es', 'axios'],
            'form-vendor': ['vee-validate', 'yup', 'zod'],
          }
        }
      }
    },
    
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@use "@/assets/styles/variables.scss" as *;`,
          api: 'modern-compiler', // Use modern Sass API
          silenceDeprecations: ['legacy-js-api'],
        }
      }
    },
    
    test: {
      globals: true,
      environment: 'jsdom',
      setupFiles: ['./src/test/setup.ts']
    }
  }
})