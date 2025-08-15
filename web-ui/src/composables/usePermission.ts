/**
 * usePermission —— 权限管理组合函数
 *
 * 📝 Responsibilities:
 *  1. 提供权限检查的响应式接口
 *  2. 支持单个和多个权限验证
 *  3. 提供基于角色的权限检查
 *  4. 集成用户会话和活动日志
 *
 * 📦 Dependencies:
 *  - Pinia (auth store)
 *  - Vue composition API
 *
 * 🔄 Update Log:
 *  - 2025-07-27  初始创建
 */

import { computed, ref } from 'vue'

import { useAuthStore } from '@/stores/auth'

export function usePermission() {
  const authStore = useAuthStore()

  /**
   * 检查单个权限
   */
  const hasPermission = (permission: string) => {
    return computed(() => authStore.checkPermission(permission))
  }

  /**
   * 检查多个权限
   */
  const hasPermissions = (permissions: string[], requireAll = true) => {
    return computed(() =>
      authStore.checkMultiplePermissions(permissions, requireAll)
    )
  }

  /**
   * 检查用户角色
   */
  const hasRole = (role: string) => {
    return computed(() => authStore.hasRole(role))
  }

  /**
   * 检查用户是否为管理员
   */
  const isAdmin = computed(() => authStore.hasRole('admin'))

  /**
   * 检查用户是否已认证
   */
  const isAuthenticated = computed(() => authStore.isAuthenticated)

  /**
   * 获取当前用户信息
   */
  const currentUser = computed(() => authStore.user)

  /**
   * 获取用户权限列表
   */
  const userPermissions = computed(() => authStore.permissions)

  /**
   * 权限装饰器 - 用于方法调用前的权限检查
   */
  const withPermission = <T extends (...args: any[]) => any>(
    permission: string | string[],
    fn: T,
    options?: {
      requireAll?: boolean
      onDenied?: () => void
      silent?: boolean
    }
  ): T => {
    const { requireAll = true, onDenied, silent = false } = options || {}

    return ((...args: Parameters<T>) => {
      const hasAccess = Array.isArray(permission)
        ? authStore.checkMultiplePermissions(permission, requireAll)
        : authStore.checkPermission(permission)

      if (!hasAccess) {
        if (onDenied) {
          onDenied()
        } else if (!silent) {
          console.warn('权限不足:', permission)
        }
        return
      }

      return fn(...args)
    }) as T
  }

  /**
   * 路由权限检查
   */
  const canAccessRoute = (routePermissions?: string | string[]) => {
    if (!routePermissions) return true

    if (Array.isArray(routePermissions)) {
      return authStore.checkMultiplePermissions(routePermissions, false) // 有其中一个权限即可
    }

    return authStore.checkPermission(routePermissions)
  }

  /**
   * 功能模块权限检查
   */
  const canAccessModule = (module: string) => {
    const modulePermissions: Record<string, string[]> = {
      users: ['users:read'],
      devices: ['devices:read'],
      alerts: ['alerts:read'],
      data: ['data:read'],
      system: ['system:config', 'system:logs'],
      reports: ['reports:generate'],
    }

    const requiredPermissions = modulePermissions[module]
    if (!requiredPermissions) return true

    return authStore.checkMultiplePermissions(requiredPermissions, false)
  }

  /**
   * 操作权限检查（CRUD）
   */
  const canPerformAction = (
    resource: string,
    action: 'create' | 'read' | 'update' | 'delete'
  ) => {
    return authStore.checkPermission(`${resource}:${action}`)
  }

  /**
   * 批量操作权限检查
   */
  const canPerformBatchAction = (resource: string, actions: string[]) => {
    const permissions = actions.map(action => `${resource}:${action}`)
    return authStore.checkMultiplePermissions(permissions, true)
  }

  /**
   * 会话管理权限
   */
  const canManageSessions = computed(
    () =>
      authStore.checkPermission('users:update') || authStore.hasRole('admin')
  )

  /**
   * 日志活动记录
   */
  const logUserActivity = async (activity: {
    type: string
    action: string
    target?: string
    details?: any
  }) => {
    try {
      await authStore.logActivity(activity)
    } catch (error) {
      console.error('Failed to log activity:', error)
    }
  }

  /**
   * 权限状态响应式对象
   */
  const permissionState = computed(() => ({
    isAuthenticated: authStore.isAuthenticated,
    isAdmin: authStore.hasRole('admin'),
    userRole: authStore.userRole,
    permissions: authStore.permissions,
    canCreateUsers: authStore.checkPermission('users:create'),
    canManageDevices: authStore.checkPermission('devices:update'),
    canConfigureSystem: authStore.checkPermission('system:config'),
    canGenerateReports: authStore.checkPermission('reports:generate'),
  }))

  return {
    // 基础权限检查
    hasPermission,
    hasPermissions,
    hasRole,
    isAdmin,
    isAuthenticated,
    currentUser,
    userPermissions,

    // 权限装饰器和工具
    withPermission,
    canAccessRoute,
    canAccessModule,
    canPerformAction,
    canPerformBatchAction,
    canManageSessions,

    // 活动日志
    logUserActivity,

    // 权限状态
    permissionState,
  }
}

/**
 * 权限检查指令工厂
 */
export function createPermissionDirective() {
  return {
    mounted(el: HTMLElement, binding: any) {
      const authStore = useAuthStore()
      const { value, arg, modifiers } = binding

      let hasAccess = false

      if (arg === 'role') {
        // v-permission:role="'admin'"
        hasAccess = authStore.hasRole(value)
      } else if (modifiers.any) {
        // v-permission.any="['perm1', 'perm2']"
        hasAccess = authStore.checkMultiplePermissions(value, false)
      } else if (Array.isArray(value)) {
        // v-permission="['perm1', 'perm2']"
        hasAccess = authStore.checkMultiplePermissions(value, true)
      } else {
        // v-permission="'single-perm'"
        hasAccess = authStore.checkPermission(value)
      }

      if (!hasAccess) {
        if (modifiers.hide) {
          // v-permission.hide - 隐藏元素
          el.style.display = 'none'
        } else {
          // 默认行为 - 移除元素
          el.remove()
        }
      }
    },

    updated(el: HTMLElement, binding: any) {
      // 权限变化时重新检查
      this.mounted(el, binding)
    },
  }
}

/**
 * 权限路由守卫
 */
export function createPermissionGuard() {
  return (to: any, from: any, next: any) => {
    const authStore = useAuthStore()

    // 检查是否需要登录
    if (to.meta?.requiresAuth && !authStore.isAuthenticated) {
      next('/login')
      return
    }

    // 检查角色权限
    if (to.meta?.roles && !authStore.hasRole(to.meta.roles)) {
      next('/403') // 权限不足页面
      return
    }

    // 检查具体权限
    if (to.meta?.permissions) {
      const permissions = Array.isArray(to.meta.permissions)
        ? to.meta.permissions
        : [to.meta.permissions]

      const hasAccess = authStore.checkMultiplePermissions(
        permissions,
        to.meta?.requireAll !== false
      )

      if (!hasAccess) {
        next('/403')
        return
      }
    }

    // 记录页面访问活动
    authStore.logActivity({
      type: 'navigation',
      action: 'visit',
      target: to.path,
      details: {
        from: from.path,
        routeName: to.name,
      },
    })

    next()
  }
}
