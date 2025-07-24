/*!
# Permission Management

Provides fine-grained permission control for frame operations and commands.
*/

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use tracing::debug;

use crate::envelope::{CmdFrame, DataFrame};

/// Permission levels for data access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read permission - can read data
    Read,
    /// Write permission - can write/modify data  
    Write,
    /// Admin permission - can manage permissions
    Admin,
}

impl Permission {
    /// Check if this permission includes another permission
    pub fn includes(&self, other: Permission) -> bool {
        match (self, other) {
            (Permission::Admin, _) => true,
            (Permission::Write, Permission::Read) => true,
            (Permission::Write, Permission::Write) => true,
            (Permission::Read, Permission::Read) => true,
            _ => false,
        }
    }
}

/// Access control entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    /// Subject (user, role, or service)
    pub subject: String,
    
    /// Resource pattern (tag pattern or wildcard)
    pub resource: String,
    
    /// Granted permissions
    pub permissions: HashSet<Permission>,
    
    /// Whether this is an allow or deny rule
    pub allow: bool,
    
    /// Rule priority (higher number = higher priority)
    pub priority: u32,
}

impl AccessControlEntry {
    /// Create new ACE
    pub fn new<S: Into<String>>(
        subject: S,
        resource: S,
        permissions: HashSet<Permission>,
        allow: bool,
    ) -> Self {
        Self {
            subject: subject.into(),
            resource: resource.into(),
            permissions,
            allow,
            priority: 100,
        }
    }
    
    /// Create allow rule
    pub fn allow<S: Into<String>>(
        subject: S,
        resource: S,
        permissions: HashSet<Permission>,
    ) -> Self {
        Self::new(subject, resource, permissions, true)
    }
    
    /// Create deny rule
    pub fn deny<S: Into<String>>(
        subject: S,
        resource: S,
        permissions: HashSet<Permission>,
    ) -> Self {
        Self::new(subject, resource, permissions, false)
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Check if this ACE matches the given subject and resource
    pub fn matches(&self, subject: &str, resource: &str) -> bool {
        self.matches_subject(subject) && self.matches_resource(resource)
    }
    
    /// Check if subject matches
    fn matches_subject(&self, subject: &str) -> bool {
        // Exact match or wildcard
        self.subject == "*" || self.subject == subject
    }
    
    /// Check if resource matches (supports wildcards)
    fn matches_resource(&self, resource: &str) -> bool {
        if self.resource == "*" {
            return true;
        }
        
        // Support simple glob patterns
        if self.resource.contains('*') {
            self.glob_match(&self.resource, resource)
        } else {
            self.resource == resource
        }
    }
    
    /// Simple glob pattern matching
    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        // Split pattern by '*' and check each part
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.is_empty() {
            return text.is_empty();
        }
        
        let mut text_pos = 0;
        
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            
            if i == 0 {
                // First part must match at the beginning
                if !text.starts_with(part) {
                    return false;
                }
                text_pos = part.len();
            } else if i == parts.len() - 1 {
                // Last part must match at the end
                return text[text_pos..].ends_with(part);
            } else {
                // Middle part must exist somewhere
                if let Some(pos) = text[text_pos..].find(part) {
                    text_pos += pos + part.len();
                } else {
                    return false;
                }
            }
        }
        
        true
    }
}

/// Permission manager
pub struct PermissionManager {
    /// Access control entries
    aces: Arc<RwLock<Vec<AccessControlEntry>>>,
    
    /// Default permissions for new subjects
    default_permissions: HashSet<Permission>,
    
    /// Whether to allow access by default when no rules match
    default_allow: bool,
}

impl PermissionManager {
    /// Create new permission manager
    pub fn new() -> Self {
        Self {
            aces: Arc::new(RwLock::new(Vec::new())),
            default_permissions: HashSet::new(),
            default_allow: false,
        }
    }
    
    /// Create permission manager with default allow
    pub fn with_default_allow() -> Self {
        let mut manager = Self::new();
        manager.default_allow = true;
        manager.default_permissions.insert(Permission::Read);
        manager
    }
    
