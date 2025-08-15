/**
 * Validation utilities for forms and inputs
 */

export interface ValidationRule {
  required?: boolean
  min?: number
  max?: number
  pattern?: RegExp
  validator?: (value: any) => boolean | string
  message?: string
}

export interface ValidationResult {
  valid: boolean
  message?: string
}

/**
 * Validate a single value against rules
 */
export function validateValue(
  value: any,
  rules: ValidationRule[]
): ValidationResult {
  for (const rule of rules) {
    const result = validateSingleRule(value, rule)
    if (!result.valid) {
      return result
    }
  }

  return { valid: true }
}

/**
 * Validate a single rule
 */
function validateSingleRule(
  value: any,
  rule: ValidationRule
): ValidationResult {
  // Required validation
  if (
    rule.required &&
    (value === null || value === undefined || value === '')
  ) {
    return {
      valid: false,
      message: rule.message || '此字段为必填项',
    }
  }

  // Skip other validations if value is empty and not required
  if (
    !rule.required &&
    (value === null || value === undefined || value === '')
  ) {
    return { valid: true }
  }

  // Min length/value validation
  if (rule.min !== undefined) {
    if (typeof value === 'string' && value.length < rule.min) {
      return {
        valid: false,
        message: rule.message || `最少需要${rule.min}个字符`,
      }
    }
    if (typeof value === 'number' && value < rule.min) {
      return {
        valid: false,
        message: rule.message || `最小值为${rule.min}`,
      }
    }
  }

  // Max length/value validation
  if (rule.max !== undefined) {
    if (typeof value === 'string' && value.length > rule.max) {
      return {
        valid: false,
        message: rule.message || `最多只能输入${rule.max}个字符`,
      }
    }
    if (typeof value === 'number' && value > rule.max) {
      return {
        valid: false,
        message: rule.message || `最大值为${rule.max}`,
      }
    }
  }

  // Pattern validation
  if (rule.pattern && !rule.pattern.test(String(value))) {
    return {
      valid: false,
      message: rule.message || '格式不正确',
    }
  }

  // Custom validator
  if (rule.validator) {
    const result = rule.validator(value)
    if (result !== true) {
      return {
        valid: false,
        message:
          typeof result === 'string' ? result : rule.message || '验证失败',
      }
    }
  }

  return { valid: true }
}

/**
 * Common validation patterns
 */
export const ValidationPatterns = {
  // Email validation
  email: /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/,

  // Phone number (Chinese mobile)
  phone: /^1[3-9]\d{9}$/,

  // IP address (IPv4)
  ipv4: /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/,

  // MAC address
  mac: /^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$/,

  // URL
  url: /^(https?|ftp):\/\/[^\s/$.?#].[^\s]*$/i,

  // Domain name
  domain: /^[a-zA-Z0-9][a-zA-Z0-9-]{1,61}[a-zA-Z0-9]\.[a-zA-Z]{2,}$/,

  // Username (alphanumeric, underscore, hyphen)
  username: /^[a-zA-Z0-9_-]{3,20}$/,

  // Password (at least 8 chars, include letters and numbers)
  password: /^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d@$!%*#?&]{8,}$/,

  // Numbers only
  numeric: /^\d+$/,

  // Decimal numbers
  decimal: /^\d+(\.\d+)?$/,

  // Hexadecimal
  hex: /^[0-9A-Fa-f]+$/,

  // Version string (semantic versioning)
  version: /^\d+\.\d+\.\d+(-[a-zA-Z0-9-]+)?$/,
}

/**
 * Pre-defined validation rules
 */
export const ValidationRules = {
  required: (message?: string): ValidationRule => ({
    required: true,
    message: message || '此字段为必填项',
  }),

  email: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.email,
    message: message || '请输入有效的邮箱地址',
  }),

  phone: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.phone,
    message: message || '请输入有效的手机号码',
  }),

  url: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.url,
    message: message || '请输入有效的URL地址',
  }),

  ipv4: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.ipv4,
    validator: (value: string) => {
      if (!ValidationPatterns.ipv4.test(value)) return false

      const parts = value.split('.')
      return parts.every(part => {
        const num = parseInt(part, 10)
        return num >= 0 && num <= 255
      })
    },
    message: message || '请输入有效的IPv4地址',
  }),

  port: (message?: string): ValidationRule => ({
    validator: (value: any) => {
      const port = parseInt(value, 10)
      return !isNaN(port) && port >= 1 && port <= 65535
    },
    message: message || '端口号必须在1-65535之间',
  }),

  macAddress: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.mac,
    message: message || '请输入有效的MAC地址',
  }),

  username: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.username,
    message:
      message || '用户名只能包含字母、数字、下划线和连字符，长度3-20个字符',
  }),

  password: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.password,
    message: message || '密码至少8位，必须包含字母和数字',
  }),

  numeric: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.numeric,
    message: message || '请输入数字',
  }),

  decimal: (message?: string): ValidationRule => ({
    pattern: ValidationPatterns.decimal,
    message: message || '请输入有效的数字',
  }),

  minLength: (min: number, message?: string): ValidationRule => ({
    min,
    message: message || `最少需要${min}个字符`,
  }),

  maxLength: (max: number, message?: string): ValidationRule => ({
    max,
    message: message || `最多只能输入${max}个字符`,
  }),

  range: (min: number, max: number, message?: string): ValidationRule => ({
    min,
    max,
    message: message || `值必须在${min}-${max}之间`,
  }),

  json: (message?: string): ValidationRule => ({
    validator: (value: string) => {
      if (!value) return true
      try {
        JSON.parse(value)
        return true
      } catch {
        return false
      }
    },
    message: message || '请输入有效的JSON格式',
  }),
}

