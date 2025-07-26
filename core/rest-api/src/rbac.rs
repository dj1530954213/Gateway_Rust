/*!
# RBAC (Role-Based Access Control) System

基于角色的访问控制系统，支持viewer/ops/admin角色
*/

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use chrono::{DateTime, Utc, Duration};

use crate::{Result, ApiError};

/// 用户角色
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// 查看者：只读权限
    Viewer,
    /// 操作员：读写权限，可以修改配置
    Ops,
    /// 管理员：完全权限
    Admin,
}

impl Role {
    /// 检查角色是否有读权限
    pub fn can_read(&self) -> bool {
        matches!(self, Role::Viewer | Role::Ops | Role::Admin)
    }

    /// 检查角色是否有写权限
    pub fn can_write(&self) -> bool {
        matches!(self, Role::Ops | Role::Admin)
    }

    /// 检查角色是否有管理权限
    pub fn can_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Viewer => write!(f, "viewer"),
            Role::Ops => write!(f, "ops"),
            Role::Admin => write!(f, "admin"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "viewer" => Ok(Role::Viewer),
            "ops" => Ok(Role::Ops),
            "admin" => Ok(Role::Admin),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

/// 权限类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// 读取配置
    ConfigRead,
    /// 写入配置
    ConfigWrite,
    /// 读取系统状态
    StatusRead,
    /// 重启服务
    ServiceRestart,
    /// 用户管理
    UserManagement,
    /// 系统管理
    SystemManagement,
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 用户ID
    pub sub: String,
    /// 用户名
    pub username: String,
    /// 用户角色
    pub roles: Vec<Role>,
    /// 过期时间
    pub exp: i64,
    /// 签发时间
    pub iat: i64,
    /// 签发者
    pub iss: String,
}

impl Claims {
    /// 创建新的Claims
    pub fn new(user_id: String, username: String, roles: Vec<Role>) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // 24小时过期

        Self {
            sub: user_id,
            username,
            roles,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: "gateway-api".to_string(),
        }
    }

