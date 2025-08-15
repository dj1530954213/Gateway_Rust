import { resolve } from 'path'

import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'
import Components from 'unplugin-vue-components/vite'
import { defineConfig, loadEnv } from 'vite'

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
            'element-plus': [
              'ElMessage',
              'ElMessageBox',
              'ElNotification',
              'ElLoading',
            ],
            axios: [['default', 'axios']],
            dayjs: [['default', 'dayjs']],
            'lodash-es': ['debounce', 'throttle', 'cloneDeep', 'merge'],
          },
        ],
        dts: true,
        eslintrc: {
          enabled: true,
          filepath: './.eslintrc-auto-import.json',
          globalsPropValue: true,
        },
      }),
      Components({
        resolvers: [
          ElementPlusResolver({
            importStyle: 'sass',
          }),
        ],
        dts: true,
      }),
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
      },
    },

    server: {
      port: parseInt(env.VITE_DEV_PORT) || 5173,
      host: '0.0.0.0',
      open: false,
      cors: true,
      proxy: {
        // Gateway Rust API代理
        '/api': {
          target: 'http://localhost:50013',
          changeOrigin: true,
          secure: false,
          configure: (proxy, _options) => {
            proxy.on('error', (err, _req, _res) => {
              console.error('API代理错误:', err.message)
            })
            proxy.on('proxyReq', (_proxyReq, req, _res) => {
              console.log('🚀 API请求:', req.method, req.url)
            })
            proxy.on('proxyRes', (proxyRes, req, _res) => {
              console.log('✅ API响应:', proxyRes.statusCode, req.url)
            })
          },
        },
        // WebSocket代理
        '/ws': {
          target: env.VITE_WS_BASE_URL || 'ws://localhost:8080',
          ws: true,
          changeOrigin: true,
          rewrite: path => path.replace(/^\/ws/, '/ws'),
        },
        // System API代理
        '/system': {
          target: 'http://localhost:50013',
          changeOrigin: true,
          secure: false,
        },
        // OpenAPI文档代理
        '/docs': {
          target: 'http://localhost:50013',
          changeOrigin: true,
          secure: false,
        },
      },
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
          manualChunks(id) {
            // 更精确的代码分割策略，避免循环依赖
            if (id.includes('node_modules')) {
              if (id.includes('vue') || id.includes('pinia') || id.includes('vue-router')) {
                return 'vue-vendor'
              }
              if (id.includes('element-plus')) {
                return 'element-plus'
              }
              if (id.includes('echarts')) {
                return 'chart-vendor'
              }
              if (id.includes('dayjs') || id.includes('lodash-es') || id.includes('axios')) {
                return 'utils-vendor'
              }
              return 'vendor'
            }
          },
        },
      },
    },

    optimizeDeps: {
      // 预构建依赖，避免运行时解析问题
      include: [
        'vue',
        'vue-router',
        'pinia',
        'element-plus',
        '@element-plus/icons-vue',
        'axios',
        'echarts',
        'dayjs'
      ],
    },

    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@use "@/assets/styles/variables.scss" as *;`,
          api: 'modern-compiler', // Use modern Sass API
          silenceDeprecations: ['legacy-js-api'],
        },
      },
    },

    test: {
      globals: true,
      environment: 'jsdom',
      setupFiles: ['./src/test/setup.ts'],
    },
  }
})
