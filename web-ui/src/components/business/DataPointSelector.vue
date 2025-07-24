<template>
  <div class="data-point-selector" :class="containerClass">
    <!-- 选择器头部 -->
    <div class="selector-header">
      <div class="header-title">
        <el-icon :size="16">
          <DataBoard />
        </el-icon>
        <span>{{ title }}</span>
        <el-tag v-if="showSelectedCount" size="small" type="info">
          已选择 {{ selectedCount }} 项
        </el-tag>
      </div>
      
      <div class="header-actions">
        <!-- 刷新按钮 -->
        <el-button
          v-if="showRefresh"
          type="text"
          size="small"
          :icon="Refresh"
          :loading="loading"
          @click="handleRefresh"
        />
        
        <!-- 展开/折叠所有 -->
        <el-button
          v-if="showExpandAll"
          type="text"
          size="small"
          @click="toggleExpandAll"
        >
          {{ allExpanded ? '折叠' : '展开' }}全部
        </el-button>
        
        <!-- 清空选择 -->
        <el-button
          v-if="showClear && selectedCount > 0"
          type="text"
          size="small"
          @click="clearSelection"
        >
          清空选择
        </el-button>
      </div>
    </div>
    
    <!-- 搜索和筛选 -->
    <div class="selector-filters">
      <SearchBox
        v-model="searchKeyword"
        placeholder="搜索数据点名称、地址或描述"
        :suggestions="searchSuggestions"
        size="small"
        @search="handleSearch"
        @clear="handleSearchClear"
      />
      
      <div class="filter-buttons">
        <el-dropdown v-if="showTypeFilter" trigger="click" @command="handleTypeFilter">
          <el-button type="text" size="small">
            数据类型 <el-icon><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="">所有类型</el-dropdown-item>
              <el-dropdown-item
                v-for="type in dataTypes"
                :key="type.value"
                :command="type.value"
              >
                {{ type.label }}
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        
        <el-dropdown v-if="showQualityFilter" trigger="click" @command="handleQualityFilter">
          <el-button type="text" size="small">
            数据质量 <el-icon><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="">所有质量</el-dropdown-item>
              <el-dropdown-item command="good">良好</el-dropdown-item>
              <el-dropdown-item command="bad">异常</el-dropdown-item>
              <el-dropdown-item command="uncertain">不确定</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
    
    <!-- 数据点树形列表 -->
    <div class="selector-content">
      <el-scrollbar height="400px">
        <el-tree
          ref="treeRef"
          :data="filteredDataPoints"
          :props="treeProps"
          :show-checkbox="multiple"
          :check-strictly="checkStrictly"
          :expand-on-click-node="false"
          :highlight-current="!multiple"
          :filter-node-method="filterNode"
          :default-expanded-keys="defaultExpandedKeys"
          node-key="id"
          @check="handleCheck"
          @current-change="handleCurrentChange"
          @node-expand="handleNodeExpand"
          @node-collapse="handleNodeCollapse"
        >
          <template #default="{ node, data }">
            <div class="tree-node-content" :class="getNodeClass(data)">
              <!-- 节点图标 -->
              <el-icon class="node-icon">
                <component :is="getNodeIcon(data)" />
              </el-icon>
              
              <!-- 节点信息 -->
              <div class="node-info">
                <div class="node-title">
                  <span class="node-name">{{ data.name }}</span>
                  <StatusTag
                    v-if="data.type === 'datapoint'"
                    :status="data.quality || 'unknown'"
                    size="small"
                    :mappings="qualityMappings"
                  />
                </div>
                
                <div v-if="showNodeDetails && data.type === 'datapoint'" class="node-details">
                  <span class="detail-item">
                    <el-tag size="small" type="info">{{ data.dataType }}</el-tag>
                  </span>
                  <span class="detail-item">地址: {{ data.address }}</span>
                  <span v-if="data.unit" class="detail-item">单位: {{ data.unit }}</span>
                </div>
              </div>
              
              <!-- 节点操作 -->
              <div class="node-actions">
                <!-- 数据预览 -->
                <el-button
                  v-if="data.type === 'datapoint' && showPreview"
                  type="text"
                  size="small"
                  :icon="View"
                  @click.stop="handlePreview(data)"
                />
                
                <!-- 更多操作 -->
                <el-dropdown
                  v-if="showNodeActions"
                  trigger="click"
                  @command="(command) => handleNodeAction(command, data)"
                >
                  <el-button type="text" size="small" :icon="MoreFilled" @click.stop />
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item command="details" :icon="InfoFilled">
                        查看详情
                      </el-dropdown-item>
                      <el-dropdown-item v-if="data.type === 'datapoint'" command="history" :icon="TrendCharts">
                        历史趋势
                      </el-dropdown-item>
                      <el-dropdown-item command="config" :icon="Setting">
                        配置
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>
            </div>
          </template>
        </el-tree>
      </el-scrollbar>
    </div>
    
    <!-- 批量操作栏 -->
    <div v-if="multiple && selectedCount > 0" class="selector-batch-actions">
      <div class="batch-info">
        已选择 {{ selectedCount }} 个数据点
      </div>
      
      <div class="batch-buttons">
        <el-button size="small" @click="handleBatchExport">
          导出
        </el-button>
        <el-button size="small" @click="handleBatchConfig">
          批量配置
        </el-button>
        <el-button size="small" type="danger" @click="handleBatchRemove">
          移除
        </el-button>
      </div>
    </div>
    
    <!-- 数据预览对话框 -->
    <el-dialog
      v-model="previewVisible"
      :title="`数据预览 - ${previewDataPoint?.name}`"
      width="500px"
      :before-close="handlePreviewClose"
    >
      <div v-if="previewDataPoint" class="data-preview">
        <el-descriptions border column="1">
          <el-descriptions-item label="数据点名称">
            {{ previewDataPoint.name }}
          </el-descriptions-item>
          <el-descriptions-item label="数据类型">
            {{ previewDataPoint.dataType }}
          </el-descriptions-item>
          <el-descriptions-item label="当前值">
            <span :class="getValueClass(previewDataPoint.quality)">
              {{ formatValue(previewDataPoint.value, previewDataPoint.dataType) }}
              {{ previewDataPoint.unit || '' }}
            </span>
          </el-descriptions-item>
          <el-descriptions-item label="数据质量">
            <StatusTag :status="previewDataPoint.quality" :mappings="qualityMappings" />
          </el-descriptions-item>
          <el-descriptions-item label="最后更新">
            {{ formatTime(previewDataPoint.lastUpdate) }}
          </el-descriptions-item>
          <el-descriptions-item label="地址">
            {{ previewDataPoint.address }}
          </el-descriptions-item>
          <el-descriptions-item v-if="previewDataPoint.description" label="描述">
            {{ previewDataPoint.description }}
          </el-descriptions-item>
        </el-descriptions>
        
        <!-- 实时数据更新 -->
        <div v-if="showRealtimeValue" class="realtime-value">
          <el-divider content-position="left">实时数据</el-divider>
          <div class="value-display">
            <span class="value-number">
              {{ formatValue(previewDataPoint.value, previewDataPoint.dataType) }}
            </span>
            <span class="value-unit">{{ previewDataPoint.unit || '' }}</span>
          </div>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="previewVisible = false">关闭</el-button>
        <el-button v-if="!isSelected(previewDataPoint)" type="primary" @click="handleSelectFromPreview">
          选择此数据点
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { 
  DataBoard, 
  Refresh, 
  ArrowDown, 
  View, 
  MoreFilled,
  InfoFilled,
  TrendCharts,
  Setting,
  Folder,
  Document,
  Monitor
} from '@element-plus/icons-vue'
import { SearchBox, StatusTag } from '../base'

