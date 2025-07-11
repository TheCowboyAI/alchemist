//! User context management for rate limiting and authentication

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{debug, info};

/// User information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub tier: UserTier,
    pub api_key: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_active: chrono::DateTime<chrono::Utc>,
}

/// User tier for rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserTier {
    Free,
    Pro,
    Enterprise,
    Admin,
}

impl UserTier {
    /// Get rate limit multiplier for tier
    pub fn rate_limit_multiplier(&self) -> f64 {
        match self {
            UserTier::Free => 1.0,
            UserTier::Pro => 10.0,
            UserTier::Enterprise => 100.0,
            UserTier::Admin => 1000.0,
        }
    }
    
    /// Get cache quota for tier (MB)
    pub fn cache_quota_mb(&self) -> usize {
        match self {
            UserTier::Free => 100,
            UserTier::Pro => 1000,
            UserTier::Enterprise => 10000,
            UserTier::Admin => 100000,
        }
    }
}

/// Thread-local user context
thread_local! {
    static CURRENT_USER: std::cell::RefCell<Option<UserInfo>> = std::cell::RefCell::new(None);
}

/// Global user registry
pub struct UserRegistry {
    users: Arc<RwLock<HashMap<String, UserInfo>>>,
}

impl UserRegistry {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a new user
    pub async fn register_user(&self, user: UserInfo) {
        let mut users = self.users.write().await;
        info!("Registering user: {} ({})", user.id, user.name);
        users.insert(user.id.clone(), user);
    }
    
    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Option<UserInfo> {
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }
    
    /// Get user by API key
    pub async fn get_user_by_api_key(&self, api_key: &str) -> Option<UserInfo> {
        let users = self.users.read().await;
        users.values()
            .find(|u| u.api_key.as_deref() == Some(api_key))
            .cloned()
    }
    
    /// Update user last active time
    pub async fn update_last_active(&self, user_id: &str) {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.last_active = chrono::Utc::now();
        }
    }
    
    /// Load users from config file
    pub async fn load_from_file(&self, path: &str) -> anyhow::Result<()> {
        let content = tokio::fs::read_to_string(path).await?;
        let users: Vec<UserInfo> = serde_json::from_str(&content)?;
        
        for user in users {
            self.register_user(user).await;
        }
        
        Ok(())
    }
    
    /// Save users to config file
    pub async fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let users = self.users.read().await;
        let users_vec: Vec<&UserInfo> = users.values().collect();
        let json = serde_json::to_string_pretty(&users_vec)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }
}

/// Global user registry instance
static USER_REGISTRY: once_cell::sync::OnceCell<UserRegistry> = once_cell::sync::OnceCell::new();

/// Get the global user registry
pub fn global_registry() -> &'static UserRegistry {
    USER_REGISTRY.get_or_init(|| UserRegistry::new())
}

/// User context for request handling
#[derive(Clone)]
pub struct UserContext {
    pub user: Option<UserInfo>,
}

impl UserContext {
    /// Create context for anonymous user
    pub fn anonymous() -> Self {
        Self { user: None }
    }
    
    /// Create context for authenticated user
    pub fn authenticated(user: UserInfo) -> Self {
        Self { user: Some(user) }
    }
    
    /// Get user ID for rate limiting
    pub fn user_id(&self) -> String {
        self.user
            .as_ref()
            .map(|u| u.id.clone())
            .unwrap_or_else(|| "anonymous".to_string())
    }
    
    /// Get user tier
    pub fn tier(&self) -> UserTier {
        self.user
            .as_ref()
            .map(|u| u.tier)
            .unwrap_or(UserTier::Free)
    }
    
    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }
    
    /// Set as current thread-local context
    pub fn set_current(self) {
        CURRENT_USER.with(|u| {
            *u.borrow_mut() = self.user;
        });
    }
    
    /// Get current thread-local context
    pub fn current() -> Self {
        CURRENT_USER.with(|u| {
            Self {
                user: u.borrow().clone(),
            }
        })
    }
}

/// Middleware to extract user context from request
pub async fn extract_user_context(api_key: Option<&str>) -> UserContext {
    if let Some(key) = api_key {
        if let Some(user) = global_registry().get_user_by_api_key(key).await {
            debug!("Authenticated user: {} ({})", user.id, user.name);
            global_registry().update_last_active(&user.id).await;
            return UserContext::authenticated(user);
        }
    }
    
    debug!("Anonymous user context");
    UserContext::anonymous()
}

/// Create default users for testing
pub async fn create_default_users() {
    let registry = global_registry();
    
    // Admin user
    registry.register_user(UserInfo {
        id: "admin".to_string(),
        name: "Admin User".to_string(),
        email: Some("admin@alchemist.local".to_string()),
        tier: UserTier::Admin,
        api_key: Some("admin-api-key".to_string()),
        created_at: chrono::Utc::now(),
        last_active: chrono::Utc::now(),
    }).await;
    
    // Test users
    registry.register_user(UserInfo {
        id: "test-free".to_string(),
        name: "Free User".to_string(),
        email: Some("free@test.local".to_string()),
        tier: UserTier::Free,
        api_key: Some("free-api-key".to_string()),
        created_at: chrono::Utc::now(),
        last_active: chrono::Utc::now(),
    }).await;
    
    registry.register_user(UserInfo {
        id: "test-pro".to_string(),
        name: "Pro User".to_string(),
        email: Some("pro@test.local".to_string()),
        tier: UserTier::Pro,
        api_key: Some("pro-api-key".to_string()),
        created_at: chrono::Utc::now(),
        last_active: chrono::Utc::now(),
    }).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_registry() {
        let registry = UserRegistry::new();
        
        let user = UserInfo {
            id: "test123".to_string(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            tier: UserTier::Pro,
            api_key: Some("test-key".to_string()),
            created_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        };
        
        registry.register_user(user.clone()).await;
        
        // Get by ID
        let retrieved = registry.get_user("test123").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test User");
        
        // Get by API key
        let by_key = registry.get_user_by_api_key("test-key").await;
        assert!(by_key.is_some());
        assert_eq!(by_key.unwrap().id, "test123");
    }
    
    #[tokio::test]
    async fn test_user_context() {
        let user = UserInfo {
            id: "context-test".to_string(),
            name: "Context Test".to_string(),
            email: None,
            tier: UserTier::Enterprise,
            api_key: None,
            created_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        };
        
        let context = UserContext::authenticated(user);
        assert!(context.is_authenticated());
        assert_eq!(context.user_id(), "context-test");
        assert_eq!(context.tier(), UserTier::Enterprise);
        
        // Test anonymous
        let anon = UserContext::anonymous();
        assert!(!anon.is_authenticated());
        assert_eq!(anon.user_id(), "anonymous");
        assert_eq!(anon.tier(), UserTier::Free);
    }
    
    #[test]
    fn test_tier_limits() {
        assert_eq!(UserTier::Free.rate_limit_multiplier(), 1.0);
        assert_eq!(UserTier::Pro.rate_limit_multiplier(), 10.0);
        assert_eq!(UserTier::Enterprise.rate_limit_multiplier(), 100.0);
        
        assert_eq!(UserTier::Free.cache_quota_mb(), 100);
        assert_eq!(UserTier::Pro.cache_quota_mb(), 1000);
    }
}