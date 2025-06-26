//! Security context management

use crate::{auth::AuthContext, claims::ClaimsIdentity};
use std::sync::Arc;

/// Thread-safe security context
#[derive(Clone)]
pub struct SecurityContext {
    inner: Arc<SecurityContextInner>,
}

struct SecurityContextInner {
    auth_context: Option<AuthContext>,
    ambient_authority: ClaimsIdentity,
    correlation_id: String,
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(correlation_id: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(SecurityContextInner {
                auth_context: None,
                ambient_authority: ClaimsIdentity::default(),
                correlation_id: correlation_id.into(),
            }),
        }
    }

    /// Create a security context with authentication
    pub fn with_auth(correlation_id: impl Into<String>, auth_context: AuthContext) -> Self {
        Self {
            inner: Arc::new(SecurityContextInner {
                auth_context: Some(auth_context),
                ambient_authority: ClaimsIdentity::default(),
                correlation_id: correlation_id.into(),
            }),
        }
    }

    /// Get current authentication context
    pub fn auth_context(&self) -> Option<&AuthContext> {
        self.inner.auth_context.as_ref()
    }

    /// Get correlation ID for tracing
    pub fn correlation_id(&self) -> &str {
        &self.inner.correlation_id
    }

    /// Get the ambient authority
    pub fn ambient_authority(&self) -> &ClaimsIdentity {
        &self.inner.ambient_authority
    }

    /// Check if the context is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.inner.auth_context.is_some()
    }

    /// Get the principal name
    pub fn principal_name(&self) -> Option<String> {
        self.inner
            .auth_context
            .as_ref()
            .and_then(|ctx| ctx.identity.name())
    }
}

/// Builder for security context
pub struct SecurityContextBuilder {
    auth_context: Option<AuthContext>,
    ambient_authority: Option<ClaimsIdentity>,
    correlation_id: Option<String>,
}

impl SecurityContextBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            auth_context: None,
            ambient_authority: None,
            correlation_id: None,
        }
    }

    /// Set authentication context
    pub fn with_auth_context(mut self, auth_context: AuthContext) -> Self {
        self.auth_context = Some(auth_context);
        self
    }

    /// Set ambient authority
    pub fn with_ambient_authority(mut self, authority: ClaimsIdentity) -> Self {
        self.ambient_authority = Some(authority);
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Build the security context
    pub fn build(self) -> SecurityContext {
        let correlation_id = self
            .correlation_id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        SecurityContext {
            inner: Arc::new(SecurityContextInner {
                auth_context: self.auth_context,
                ambient_authority: self.ambient_authority.unwrap_or_default(),
                correlation_id,
            }),
        }
    }
}

impl Default for SecurityContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-local security context storage
thread_local! {
    static CURRENT_CONTEXT: std::cell::RefCell<Option<SecurityContext>> = std::cell::RefCell::new(None);
}

/// Get the current security context
pub fn current() -> Option<SecurityContext> {
    CURRENT_CONTEXT.with(|ctx| ctx.borrow().clone())
}

/// Set the current security context
pub fn set_current(context: SecurityContext) {
    CURRENT_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(context);
    });
}

/// Clear the current security context
pub fn clear_current() {
    CURRENT_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

/// Run a function with a specific security context
pub fn with_context<F, R>(context: SecurityContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    let previous = current();
    set_current(context);
    let result = f();
    match previous {
        Some(ctx) => set_current(ctx),
        None => clear_current(),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::{Claim, claim_types};
    use serde_json::json;

    #[test]
    fn test_security_context_creation() {
        let context = SecurityContext::new("test-correlation");
        assert_eq!(context.correlation_id(), "test-correlation");
        assert!(!context.is_authenticated());
        assert!(context.principal_name().is_none());
    }

    #[test]
    fn test_security_context_with_auth() {
        let mut identity = ClaimsIdentity::new();
        identity.add_claim(Claim::new(
            "test-issuer",
            "user123",
            claim_types::NAME,
            json!("John Doe"),
        ));

        let auth_context = AuthContext::new(identity, "password");
        let context = SecurityContext::with_auth("test-correlation", auth_context);

        assert!(context.is_authenticated());
        assert_eq!(context.principal_name(), Some("John Doe".to_string()));
    }

    #[test]
    fn test_security_context_builder() {
        let context = SecurityContextBuilder::new()
            .with_correlation_id("custom-id")
            .build();

        assert_eq!(context.correlation_id(), "custom-id");
    }

    #[test]
    fn test_thread_local_context() {
        let context = SecurityContext::new("test-correlation");

        assert!(current().is_none());

        set_current(context.clone());
        assert!(current().is_some());
        assert_eq!(current().unwrap().correlation_id(), "test-correlation");

        clear_current();
        assert!(current().is_none());
    }

    #[test]
    fn test_with_context() {
        let context = SecurityContext::new("test-correlation");

        let result = with_context(context, || {
            let ctx = current().expect("Context should be set");
            ctx.correlation_id().to_string()
        });

        assert_eq!(result, "test-correlation");
        assert!(current().is_none());
    }
}
