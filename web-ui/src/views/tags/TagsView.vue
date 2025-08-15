<template>
  <div class="tags-view">
    <div class="page-header">
      <h1>点位管理</h1>
      <p>管理设备数据点位的配置和采集</p>
    </div>

    <div class="tags-content">
      <!-- 真实数据点位管理 -->
      <el-card class="tags-card">
        <template #header>
          <div class="card-header">
            <span>数据点位</span>
            <el-button type="primary" size="small" @click="handleAddTag">
              <el-icon><Plus /></el-icon>
              添加点位
            </el-button>
          </div>
        </template>

        <el-table v-loading="loading" :data="tags" style="width: 100%">
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
import { Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ref, onMounted } from 'vue'

import { tagsApi } from '@/api'

// 状态管理
const tags = ref<any[]>([])
const loading = ref(false)

// 加载标签数据
const loadTags = async () => {
  try {
    loading.value = true
    const response = await tagsApi.list()
    if (response.success && response.data) {
      tags.value = response.data
    }
  } catch (error) {
    console.error('Failed to load tags:', error)
    ElMessage.error('加载点位数据失败')
  } finally {
    loading.value = false
  }
}

const handleAddTag = () => {
  ElMessage.info('添加点位功能开发中')
}

const editTag = async (tag: any) => {
  ElMessage.info('编辑点位功能开发中')
}

const deleteTag = async (tag: any) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除点位 "${tag.name}" 吗？`,
      '确认删除',
      {
        type: 'warning',
        confirmButtonText: '删除',
        cancelButtonText: '取消',
      }
    )

    await tagsApi.delete(tag.id)
    ElMessage.success('删除成功')
    await loadTags()
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('Failed to delete tag:', error)
      ElMessage.error('删除失败')
    }
  }
}

// 生命周期
onMounted(async () => {
  await loadTags()
})
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