export interface DataPoint {
  id: string
  name: string
  type: 'device' | 'group' | 'datapoint'
  dataType?: 'boolean' | 'number' | 'string' | 'object'
  address?: string
  value?: any
  unit?: string
  quality?: 'good' | 'bad' | 'uncertain'
  lastUpdate?: Date
  description?: string
  children?: DataPoint[]
  deviceId?: string
  groupId?: string
}

interface Props {
  dataPoints: DataPoint[]
  modelValue?: string[] | string
  multiple?: boolean
  title?: string
  
  // 功能控制
  showRefresh?: boolean
  showExpandAll?: boolean
  showClear?: boolean
  showSelectedCount?: boolean
  showTypeFilter?: boolean
  showQualityFilter?: boolean
  showNodeDetails?: boolean
  showPreview?: boolean
  showNodeActions?: boolean
  showRealtimeValue?: boolean
  
  // 树形配置
  checkStrictly?: boolean
  defaultExpandLevel?: number
  
  // 外观配置
  height?: string
  size?: 'small' | 'default' | 'large'
  
  // 自定义样式
  customClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: () => [],
  multiple: true,
  title: '选择数据点',
  showRefresh: true,
  showExpandAll: true,
  showClear: true,
  showSelectedCount: true,
  showTypeFilter: true,
  showQualityFilter: true,
  showNodeDetails: true,
  showPreview: true,
  showNodeActions: true,
  showRealtimeValue: false,
  checkStrictly: false,
  defaultExpandLevel: 1,
  height: '400px',
  size: 'default',
})

