# 🔐 配置管理安全改进计划

## 当前配置安全风险评估 ⚠️ 高风险

### 发现的安全问题
1. **硬编码密钥** - 数据库密码明文存储
2. **配置文件暴露** - 敏感信息提交到代码仓库
3. **缺乏加密** - 配置传输和存储未加密
4. **权限过松** - 配置文件访问权限不受限

## 立即执行的安全修复

### 1. 密钥管理系统集成 (本周完成)

#### Kubernetes Secret管理
```yaml
# kubernetes/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: gateway-secrets
  namespace: industrial-gateway
type: Opaque
stringData:
  pg_dsn: "postgres://gateway_user:${PG_PASSWORD}@postgres-svc:5432/iot"
  influx_token: "${INFLUX_TOKEN}"
  jwt_secret: "${JWT_SECRET}"
  mqtt_password: "${MQTT_PASSWORD}"
```

#### 配置引用Secret
```yaml
# deployment.yaml
spec:
  containers:
  - name: gateway
    env:
    - name: WEBGW_PG_DSN
      valueFrom:
        secretKeyRef:
          name: gateway-secrets
          key: pg_dsn
    - name: WEBGW_INFLUX_TOKEN
      valueFrom:
        secretKeyRef:
          name: gateway-secrets
          key: influx_token
```

### 2. 配置加密方案 (下周完成)

#### Sealed Secrets集成
```bash
# 安装Sealed Secrets Controller
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.18.0/controller.yaml

# 创建加密配置
echo -n mypassword | kubectl create secret generic gateway-secrets --dry-run=client --from-file=password=/dev/stdin -o yaml | kubeseal -o yaml
```

#### SOPS配置文件加密
```yaml
# .sops.yaml
creation_rules:
  - path_regex: .*\.enc\.yaml$
    kms: 'arn:aws:kms:region:account:key/key-id'
    age: 'age1...'
```

### 3. 环境变量完整覆盖

#### 配置结构重构
```rust
// config-manager/src/lib.rs
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub http: HttpConfig,
    
    #[serde(default)]
    pub database: DatabaseConfig,
    
    #[serde(default)]
    pub security: SecurityConfig,
}

#[derive(Debug, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub tls_enabled: bool,
    
    #[serde(env = "WEBGW_JWT_SECRET")]
    pub jwt_secret: Option<String>,
    
    #[serde(env = "WEBGW_ENCRYPTION_KEY")]
    pub encryption_key: Option<String>,
}
```

### 4. 配置验证机制

#### 启动时配置检查
```rust
impl AppConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 检查必需的安全配置
        if self.security.jwt_secret.is_none() {
            return Err(ConfigError::MissingRequired("JWT_SECRET"));
        }
        
        // 检查数据库连接配置
        if !self.database.dsn.starts_with("postgres://") {
            return Err(ConfigError::InvalidFormat("pg_dsn"));
        }
        
        // 检查TLS配置
        if self.security.tls_enabled && self.security.cert_path.is_none() {
            return Err(ConfigError::MissingRequired("TLS_CERT_PATH"));
        }
        
        Ok(())
    }
}
```

## 运维安全最佳实践

### 配置访问控制
- Pod只能访问所需的Secret
- 使用ServiceAccount权限分离
- 定期轮换密钥
- 审计配置访问日志

### 配置热更新机制
```rust
// 监听配置文件变更
use notify::{Watcher, RecursiveMode, Event};

pub async fn watch_config_changes() -> Result<(), Error> {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    
    watcher.watch(Path::new("/app/config"), RecursiveMode::Recursive)?;
    
    for res in rx {
        match res {
            Ok(event) => {
                if let Some(path) = event.paths.first() {
                    reload_config(path).await?;
                }
            }
            Err(e) => error!("配置文件监听错误: {:?}", e),
        }
    }
    
    Ok(())
}
```

### 配置备份和恢复
```bash
#!/bin/bash
# 配置备份脚本
kubectl get secrets -n industrial-gateway -o yaml > secrets-backup-$(date +%Y%m%d).yaml
kubectl get configmaps -n industrial-gateway -o yaml > configmaps-backup-$(date +%Y%m%d).yaml
```

## 实施时间表

| 任务 | 优先级 | 完成时间 |
|-----|-------|---------|
| 移除硬编码密钥 | 🔴 极高 | 本周内 |
| 集成Kubernetes Secret | 🔴 高 | 本周内 |
| 实施配置加密 | 🟡 中 | 下周内 |
| 添加配置验证 | 🟡 中 | 下周内 |
| 配置热更新 | 🟢 低 | 两周内 |