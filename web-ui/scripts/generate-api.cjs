#!/usr/bin/env node

/**
 * Gateway Rust - OpenAPI TypeScript 客户端生成脚本
 */

const { exec } = require('child_process')
const fs = require('fs')
const path = require('path')

const API_URL = process.env.VITE_API_BASE_URL || 'http://localhost:8080'
const OPENAPI_URL = `${API_URL}/docs/openapi.json`
const OUTPUT_DIR = './src/api/generated'

console.log('🚀 开始生成 OpenAPI TypeScript 客户端...')
console.log(`📡 API 地址: ${OPENAPI_URL}`)
console.log(`📁 输出目录: ${OUTPUT_DIR}`)

// 确保输出目录存在
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true })
  console.log(`✅ 创建输出目录: ${OUTPUT_DIR}`)
}

// 检查 API 是否可用
async function checkApiAvailability() {
  return new Promise((resolve) => {
    const { spawn } = require('child_process')
    const curl = spawn('curl', ['-f', '-s', OPENAPI_URL], {
      stdio: 'pipe'
    })
    
    curl.on('close', (code) => {
      resolve(code === 0)
    })
    
    curl.on('error', () => {
      resolve(false)
    })
  })
}

// 生成客户端
async function generateClient() {
  const isApiAvailable = await checkApiAvailability()
  
  if (!isApiAvailable) {
    console.warn('⚠️  无法连接到 Gateway Rust API 服务')
    console.warn('   请确保后端服务正在运行，然后重新执行此脚本')
    console.warn('   启动命令: cargo run -p web-gw-api')
    
    // 创建一个占位符文件，说明如何生成客户端
    const placeholderContent = `/**
 * OpenAPI TypeScript 客户端 - 占位符
 * 
 * 要生成实际的客户端代码，请：
 * 1. 启动 Gateway Rust 后端服务: cargo run -p web-gw-api
 * 2. 运行生成脚本: npm run gen:api
 * 
 * 生成的客户端将包含：
 * - 完整的 TypeScript 类型定义
 * - 自动生成的 API 服务类
 * - 与后端 OpenAPI 规范完全同步的接口
 */

export interface PlaceholderApi {
  // 这是一个占位符接口
  // 实际的 API 客户端将在后端服务启动后生成
}

export const GatewayRustApiClient = {
  // 占位符实现
  info: () => console.warn('请先生成 OpenAPI 客户端')
}

export default GatewayRustApiClient
`
    
    fs.writeFileSync(path.join(OUTPUT_DIR, 'index.ts'), placeholderContent)
    console.log('📝 已创建占位符文件')
    process.exit(1)
  }
  
  console.log('✅ API 服务可用，开始生成客户端...')
  
  // 使用 openapi-typescript-codegen 生成客户端
  const generateCommand = `npx openapi-typescript-codegen --input "${OPENAPI_URL}" --output "${OUTPUT_DIR}" --client axios --useOptions --useUnionTypes --exportCore false --exportSchemas --exportModels --exportServices --write --prettier`
  
  exec(generateCommand, (error, stdout, stderr) => {
    if (error) {
      console.error('❌ 生成失败:', error.message)
      return
    }
    
    if (stderr) {
      console.warn('⚠️  警告:', stderr)
    }
    
    console.log(stdout)
    console.log('🎉 OpenAPI TypeScript 客户端生成完成！')
    console.log('📦 生成的文件位于:', OUTPUT_DIR)
    
    // 创建导出文件
    const exportContent = `/**
 * Gateway Rust - OpenAPI TypeScript 客户端
 * 
 * 自动生成，请勿手动修改
 * 生成时间: ${new Date().toISOString()}
 * API 地址: ${OPENAPI_URL}
 */

export * from './services'
export * from './models'
export * from './core'

// 重新导出主要服务
export { DefaultService as GatewayRustApi } from './services/DefaultService'
`
    
    fs.writeFileSync(path.join(OUTPUT_DIR, 'index.ts'), exportContent)
    console.log('✅ 创建统一导出文件')
    
    // 更新主 API 文件以使用生成的客户端
    console.log('💡 可以在代码中这样使用生成的客户端:')
    console.log('   import { GatewayRustApi } from "@/api/generated"')
  })
}

// 执行生成
generateClient().catch(console.error)