interface Emits {
  'update:modelValue': [value: string[] | string]
  change: [selected: DataPoint[]]
  refresh: []
  preview: [dataPoint: DataPoint]
  nodeAction: [action: string, dataPoint: DataPoint]
}

const emit = defineEmits<Emits>()

// 状态
const loading = ref(false)
const searchKeyword = ref('')
const allExpanded = ref(false)
const previewVisible = ref(false)
const previewDataPoint = ref<DataPoint | null>(null)
const treeRef = ref()

// 过滤条件
const typeFilter = ref('')
const qualityFilter = ref('')

// 树形配置
const treeProps = {
  children: 'children',
  label: 'name',
}

// 数据类型选项
const dataTypes = [
  { label: '布尔型', value: 'boolean' },
  { label: '数值型', value: 'number' },
  { label: '字符型', value: 'string' },
  { label: '对象型', value: 'object' },
]

// 质量映射
const qualityMappings = {
  good: { type: 'success', text: '良好' },
  bad: { type: 'danger', text: '异常' },
  uncertain: { type: 'warning', text: '不确定' },
  unknown: { type: 'info', text: '未知' }
}

// 计算属性
const containerClass = computed(() => {
  const classes = []
  
  classes.push(`data-point-selector--${props.size}`)
  
  if (props.customClass) {
    classes.push(props.customClass)
  }
  
  return classes.join(' ')
})

const selectedItems = computed({
  get: () => {
    if (props.multiple) {
      return Array.isArray(props.modelValue) ? props.modelValue : []
    } else {
      return typeof props.modelValue === 'string' ? props.modelValue : ''
    }
  },
  set: (value) => {
    emit('update:modelValue', value)
  }
})

const selectedCount = computed(() => {
  return props.multiple ? (selectedItems.value as string[]).length : (selectedItems.value ? 1 : 0)
})

const defaultExpandedKeys = computed(() => {
  const keys: string[] = []
  
  const collectKeys = (nodes: DataPoint[], level: number) => {
    if (level >= props.defaultExpandLevel) return
    
    nodes.forEach(node => {
      if (node.children && node.children.length > 0) {
        keys.push(node.id)
        collectKeys(node.children, level + 1)
      }
    })
  }
  
  collectKeys(props.dataPoints, 0)
  return keys
})

const filteredDataPoints = computed(() => {
  let result = [...props.dataPoints]
  
  // 应用过滤
  if (typeFilter.value || qualityFilter.value || searchKeyword.value) {
    result = filterDataPoints(result)
  }
  
  return result
})

const searchSuggestions = computed(() => {
  const suggestions: any[] = []
  
  const collectSuggestions = (nodes: DataPoint[]) => {
    nodes.forEach(node => {
      if (node.type === 'datapoint') {
        suggestions.push({
          text: node.name,
          value: node.id,
          description: node.address,
          category: node.dataType
        })
      }
      
      if (node.children) {
        collectSuggestions(node.children)
      }
    })
  }
  
  collectSuggestions(props.dataPoints)
  return suggestions.slice(0, 10)
})

// 方法
const filterDataPoints = (nodes: DataPoint[]): DataPoint[] => {
  return nodes.filter(node => {
    // 递归处理子节点
    if (node.children) {
      node.children = filterDataPoints(node.children)
    }
    
    // 如果是数据点，应用过滤条件
    if (node.type === 'datapoint') {
      let match = true
      
      // 类型过滤
      if (typeFilter.value && node.dataType !== typeFilter.value) {
        match = false
      }
      
      // 质量过滤
      if (qualityFilter.value && node.quality !== qualityFilter.value) {
        match = false
      }
      
      // 搜索关键词
      if (searchKeyword.value) {
        const keyword = searchKeyword.value.toLowerCase()
        const searchText = `${node.name} ${node.address || ''} ${node.description || ''}`.toLowerCase()
        if (!searchText.includes(keyword)) {
          match = false
        }
      }
      
      return match
    }
    
    // 如果是分组或设备，只要有子节点就保留
    return node.children && node.children.length > 0
  })
}