/**
 * Validate form data
 */
export function validateForm(
  data: Record<string, any>,
  rules: Record<string, ValidationRule[]>
): { valid: boolean; errors: Record<string, string> } {
  const errors: Record<string, string> = {}

  for (const [field, fieldRules] of Object.entries(rules)) {
    const value = data[field]
    const result = validateValue(value, fieldRules)

    if (!result.valid) {
      errors[field] = result.message || '验证失败'
    }
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors,
  }
}

/**
 * Validate driver configuration
 */
export function validateDriverConfig(
  config: any,
  driverType: string
): ValidationResult {
  const baseRules = [ValidationRules.required()]

  switch (driverType) {
    case 'modbus_tcp':
    case 'modbus_rtu':
      return validateValue(config, [
        ...baseRules,
        {
          validator: (value: any) => {
            if (!value.host) return '主机地址为必填项'
            if (!value.port) return '端口号为必填项'
            if (value.polling_interval <= 0) return '轮询间隔必须大于0'
            return true
          },
        },
      ])

    case 'opc_ua':
      return validateValue(config, [
        ...baseRules,
        {
          validator: (value: any) => {
            if (!value.endpoint_url) return 'OPC UA端点URL为必填项'
            if (!ValidationPatterns.url.test(value.endpoint_url)) {
              return '请输入有效的OPC UA端点URL'
            }
            return true
          },
        },
      ])

    default:
      return { valid: true }
  }
}

/**
 * Validate connector configuration
 */
export function validateConnectorConfig(
  config: any,
  connectorType: string
): ValidationResult {
  const baseRules = [ValidationRules.required()]

  switch (connectorType) {
    case 'mqtt':
      return validateValue(config, [
        ...baseRules,
        {
          validator: (value: any) => {
            if (!value.endpoint) return 'MQTT服务器地址为必填项'
            if (!value.topic) return 'MQTT主题为必填项'
            return true
          },
        },
      ])

    case 'http':
      return validateValue(config, [
        ...baseRules,
        {
          validator: (value: any) => {
            if (!value.endpoint) return 'HTTP端点为必填项'
            if (!ValidationPatterns.url.test(value.endpoint)) {
              return '请输入有效的HTTP端点URL'
            }
            return true
          },
        },
      ])

    case 'influxdb':
      return validateValue(config, [
        ...baseRules,
        {
          validator: (value: any) => {
            if (!value.endpoint) return 'InfluxDB服务器地址为必填项'
            if (!value.database) return '数据库名称为必填项'
            return true
          },
        },
      ])

    default:
      return { valid: true }
  }
}

/**
 * Validate data point configuration
 */
export function validateDataPoint(dataPoint: any): ValidationResult {
  const rules: ValidationRule[] = [
    ValidationRules.required('数据点名称为必填项'),
  ]

  const nameResult = validateValue(dataPoint.name, rules)
  if (!nameResult.valid) return nameResult

  const addressResult = validateValue(dataPoint.address, [
    ValidationRules.required('数据点地址为必填项'),
  ])
  if (!addressResult.valid) return addressResult

  // Validate address format based on data type
  if (dataPoint.data_type && dataPoint.address) {
    const addressValidation = validateDataPointAddress(
      dataPoint.address,
      dataPoint.data_type
    )
    if (!addressValidation.valid) return addressValidation
  }

  // Validate scale factor and offset
  if (dataPoint.scale_factor !== undefined && dataPoint.scale_factor === 0) {
    return {
      valid: false,
      message: '缩放因子不能为0',
    }
  }

  // Validate min/max values
  if (dataPoint.min_value !== undefined && dataPoint.max_value !== undefined) {
    if (dataPoint.min_value >= dataPoint.max_value) {
      return {
        valid: false,
        message: '最小值必须小于最大值',
      }
    }
  }

  return { valid: true }
}

/**
 * Validate data point address format
 */
function validateDataPointAddress(
  address: string,
  dataType: string
): ValidationResult {
  // Basic address format validation
  if (!address || address.trim() === '') {
    return {
      valid: false,
      message: '地址不能为空',
    }
  }

  // For Modbus addresses, check format
  if (address.startsWith('4') || address.startsWith('3')) {
    const addressNum = parseInt(address, 10)
    if (isNaN(addressNum) || addressNum < 1 || addressNum > 65535) {
      return {
        valid: false,
        message: 'Modbus地址必须在1-65535之间',
      }
    }
  }

  return { valid: true }
}

/**
 * Sanitize input to prevent XSS
 */
export function sanitizeInput(input: string): string {
  if (typeof input !== 'string') return ''

  return input
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
    .replace(/\//g, '&#x2F;')
}

/**
 * Check if value is safe for SQL queries (basic check)
 */
export function isSQLSafe(value: string): boolean {
  if (typeof value !== 'string') return true

  const sqlPatterns = [
    /(\b(SELECT|INSERT|UPDATE|DELETE|DROP|CREATE|ALTER|EXEC|UNION)\b)/i,
    /(;|--|\*|\/\*|\*\/)/,
    /(\bOR\b|\bAND\b)\s+\d+\s*=\s*\d+/i,
  ]

  return !sqlPatterns.some(pattern => pattern.test(value))
}