    /// Add access control entry
    pub fn add_ace(&self, ace: AccessControlEntry) {
        let mut aces = self.aces.write().unwrap();
        aces.push(ace);
        
        // Sort by priority (highest first)
        aces.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Remove access control entries for subject
    pub fn remove_subject(&self, subject: &str) {
        let mut aces = self.aces.write().unwrap();
        aces.retain(|ace| ace.subject != subject);
    }
    
    /// Check if subject has permission for resource
    pub fn check_permission(
        &self,
        subject: &str,
        resource: &str,
        permission: Permission,
    ) -> bool {
        let aces = self.aces.read().unwrap();
        
        // Check ACEs in priority order
        for ace in aces.iter() {
            if ace.matches(subject, resource) && ace.permissions.contains(&permission) {
                debug!("Permission {} for {}/{}: {} (rule: {})", 
                       permission_to_string(permission), subject, resource, 
                       ace.allow, ace.resource);
                return ace.allow;
            }
        }
        
        // No specific rule found, use default
        if self.default_allow && self.default_permissions.contains(&permission) {
            debug!("Permission {} for {}/{}: {} (default)", 
                   permission_to_string(permission), subject, resource, true);
            true
        } else {
            debug!("Permission {} for {}/{}: {} (no rule)", 
                   permission_to_string(permission), subject, resource, false);
            false
        }
    }
    
    /// Check if subject can read data frame
    pub fn can_read_frame(&self, subject: &str, frame: &DataFrame) -> bool {
        self.check_permission(subject, &frame.tag, Permission::Read)
    }
    
    /// Check if subject can execute command
    pub fn can_execute_command(&self, subject: &str, command: &CmdFrame) -> bool {
        self.check_permission(subject, &command.tag, Permission::Write)
    }
    
    /// Get all permissions for subject and resource
    pub fn get_permissions(&self, subject: &str, resource: &str) -> HashSet<Permission> {
        let mut granted = HashSet::new();
        let aces = self.aces.read().unwrap();
        
        // Collect permissions from all matching rules
        for ace in aces.iter() {
            if ace.matches(subject, resource) {
                if ace.allow {
                    granted.extend(&ace.permissions);
                } else {
                    // Remove denied permissions
                    for perm in &ace.permissions {
                        granted.remove(perm);
                    }
                }
            }
        }
        
        // Add default permissions if allowed
        if self.default_allow {
            granted.extend(&self.default_permissions);
        }
        
        granted
    }
    
    /// Get all subjects
    pub fn get_subjects(&self) -> Vec<String> {
        let aces = self.aces.read().unwrap();
        let subjects: HashSet<String> = aces.iter()
            .map(|ace| ace.subject.clone())
            .collect();
        
        subjects.into_iter().collect()
    }
    
    /// Get all resources for subject
    pub fn get_resources(&self, subject: &str) -> Vec<String> {
        let aces = self.aces.read().unwrap();
        aces.iter()
            .filter(|ace| ace.subject == subject)
            .map(|ace| ace.resource.clone())
            .collect()
    }
    
    /// Set default permissions
    pub fn set_default_permissions(&mut self, permissions: HashSet<Permission>) {
        self.default_permissions = permissions;
    }
    
    /// Clear all ACEs
    pub fn clear(&self) {
        let mut aces = self.aces.write().unwrap();
        aces.clear();
    }
    
    /// Get ACE count
    pub fn ace_count(&self) -> usize {
        let aces = self.aces.read().unwrap();
        aces.len()
    }
    
    /// Export ACEs to JSON
    pub fn export_aces(&self) -> Result<String> {
        let aces = self.aces.read().unwrap();
        serde_json::to_string_pretty(&*aces)
            .context("Failed to serialize ACEs")
    }
    
    /// Import ACEs from JSON
    pub fn import_aces(&self, json: &str) -> Result<()> {
        let imported_aces: Vec<AccessControlEntry> = serde_json::from_str(json)
            .context("Failed to deserialize ACEs")?;
        
        let mut aces = self.aces.write().unwrap();
        aces.extend(imported_aces);
        
        // Sort by priority
        aces.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to convert permission to string
fn permission_to_string(permission: Permission) -> &'static str {
    match permission {
        Permission::Read => "READ",
        Permission::Write => "WRITE",
        Permission::Admin => "ADMIN",
    }
}

/// Permission check result with details
#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub permission: Permission,
    pub subject: String,
    pub resource: String,
    pub matching_rule: Option<String>,
}

impl PermissionManager {
    /// Check permission with detailed result
    pub fn check_permission_detailed(
        &self,
        subject: &str,
        resource: &str,
        permission: Permission,
    ) -> PermissionCheckResult {
        let aces = self.aces.read().unwrap();
        
        // Check ACEs in priority order
        for ace in aces.iter() {
            if ace.matches(subject, resource) && ace.permissions.contains(&permission) {
                return PermissionCheckResult {
                    allowed: ace.allow,
                    permission,
                    subject: subject.to_string(),
                    resource: resource.to_string(),
                    matching_rule: Some(format!("{}:{}", ace.subject, ace.resource)),
                };
            }
        }
        
        // No specific rule found, use default
        let allowed = self.default_allow && self.default_permissions.contains(&permission);
        
        PermissionCheckResult {
            allowed,
            permission,
            subject: subject.to_string(),
            resource: resource.to_string(),
            matching_rule: if allowed { Some("default".to_string()) } else { None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_includes() {
        assert!(Permission::Admin.includes(Permission::Read));
        assert!(Permission::Admin.includes(Permission::Write));
        assert!(Permission::Admin.includes(Permission::Admin));
        
        assert!(Permission::Write.includes(Permission::Read));
        assert!(Permission::Write.includes(Permission::Write));
        assert!(!Permission::Write.includes(Permission::Admin));
        
        assert!(Permission::Read.includes(Permission::Read));
        assert!(!Permission::Read.includes(Permission::Write));
        assert!(!Permission::Read.includes(Permission::Admin));
    }
    
    #[test]
    fn test_ace_matching() {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::Read);
        
        let ace = AccessControlEntry::allow("user1", "tag.*", permissions);
        
        assert!(ace.matches("user1", "tag.temperature"));
        assert!(ace.matches("user1", "tag.pressure"));
        assert!(!ace.matches("user2", "tag.temperature"));
        assert!(!ace.matches("user1", "other.tag"));
    }
    
    #[test]
    fn test_glob_matching() {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::Read);
        
        let ace = AccessControlEntry::allow("*", "plant.*.temperature", permissions);
        
        assert!(ace.matches("user1", "plant.room1.temperature"));
        assert!(ace.matches("user2", "plant.hall.temperature"));
        assert!(!ace.matches("user1", "plant.room1.pressure"));
        assert!(!ace.matches("user1", "factory.room1.temperature"));
    }
    
    #[test]
    fn test_permission_manager() {
        let manager = PermissionManager::new();
        
        // Add some ACEs
        let mut read_perms = HashSet::new();
        read_perms.insert(Permission::Read);
        
        let mut write_perms = HashSet::new();
        write_perms.insert(Permission::Write);
        
        manager.add_ace(AccessControlEntry::allow("user1", "sensor.*", read_perms.clone()));
        manager.add_ace(AccessControlEntry::allow("admin", "*", write_perms.clone()));
        manager.add_ace(AccessControlEntry::deny("user1", "sensor.secret", read_perms.clone()).with_priority(200));
        
        // Test permissions
        assert!(manager.check_permission("user1", "sensor.temperature", Permission::Read));
        assert!(!manager.check_permission("user1", "sensor.secret", Permission::Read));
        assert!(!manager.check_permission("user1", "sensor.temperature", Permission::Write));
        assert!(manager.check_permission("admin", "sensor.temperature", Permission::Write));
        assert!(!manager.check_permission("user2", "sensor.temperature", Permission::Read));
    }
    
    #[test]
    fn test_permission_priority() {
        let manager = PermissionManager::new();
        
        let mut read_perms = HashSet::new();
        read_perms.insert(Permission::Read);
        
        // Lower priority allow rule
        manager.add_ace(
            AccessControlEntry::allow("user1", "*", read_perms.clone())
                .with_priority(50)
        );
        
        // Higher priority deny rule
        manager.add_ace(
            AccessControlEntry::deny("user1", "secret.*", read_perms.clone())
                .with_priority(100)
        );
        
        // Deny should win due to higher priority
        assert!(!manager.check_permission("user1", "secret.data", Permission::Read));
        assert!(manager.check_permission("user1", "public.data", Permission::Read));
    }
    
    #[test]
    fn test_default_permissions() {
        let manager = PermissionManager::with_default_allow();
        
        // Should allow read by default
        assert!(manager.check_permission("unknown_user", "any.tag", Permission::Read));
        assert!(!manager.check_permission("unknown_user", "any.tag", Permission::Write));
    }
    
    #[test]
    fn test_get_permissions() {
        let manager = PermissionManager::new();
        
        let mut read_perms = HashSet::new();
        read_perms.insert(Permission::Read);
        
        let mut write_perms = HashSet::new();
        write_perms.insert(Permission::Write);
        
        manager.add_ace(AccessControlEntry::allow("user1", "data.*", read_perms));
        manager.add_ace(AccessControlEntry::allow("user1", "data.config", write_perms));
        
        let permissions = manager.get_permissions("user1", "data.config");
        assert!(permissions.contains(&Permission::Read));
        assert!(permissions.contains(&Permission::Write));
        assert!(!permissions.contains(&Permission::Admin));
    }
    
    #[test]
    fn test_import_export() {
        let manager = PermissionManager::new();
        
        let mut perms = HashSet::new();
        perms.insert(Permission::Read);
        perms.insert(Permission::Write);
        
        manager.add_ace(AccessControlEntry::allow("user1", "test.*", perms));
        
        let json = manager.export_aces().unwrap();
        assert!(json.contains("user1"));
        assert!(json.contains("test.*"));
        
        let manager2 = PermissionManager::new();
        manager2.import_aces(&json).unwrap();
        
        assert!(manager2.check_permission("user1", "test.data", Permission::Read));
    }
}