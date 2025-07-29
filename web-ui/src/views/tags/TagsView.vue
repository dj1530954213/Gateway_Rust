<template>
  <div class="tags-view">
    <div class="page-header">
      <h1>点位管理</h1>
      <p>管理设备数据点位的配置和采集</p>
    </div>
    
    <div class="tags-content">
      <!-- 临时内容，展示Mock数据 -->
      <el-card class="tags-card">
        <template #header>
          <div class="card-header">
            <span>数据点位</span>
            <el-button type="primary" size="small">
              <el-icon><Plus /></el-icon>
              添加点位
            </el-button>
          </div>
        </template>
        
        <el-table :data="mockTags" style="width: 100%">
          <el-table-column prop="name" label="点位名称" width="180" />
          <el-table-column prop="address" label="地址" width="120" />
          <el-table-column prop="dataType" label="数据类型" width="100" />
          <el-table-column prop="unit" label="单位" width="80" />
          <el-table-column prop="value" label="当前值" width="100" />
          <el-table-column prop="timestamp" label="更新时间" />
          <el-table-column label="操作" fixed="right" width="200">
            <template #default="{ row }">
              <el-button size="small" type="primary" @click="editTag(row)">
                编辑
              </el-button>
              <el-button size="small" type="danger" @click="deleteTag(row)">
                删除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'

// Mock数据
const mockTags = ref([
  {
    id: 1,
    name: 'Temperature_01',
    address: '40001',
    dataType: 'Float',
    unit: '°C',
    value: '23.5',
    timestamp: '2025-01-29 14:30:25'
  },
  {
    id: 2,
    name: 'Pressure_01', 
    address: '40002',
    dataType: 'Float',
    unit: 'Pa',
    value: '101325',
    timestamp: '2025-01-29 14:30:20'
  },
  {
    id: 3,
    name: 'Status_Motor',
    address: '10001',
    dataType: 'Boolean',
    unit: '',
    value: 'true',
    timestamp: '2025-01-29 14:30:15'
  }
])

const editTag = (tag: any) => {
  ElMessage.info(`编辑点位: ${tag.name}`)
}

const deleteTag = (tag: any) => {
  ElMessage.warning(`删除点位: ${tag.name}`)
}
</script>

<style scoped lang="scss">
.tags-view {
  padding: 20px;
}

.page-header {
  margin-bottom: 20px;
  
  h1 {
    margin: 0 0 8px 0;
    font-size: 24px;
    color: var(--el-text-color-primary);
  }
  
  p {
    margin: 0;
    color: var(--el-text-color-secondary);
  }
}

.tags-content {
  .tags-card {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
  }
}
</style>