    /// 检查token是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// 检查是否有指定权限
    pub fn has_permission(&self, permission: Permission) -> bool {
        for role in &self.roles {
            match permission {
                Permission::ConfigRead | Permission::StatusRead => {
                    if role.can_read() {
                        return true;
                    }
                }
                Permission::ConfigWrite | Permission::ServiceRestart => {
                    if role.can_write() {
                        return true;
                    }
                }
                Permission::UserManagement | Permission::SystemManagement => {
                    if role.can_admin() {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub roles: Vec<Role>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl User {
    /// 创建新用户
    pub fn new(id: String, username: String, password: &str, roles: Vec<Role>) -> Self {
        Self {
            id,
            username,
            password_hash: Self::hash_password(password),
            roles,
            active: true,
            created_at: Utc::now(),
            last_login: None,
        }
    }

    /// 验证密码
    pub fn verify_password(&self, password: &str) -> bool {
        self.password_hash == Self::hash_password(password)
    }

    /// 哈希密码
    fn hash_password(password: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 更新最后登录时间
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
    }
}

/// RBAC管理器
#[derive(Clone)]
pub struct RbacManager {
    users: Arc<RwLock<HashMap<String, User>>>,
    jwt_secret: String,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl RbacManager {
    /// 创建新的RBAC管理器
    pub fn new(jwt_secret: String) -> Self {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

        let manager = Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            jwt_secret,
            encoding_key,
            decoding_key,
        };

        // 创建默认用户
        tokio::spawn({
            let manager = manager.clone();
            async move {
                manager.create_default_users().await;
            }
        });

        manager
    }

    /// 创建默认用户
    async fn create_default_users(&self) {
        let mut users = self.users.write().await;

        // 创建默认管理员
        let admin = User::new(
            "admin".to_string(),
            "admin".to_string(),
            "admin123",
            vec![Role::Admin],
        );
        users.insert("admin".to_string(), admin);

        // 创建默认操作员
        let ops = User::new(
            "ops".to_string(),
            "ops".to_string(),
            "ops123",
            vec![Role::Ops],
        );
        users.insert("ops".to_string(), ops);

        // 创建默认查看者
        let viewer = User::new(
            "viewer".to_string(),
            "viewer".to_string(),
            "viewer123",
            vec![Role::Viewer],
        );
        users.insert("viewer".to_string(), viewer);

        tracing::info!("Default RBAC users created: admin, ops, viewer");
    }

    /// 用户登录
    pub async fn login(&self, username: &str, password: &str) -> Result<String> {
        let mut users = self.users.write().await;
        
        // 查找用户
        let user = users.values_mut()
            .find(|u| u.username == username && u.active)
            .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

        // 验证密码
        if !user.verify_password(password) {
            return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
        }

        // 更新最后登录时间
        user.update_last_login();

        // 生成JWT token
        let claims = Claims::new(
            user.id.clone(),
            user.username.clone(),
            user.roles.clone(),
        );

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ApiError::Internal(format!("Failed to generate token: {}", e)))?;

        tracing::info!("User {} logged in successfully", username);
        Ok(token)
    }

    /// 验证JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        let token_data: TokenData<Claims> = decode(token, &self.decoding_key, &validation)
            .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))?;

        if token_data.claims.is_expired() {
            return Err(ApiError::Unauthorized("Token has expired".to_string()));
        }

        Ok(token_data.claims)
    }

    /// 检查权限
    pub fn check_permission(&self, claims: &Claims, permission: Permission) -> Result<()> {
        if claims.has_permission(permission) {
            Ok(())
        } else {
            Err(ApiError::Forbidden("Insufficient permissions".to_string()))
        }
    }

    /// 添加用户
    pub async fn add_user(&self, user: User) -> Result<()> {
        let mut users = self.users.write().await;
        
        if users.contains_key(&user.id) {
            return Err(ApiError::BadRequest("User already exists".to_string()));
        }

        users.insert(user.id.clone(), user);
        Ok(())
    }

    /// 获取用户
    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }

    /// 列出所有用户
    pub async fn list_users(&self) -> Vec<User> {
        let users = self.users.read().await;
        users.values().cloned().collect()
    }

    /// 删除用户
    pub async fn remove_user(&self, user_id: &str) -> Result<()> {
        let mut users = self.users.write().await;
        
        if user_id == "admin" {
            return Err(ApiError::BadRequest("Cannot delete admin user".to_string()));
        }

        users.remove(user_id)
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        Ok(())
    }

    /// 更新用户角色
    pub async fn update_user_roles(&self, user_id: &str, roles: Vec<Role>) -> Result<()> {
        let mut users = self.users.write().await;
        
        let user = users.get_mut(user_id)
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        user.roles = roles;
        Ok(())
    }

    /// 获取角色权限映射
    pub fn get_role_permissions() -> HashMap<Role, Vec<Permission>> {
        let mut permissions = HashMap::new();

        permissions.insert(Role::Viewer, vec![
            Permission::ConfigRead,
            Permission::StatusRead,
        ]);

        permissions.insert(Role::Ops, vec![
            Permission::ConfigRead,
            Permission::ConfigWrite,
            Permission::StatusRead,
            Permission::ServiceRestart,
        ]);

        permissions.insert(Role::Admin, vec![
            Permission::ConfigRead,
            Permission::ConfigWrite,
            Permission::StatusRead,
            Permission::ServiceRestart,
            Permission::UserManagement,
            Permission::SystemManagement,
        ]);

        permissions
    }
}

/// 身份验证中间件扩展
pub trait AuthExtension {
    /// 从请求中提取Claims
    fn get_claims(&self) -> Option<&Claims>;
    
    /// 检查权限
    fn check_permission(&self, permission: Permission) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rbac_manager() {
        let manager = RbacManager::new("test-secret".to_string());
        
        // 等待默认用户创建
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 测试登录
        let token = manager.login("admin", "admin123").await.unwrap();
        assert!(!token.is_empty());
        
        // 测试token验证
        let claims = manager.validate_token(&token).unwrap();
        assert_eq!(claims.username, "admin");
        assert!(claims.roles.contains(&Role::Admin));
    }

    #[test]
    fn test_role_permissions() {
        let viewer_role = Role::Viewer;
        assert!(viewer_role.can_read());
        assert!(!viewer_role.can_write());
        assert!(!viewer_role.can_admin());

        let ops_role = Role::Ops;
        assert!(ops_role.can_read());
        assert!(ops_role.can_write());
        assert!(!ops_role.can_admin());

        let admin_role = Role::Admin;
        assert!(admin_role.can_read());
        assert!(admin_role.can_write());
        assert!(admin_role.can_admin());
    }

    #[test]
    fn test_claims_permissions() {
        let claims = Claims::new(
            "test".to_string(),
            "test".to_string(),
            vec![Role::Ops],
        );

        assert!(claims.has_permission(Permission::ConfigRead));
        assert!(claims.has_permission(Permission::ConfigWrite));
        assert!(!claims.has_permission(Permission::UserManagement));
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test".to_string(),
            "testuser".to_string(),
            "password123",
            vec![Role::Viewer],
        );

        assert_eq!(user.username, "testuser");
        assert!(user.verify_password("password123"));
        assert!(!user.verify_password("wrongpassword"));
        assert!(user.roles.contains(&Role::Viewer));
    }
}