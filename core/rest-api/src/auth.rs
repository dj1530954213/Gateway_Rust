/*!
# Authentication and Authorization

JWT-based authentication and role-based authorization for REST API.
*/

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use frame_bus::permissions::{Permission, PermissionManager};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // Subject (user ID)
    pub name: String,         // User name
    pub roles: Vec<String>,   // User roles
    pub exp: u64,             // Expiration time
    pub iat: u64,             // Issued at
}

impl Claims {
    /// Create new claims
    pub fn new(user_id: String, name: String, roles: Vec<String>, expires_in_hours: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            sub: user_id,
            name,
            roles,
            exp: now + (expires_in_hours * 3600),
            iat: now,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.exp
    }
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub active: bool,
}

impl User {
    /// Create new user
    pub fn new(id: String, name: String, email: String, password: &str) -> Self {
        Self {
            id,
            name,
            email,
            password_hash: hash_password(password),
            roles: vec!["user".to_string()],
            active: true,
        }
    }

    /// Verify password
    pub fn verify_password(&self, password: &str) -> bool {
        self.password_hash == hash_password(password)
    }

    /// Check if user has role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

/// Authentication manager
pub struct AuthManager {
    /// JWT encoding/decoding keys
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    
    /// User storage
    users: Arc<RwLock<HashMap<String, User>>>,
    
    /// Permission manager
    permission_manager: Arc<PermissionManager>,
    
    /// JWT validation settings
    validation: Validation,
}

impl AuthManager {
    /// Create new auth manager
    pub fn new(jwt_secret: &str, permission_manager: Arc<PermissionManager>) -> Self {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
        
        let mut validation = Validation::default();
        validation.validate_exp = true;

        Self {
            encoding_key,
            decoding_key,
            users: Arc::new(RwLock::new(HashMap::new())),
            permission_manager,
            validation,
        }
    }

    /// Add user
    pub fn add_user(&self, user: User) {
        let mut users = self.users.write().unwrap();
        users.insert(user.id.clone(), user);
    }

    /// Authenticate user with username/password
    pub fn authenticate(&self, username: &str, password: &str) -> Result<String> {
        let users = self.users.read().unwrap();
        
        let user = users.values()
            .find(|u| u.name == username || u.email == username)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        if !user.active {
            return Err(anyhow::anyhow!("User account is disabled"));
        }

        if !user.verify_password(password) {
            return Err(anyhow::anyhow!("Invalid password"));
        }

        // Generate JWT token
        let claims = Claims::new(
            user.id.clone(),
            user.name.clone(),
            user.roles.clone(),
            24, // 24 hours
        );

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .context("Failed to generate JWT token")?;

        Ok(token)
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data: TokenData<Claims> = decode(token, &self.decoding_key, &self.validation)
            .context("Invalid JWT token")?;

        if token_data.claims.is_expired() {
            return Err(anyhow::anyhow!("Token has expired"));
        }

        Ok(token_data.claims)
    }

    /// Get user by ID
    pub fn get_user(&self, user_id: &str) -> Option<User> {
        let users = self.users.read().unwrap();
        users.get(user_id).cloned()
    }

    /// Check if user has permission for resource
    pub fn check_permission(&self, user_id: &str, resource: &str, permission: Permission) -> bool {
        self.permission_manager.check_permission(user_id, resource, permission)
    }

    /// Get user permissions for resource
    pub fn get_permissions(&self, user_id: &str, resource: &str) -> std::collections::HashSet<Permission> {
        self.permission_manager.get_permissions(user_id, resource)
    }

    /// Create admin user (for testing/setup)
    pub fn create_admin_user(&self, username: &str, password: &str) -> Result<()> {
        let admin_user = User {
            id: "admin".to_string(),
            name: username.to_string(),
            email: format!("{}@localhost", username),
            password_hash: hash_password(password),
            roles: vec!["admin".to_string(), "user".to_string()],
            active: true,
        };

        self.add_user(admin_user);

        // Grant admin permissions
        use frame_bus::permissions::AccessControlEntry;
        use std::collections::HashSet;

        let mut admin_perms = HashSet::new();
        admin_perms.insert(Permission::Admin);
        admin_perms.insert(Permission::Write);
        admin_perms.insert(Permission::Read);

        self.permission_manager.add_ace(
            AccessControlEntry::allow("admin", "*", admin_perms)
                .with_priority(1000)
        );

        Ok(())
    }
}

/// Hash password using SHA256
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

/// User info for response
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            roles: user.roles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test123";
        let hash1 = hash_password(password);
        let hash2 = hash_password(password);
        assert_eq!(hash1, hash2);
        
        let wrong_password = "wrong";
        let hash3 = hash_password(wrong_password);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "user1".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password123"
        );

        assert_eq!(user.id, "user1");
        assert_eq!(user.name, "Test User");
        assert!(user.verify_password("password123"));
        assert!(!user.verify_password("wrong"));
        assert!(user.has_role("user"));
        assert!(!user.has_role("admin"));
    }

    #[test]
    fn test_jwt_token() {
        let permission_manager = Arc::new(PermissionManager::new());
        let auth_manager = AuthManager::new("test-secret", permission_manager);

        let user = User::new(
            "test".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password"
        );
        auth_manager.add_user(user);

        // Test authentication
        let token = auth_manager.authenticate("Test User", "password").unwrap();
        assert!(!token.is_empty());

        // Test token validation
        let claims = auth_manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "test");
        assert_eq!(claims.name, "Test User");
    }
}