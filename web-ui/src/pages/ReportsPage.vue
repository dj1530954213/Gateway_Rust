<template>
  <div class="reports-page">
    <div class="page-header">
      <div class="header-content">
        <h1>报表管理</h1>
        <p>生成、管理和分享各类数据报表</p>
      </div>
      <div class="header-stats">
        <div class="stat-item">
          <div class="stat-value">{{ totalReports }}</div>
          <div class="stat-label">总报表数</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ scheduledTasks }}</div>
          <div class="stat-label">定时任务</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ monthlyReports }}</div>
          <div class="stat-label">本月生成</div>
        </div>
      </div>
    </div>

    <div class="page-content">
      <!-- 快速操作卡片 -->
      <div class="quick-actions">
        <div class="action-grid">
          <div class="action-card" @click="showReportGenerator = true">
            <div class="action-icon">
              <el-icon><DocumentAdd /></el-icon>
            </div>
            <div class="action-info">
              <div class="action-title">生成新报表</div>
              <div class="action-desc">使用模板快速生成报表</div>
            </div>
          </div>

          <div class="action-card" @click="showTemplateManager = true">
            <div class="action-icon">
              <el-icon><Grid /></el-icon>
            </div>
            <div class="action-info">
              <div class="action-title">模板管理</div>
              <div class="action-desc">创建和管理报表模板</div>
            </div>
          </div>

          <div class="action-card" @click="showScheduleManager = true">
            <div class="action-icon">
              <el-icon><Timer /></el-icon>
            </div>
            <div class="action-info">
              <div class="action-title">定时任务</div>
              <div class="action-desc">设置和管理定时生成</div>
            </div>
          </div>

          <div class="action-card" @click="showAnalytics = true">
            <div class="action-icon">
              <el-icon><TrendCharts /></el-icon>
            </div>
            <div class="action-info">
              <div class="action-title">使用统计</div>
              <div class="action-desc">查看报表使用情况</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 报表列表 -->
      <div class="main-content">
        <ReportListCard ref="reportListRef" />
      </div>
    </div>

    <!-- 对话框组件 -->
    <ReportGeneratorDialog
      v-model="showReportGenerator"
      @report-generated="handleReportGenerated"
    />

    <TemplateManagerDialog v-model="showTemplateManager" />

    <ScheduleManagerDialog v-model="showScheduleManager" />

    <AnalyticsDialog v-model="showAnalytics" />
  </div>
</template>

<script setup lang="ts">
/**
 * ReportsPage —— 报表管理页面
 *
 * 📝 Responsibilities:
 *  1. 报表生成和管理入口
 *  2. 快速操作面板
 *  3. 统计信息展示
 *  4. 各功能模块整合
 *
 * 📦 Dependencies:
 *  - ReportListCard 报表列表组件
 *  - ReportGeneratorDialog 报表生成器
 *  - 其他管理对话框组件
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { DocumentAdd, Grid, Timer, TrendCharts } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { ref, computed, onMounted } from 'vue'

import AnalyticsDialog from '../components/reports/AnalyticsDialog.vue'
import ReportGeneratorDialog from '../components/reports/ReportGeneratorDialog.vue'
import ReportListCard from '../components/reports/ReportListCard.vue'
import ScheduleManagerDialog from '../components/reports/ScheduleManagerDialog.vue'
import TemplateManagerDialog from '../components/reports/TemplateManagerDialog.vue'

// ===== 响应式数据 =====
const showReportGenerator = ref(false)
const showTemplateManager = ref(false)
const showScheduleManager = ref(false)
const showAnalytics = ref(false)

const reportListRef = ref()

// 统计数据
const stats = ref({
  total: 156,
  scheduled: 12,
  monthly: 23,
})

// ===== 计算属性 =====
const totalReports = computed(() => stats.value.total)
const scheduledTasks = computed(() => stats.value.scheduled)
const monthlyReports = computed(() => stats.value.monthly)

// ===== 方法 =====

/**
 * 处理报表生成完成
 */
function handleReportGenerated(report: any) {
  ElMessage.success('报表生成任务已创建')

  // 刷新报表列表
  if (reportListRef.value) {
    reportListRef.value.refreshReports()
  }

  // 更新统计数据
  stats.value.total++
  stats.value.monthly++
}

/**
 * 加载统计数据
 */
async function loadStats() {
  try {
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 500))

    // 更新统计数据
    stats.value = {
      total: 156 + Math.floor(Math.random() * 20),
      scheduled: 12 + Math.floor(Math.random() * 5),
      monthly: 23 + Math.floor(Math.random() * 10),
    }
  } catch (error) {
    console.error('加载统计数据失败:', error)
  }
}

// ===== 生命周期 =====
onMounted(() => {
  loadStats()
})
</script>

<style scoped lang="scss">
.reports-page {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;

    .header-content {
      h1 {
        margin: 0 0 8px 0;
        font-size: 24px;
        color: #303133;
      }

      p {
        margin: 0;
        color: #606266;
        font-size: 14px;
      }
    }

    .header-stats {
      display: flex;
      gap: 24px;

      .stat-item {
        text-align: center;

        .stat-value {
          font-size: 28px;
          font-weight: 700;
          color: #409eff;
          margin-bottom: 4px;
        }

        .stat-label {
          font-size: 12px;
          color: #909399;
        }
      }
    }
  }

  .page-content {
    .quick-actions {
      margin-bottom: 24px;

      .action-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 16px;

        .action-card {
          display: flex;
          align-items: center;
          gap: 16px;
          padding: 20px;
          background: white;
          border: 1px solid #ebeef5;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.3s;

          &:hover {
            border-color: #c6e2ff;
            background: #f0f9ff;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(64, 158, 255, 0.15);
          }

          .action-icon {
            font-size: 32px;
            color: #409eff;
            flex-shrink: 0;
          }

          .action-info {
            flex: 1;

            .action-title {
              font-size: 16px;
              font-weight: 600;
              color: #303133;
              margin-bottom: 4px;
            }

            .action-desc {
              font-size: 12px;
              color: #606266;
              line-height: 1.4;
            }
          }
        }
      }
    }

    .main-content {
      background: white;
      border-radius: 8px;
      border: 1px solid #ebeef5;
      padding: 24px;
    }
  }
}

// 响应式设计
@media (max-width: 1200px) {
  .reports-page {
    .quick-actions .action-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

@media (max-width: 768px) {
  .reports-page {
    .page-header {
      flex-direction: column;
      gap: 16px;
      align-items: stretch;

      .header-stats {
        justify-content: space-around;
      }
    }

    .quick-actions .action-grid {
      grid-template-columns: 1fr;
      gap: 12px;

      .action-card {
        padding: 16px;

        .action-icon {
          font-size: 24px;
        }

        .action-info .action-title {
          font-size: 14px;
        }
      }
    }

    .main-content {
      padding: 16px;
    }
  }
}
</style>
