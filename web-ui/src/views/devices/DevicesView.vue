<template>
  <div class="devices-view">
    <div class="page-header">
      <h1>设备管理</h1>
      <p>管理IoT设备的连接、配置和监控</p>
    </div>
    
    <div class="devices-content">
      <!-- 临时内容，展示Mock数据 -->
      <el-card class="devices-card">
        <template #header>
          <div class="card-header">
            <span>设备列表</span>
            <el-button type="primary" size="small">
              <el-icon><Plus /></el-icon>
              添加设备
            </el-button>
          </div>
        </template>
        
        <el-table :data="mockDevices" style="width: 100%">
          <el-table-column prop="name" label="设备名称" width="180" />
          <el-table-column prop="protocol" label="协议类型" width="120" />
          <el-table-column prop="status" label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="row.status === 'online' ? 'success' : 'danger'">
                {{ row.status === 'online' ? '在线' : '离线' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="lastSeen" label="最后连接" />
          <el-table-column label="操作" fixed="right" width="200">
            <template #default="{ row }">
              <el-button size="small" type="primary" @click="editDevice(row)">
                编辑
              </el-button>
              <el-button size="small" type="danger" @click="deleteDevice(row)">
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
const mockDevices = ref([
  {
    id: 1,
    name: 'PLC-001',
    protocol: 'Modbus TCP',
    status: 'online',
    lastSeen: '2025-01-29 14:30:25'
  },
  {
    id: 2,
    name: 'Sensor-002',
    protocol: 'MQTT',
    status: 'online',
    lastSeen: '2025-01-29 14:28:15'
  },
  {
    id: 3,
    name: 'Gateway-003',
    protocol: 'OPC UA',
    status: 'offline',
    lastSeen: '2025-01-29 12:15:33'
  }
])

const editDevice = (device: any) => {
  ElMessage.info(`编辑设备: ${device.name}`)
}

const deleteDevice = (device: any) => {
  ElMessage.warning(`删除设备: ${device.name}`)
}
</script>

<style scoped lang="scss">
.devices-view {
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

.devices-content {
  .devices-card {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
  }
}
</style>