const filterNode = (value: string, data: DataPoint) => {
  if (!value) return true
  return data.name.toLowerCase().includes(value.toLowerCase())
}

const getNodeClass = (data: DataPoint) => {
  const classes = [`node-${data.type}`]
  
  if (data.type === 'datapoint') {
    classes.push(`quality-${data.quality || 'unknown'}`)
  }
  
  return classes.join(' ')
}

const getNodeIcon = (data: DataPoint) => {
  switch (data.type) {
    case 'device':
      return Monitor
    case 'group':
      return Folder
    case 'datapoint':
      return Document
    default:
      return Document
  }
}

const formatValue = (value: any, dataType?: string) => {
  if (value === null || value === undefined) return '无数据'
  
  switch (dataType) {
    case 'boolean':
      return value ? '是' : '否'
    case 'number':
      return typeof value === 'number' ? value.toFixed(2) : value
    default:
      return String(value)
  }
}

const formatTime = (date?: Date) => {
  if (!date) return '未知'
  return date.toLocaleString('zh-CN')
}

const getValueClass = (quality?: string) => {
  switch (quality) {
    case 'good':
      return 'value-good'
    case 'bad':
      return 'value-bad'
    case 'uncertain':
      return 'value-uncertain'
    default:
      return 'value-unknown'
  }
}

const isSelected = (dataPoint?: DataPoint | null) => {
  if (!dataPoint) return false
  
  if (props.multiple) {
    return (selectedItems.value as string[]).includes(dataPoint.id)
  } else {
    return selectedItems.value === dataPoint.id
  }
}

// 事件处理
const handleRefresh = () => {
  loading.value = true
  emit('refresh')
  setTimeout(() => {
    loading.value = false
  }, 1000)
}

const toggleExpandAll = () => {
  allExpanded.value = !allExpanded.value
  
  const expandAllNodes = (keys: string[]) => {
    keys.forEach(key => {
      if (allExpanded.value) {
        treeRef.value?.setExpandedKeys([...treeRef.value.getExpandedKeys(), key])
      } else {
        treeRef.value?.setExpandedKeys([])
      }
    })
  }
  
  // 收集所有节点key
  const allKeys: string[] = []
  const collectKeys = (nodes: DataPoint[]) => {
    nodes.forEach(node => {
      allKeys.push(node.id)
      if (node.children) {
        collectKeys(node.children)
      }
    })
  }
  
  collectKeys(props.dataPoints)
  expandAllNodes(allKeys)
}

const clearSelection = () => {
  if (props.multiple) {
    selectedItems.value = []
  } else {
    selectedItems.value = ''
  }
  
  if (treeRef.value) {
    treeRef.value.setCheckedKeys([])
    treeRef.value.setCurrentKey(null)
  }
}

const handleSearch = (keyword: string) => {
  searchKeyword.value = keyword
  treeRef.value?.filter(keyword)
}

const handleSearchClear = () => {
  searchKeyword.value = ''
  treeRef.value?.filter('')
}

const handleTypeFilter = (type: string) => {
  typeFilter.value = type
}

const handleQualityFilter = (quality: string) => {
  qualityFilter.value = quality
}

const handleCheck = (data: DataPoint, checked: { checkedKeys: string[] }) => {
  if (props.multiple) {
    selectedItems.value = checked.checkedKeys
    
    // 获取选中的数据点对象
    const selectedDataPoints = getSelectedDataPoints(checked.checkedKeys)
    emit('change', selectedDataPoints)
  }
}

const handleCurrentChange = (data: DataPoint) => {
  if (!props.multiple && data.type === 'datapoint') {
    selectedItems.value = data.id
    emit('change', [data])
  }
}

const handleNodeExpand = (data: DataPoint) => {
  // 节点展开处理
}

const handleNodeCollapse = (data: DataPoint) => {
  // 节点折叠处理
}

const handlePreview = (dataPoint: DataPoint) => {
  previewDataPoint.value = dataPoint
  previewVisible.value = true
  emit('preview', dataPoint)
}

const handlePreviewClose = () => {
  previewVisible.value = false
  previewDataPoint.value = null
}

const handleSelectFromPreview = () => {
  if (previewDataPoint.value) {
    if (props.multiple) {
      const currentSelected = selectedItems.value as string[]
      if (!currentSelected.includes(previewDataPoint.value.id)) {
        selectedItems.value = [...currentSelected, previewDataPoint.value.id]
      }
    } else {
      selectedItems.value = previewDataPoint.value.id
    }
    
    previewVisible.value = false
  }
}

