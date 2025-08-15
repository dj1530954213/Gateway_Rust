# 🔐 Gateway_Rust 安全配置指南

## ⚠️ 重要安全警告

**绝对禁止在代码仓库中存储任何真实凭据！**

本系统已移除所有硬编码凭据，请按照本指南正确配置生产环境。

## 🛡️ 安全配置步骤

### 1. 环境变量配置

#### 生产环境 (推荐方式)
使用系统环境变量或密钥管理服务：

```bash
# 数据库配置
export DATABASE_URL="postgres://postgres:YOUR_SECURE_PASSWORD@localhost:5432/gateway_rust"

# InfluxDB配置  
export INFLUX_URL="https://your-influx-server:8086"
export INFLUX_TOKEN="YOUR_SECURE_INFLUX_TOKEN"

# 安全密钥
export GATEWAY_JWT_SECRET="YOUR_SECURE_32_CHAR_JWT_SECRET"
export GRAFANA_SECRET_KEY="YOUR_SECURE_32_CHAR_GRAFANA_KEY"

# 管理员密码
export INFLUXDB_ADMIN_PASSWORD="YOUR_SECURE_INFLUX_PASSWORD"  
export GRAFANA_ADMIN_PASSWORD="YOUR_SECURE_GRAFANA_PASSWORD"
```

#### 开发环境
创建 `.env.local` 文件（此文件不会被提交到Git）：

```bash
# 复制示例配置
cp .env.example .env.local

# 编辑配置文件，设置开发环境凭据
vi .env.local
```

### 2. 密钥管理最佳实践

#### 密码强度要求
- **最少16字符**，推荐32字符
- 包含大小写字母、数字、特殊字符
- 使用密码管理器生成随机密码

#### 密钥轮换
```bash
# 每90天轮换一次密钥
# JWT Secret 建议每30天轮换
```

### 3. 生产环境部署方案

#### 方案A：Docker Secrets (推荐)
```yaml
# docker-compose.yml
version: '3.8'
services:
  gateway:
    image: gateway-rust:latest
    secrets:
      - db_password
      - influx_token
    environment:
      DATABASE_URL_FILE: /run/secrets/db_password
      INFLUX_TOKEN_FILE: /run/secrets/influx_token

secrets:
  db_password:
    external: true
  influx_token:
    external: true
```

#### 方案B：Kubernetes Secrets
```yaml
# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: gateway-secrets
type: Opaque
data:
  database-url: <base64_encoded>
  influx-token: <base64_encoded>
```

#### 方案C：外部密钥管理
- **HashiCorp Vault**
- **AWS Secrets Manager** 
- **Azure Key Vault**
- **Google Secret Manager**

### 4. 配置文件安全检查

#### 检查硬编码凭据
```bash
# 扫描可能的硬编码密码
grep -r -i "password\|secret\|token\|key" config/ --include="*.yaml" --include="*.yml"

# 确保输出中没有真实凭据
```

#### 验证环境变量
```bash
# 确认环境变量已正确设置
./scripts/check_env.sh
```

### 5. 安全配置清单

#### 部署前必检项目
- [ ] 所有 `.env.*` 文件已添加到 `.gitignore`
- [ ] 配置文件中无硬编码凭据
- [ ] 环境变量已正确设置
- [ ] 密钥强度符合要求
- [ ] 启用密钥轮换机制
- [ ] 配置访问控制和审计日志

## 🚨 应急响应

### 如果凭据泄露
1. **立即轮换所有受影响密钥**
2. **检查访问日志，识别异常活动**
3. **更新所有部署环境**
4. **通知相关安全团队**

### 联系方式
- 安全事件: security@company.com
- 紧急响应: +86-xxx-xxxx-xxxx

## 📋 参考资源

- [OWASP 密钥管理最佳实践](https://owasp.org/www-project-key-management-best-practices/)
- [NIST 密码学标准](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
- [12-Factor App 配置原则](https://12factor.net/config)

---

**记住：安全是一个持续过程，不是一次性任务！**