const handleNodeAction = (action: string, dataPoint: DataPoint) => {
  emit('nodeAction', action, dataPoint)
}

const handleBatchExport = () => {
  // 批量导出处理
}

const handleBatchConfig = () => {
  // 批量配置处理
}

const handleBatchRemove = () => {
  // 批量移除处理
}

const getSelectedDataPoints = (keys: string[]): DataPoint[] => {
  const result: DataPoint[] = []
  
  const findDataPoints = (nodes: DataPoint[]) => {
    nodes.forEach(node => {
      if (keys.includes(node.id) && node.type === 'datapoint') {
        result.push(node)
      }
      
      if (node.children) {
        findDataPoints(node.children)
      }
    })
  }
  
  findDataPoints(props.dataPoints)
  return result
}

// 监听
watch(() => props.modelValue, (newValue) => {
  if (props.multiple && treeRef.value && Array.isArray(newValue)) {
    treeRef.value.setCheckedKeys(newValue)
  } else if (!props.multiple && treeRef.value && typeof newValue === 'string') {
    treeRef.value.setCurrentKey(newValue)
  }
})

// 生命周期
onMounted(() => {
  // 初始化选中状态
  if (props.multiple && Array.isArray(props.modelValue)) {
    treeRef.value?.setCheckedKeys(props.modelValue)
  } else if (!props.multiple && typeof props.modelValue === 'string') {
    treeRef.value?.setCurrentKey(props.modelValue)
  }
})
</script>

<style scoped lang="scss">
.data-point-selector {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 6px;
  
  &.data-point-selector--small {
    .selector-header {
      padding: 8px 12px;
    }
    
    .selector-filters {
      padding: 8px 12px;
    }
  }
  
  &.data-point-selector--large {
    .tree-node-content {
      padding: 8px;
    }
  }
}

.selector-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  
  .header-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--el-text-color-primary);
  }
  
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
}

.selector-filters {
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  
  .filter-buttons {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }
}

.selector-content {
  padding: 8px;
  
  .tree-node-content {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    width: 100%;
    border-radius: 4px;
    transition: background-color 0.3s;
    
    &:hover {
      background-color: var(--el-fill-color-light);
    }
    
    &.node-device {
      font-weight: 600;
    }
    
    &.node-group {
      font-weight: 500;
    }
    
    .node-icon {
      margin-right: 8px;
      color: var(--el-text-color-secondary);
    }
    
    .node-info {
      flex: 1;
      min-width: 0;
      
      .node-title {
        display: flex;
        align-items: center;
        gap: 8px;
        
        .node-name {
          font-size: 14px;
          color: var(--el-text-color-primary);
        }
      }
      
      .node-details {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-top: 4px;
        
        .detail-item {
          font-size: 12px;
          color: var(--el-text-color-secondary);
        }
      }
    }
    
    .node-actions {
      display: flex;
      align-items: center;
      gap: 4px;
      opacity: 0;
      transition: opacity 0.3s;
    }
    
    &:hover .node-actions {
      opacity: 1;
    }
  }
}

.selector-batch-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color-light);
  
  .batch-info {
    font-size: 13px;
    color: var(--el-text-color-secondary);
  }
  
  .batch-buttons {
    display: flex;
    gap: 8px;
  }
}

.data-preview {
  .realtime-value {
    margin-top: 16px;
    text-align: center;
    
    .value-display {
      display: flex;
      align-items: baseline;
      justify-content: center;
      gap: 4px;
      
      .value-number {
        font-size: 24px;
        font-weight: 600;
        
        &.value-good {
          color: var(--el-color-success);
        }
        
        &.value-bad {
          color: var(--el-color-danger);
        }
        
        &.value-uncertain {
          color: var(--el-color-warning);
        }
        
        &.value-unknown {
          color: var(--el-text-color-secondary);
        }
      }
      
      .value-unit {
        font-size: 14px;
        color: var(--el-text-color-secondary);
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .data-point-selector {
    .selector-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
    
    .selector-filters {
      .filter-buttons {
        flex-wrap: wrap;
      }
    }
    
    .tree-node-content {
      .node-details {
        flex-direction: column;
        align-items: flex-start;
        gap: 4px;
      }
    }
    
    .selector-batch-actions {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
  }
}
</style>