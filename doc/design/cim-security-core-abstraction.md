# CIM Security Core Abstraction Design

## Overview

`cim-security` is a core abstraction module that provides foundational security interfaces for the CIM architecture. It defines traits and types for cryptography, claims-based authentication, and secrets management without prescribing specific implementations.

## Scope

### What Belongs in cim-security (Core)
- Abstract cryptographic operation traits
- Claims and identity verification interfaces
- Secrets management abstractions
- Security context and policy traits
- Authentication and authorization interfaces
- Audit and compliance traits

### What Does NOT Belong (Goes to cim-keys)
- Specific implementations (GPG, SSH, TLS)
- Hardware token drivers (YubiKey, etc.)
- Certificate generation and PKI operations
- Key storage implementations
- Concrete cryptographic algorithms

## Module Structure

```
cim-security/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── crypto.rs          # Cryptographic operation traits
│   ├── claims.rs          # Claims-based security
│   ├── secrets.rs         # Secrets management
│   ├── auth.rs            # Authentication traits
│   ├── policy.rs          # Security policy abstractions
│   ├── audit.rs           # Audit trail traits
│   ├── context.rs         # Security context
│   ├── types.rs           # Common types
│   └── error.rs           # Error types
```

## Core Trait Definitions

### Cryptographic Operations

```rust
//! crypto.rs - Core cryptographic abstractions

use async_trait::async_trait;
use crate::{SecurityError, Result};

/// Trait for signing operations
#[async_trait]
pub trait Signer: Send + Sync {
    /// Sign data and return signature
    async fn sign(&self, data: &[u8]) -> Result<Signature>;
    
    /// Get the public key for verification
    async fn public_key(&self) -> Result<PublicKey>;
}

/// Trait for signature verification
#[async_trait]
pub trait Verifier: Send + Sync {
    /// Verify a signature against data
    async fn verify(&self, data: &[u8], signature: &Signature) -> Result<bool>;
}

/// Trait for encryption operations
#[async_trait]
pub trait Encryptor: Send + Sync {
    /// Encrypt data for recipients
    async fn encrypt(&self, data: &[u8], recipients: &[PublicKey]) -> Result<EncryptedData>;
}

/// Trait for decryption operations
#[async_trait]
pub trait Decryptor: Send + Sync {
    /// Decrypt data
    async fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>>;
}

/// Trait for key derivation
#[async_trait]
pub trait KeyDerivation: Send + Sync {
    /// Derive a key from input material
    async fn derive(&self, input: &[u8], context: &[u8]) -> Result<DerivedKey>;
}

/// Opaque types for crypto materials
#[derive(Clone, Debug)]
pub struct Signature(Vec<u8>);

#[derive(Clone, Debug)]
pub struct PublicKey(Vec<u8>);

#[derive(Clone, Debug)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub metadata: EncryptionMetadata,
}

#[derive(Clone, Debug)]
pub struct DerivedKey(Vec<u8>);
```

### Claims-Based Security

```rust
//! claims.rs - Claims and identity abstractions

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::Result;

/// A security claim
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claim {
    pub issuer: String,
    pub subject: String,
    pub claim_type: String,
    pub value: serde_json::Value,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A collection of claims forming an identity
#[derive(Clone, Debug, Default)]
pub struct ClaimsIdentity {
    pub claims: Vec<Claim>,
    pub authentication_type: Option<String>,
    pub name_claim_type: String,
    pub role_claim_type: String,
}

/// Trait for claim validation
#[async_trait]
pub trait ClaimValidator: Send + Sync {
    /// Validate a claim
    async fn validate(&self, claim: &Claim) -> Result<bool>;
    
    /// Validate a full identity
    async fn validate_identity(&self, identity: &ClaimsIdentity) -> Result<bool>;
}

/// Trait for claim issuance
#[async_trait]
pub trait ClaimIssuer: Send + Sync {
    /// Issue a new claim
    async fn issue_claim(
        &self,
        subject: &str,
        claim_type: &str,
        value: serde_json::Value,
        duration: Option<chrono::Duration>,
    ) -> Result<Claim>;
    
    /// Sign a claim
    async fn sign_claim(&self, claim: &Claim) -> Result<SignedClaim>;
}

/// A signed claim with proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedClaim {
    pub claim: Claim,
    pub signature: Vec<u8>,
    pub algorithm: String,
}
```

### Secrets Management

```rust
//! secrets.rs - Secrets management abstractions

use async_trait::async_trait;
use crate::Result;

/// A reference to a secret
#[derive(Clone, Debug)]
pub struct SecretRef {
    pub id: String,
    pub version: Option<String>,
}

/// Metadata about a secret
#[derive(Clone, Debug)]
pub struct SecretMetadata {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub tags: HashMap<String, String>,
}

/// Trait for secret storage
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Store a secret
    async fn put_secret(
        &self,
        id: &str,
        value: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<SecretRef>;
    
    /// Retrieve a secret
    async fn get_secret(&self, secret_ref: &SecretRef) -> Result<Vec<u8>>;
    
    /// Delete a secret
    async fn delete_secret(&self, secret_ref: &SecretRef) -> Result<()>;
    
    /// List secrets
    async fn list_secrets(&self, filter: Option<&str>) -> Result<Vec<SecretMetadata>>;
}

/// Trait for secret rotation
#[async_trait]
pub trait SecretRotation: Send + Sync {
    /// Rotate a secret
    async fn rotate_secret(&self, secret_ref: &SecretRef) -> Result<SecretRef>;
    
    /// Get rotation policy
    async fn rotation_policy(&self, secret_ref: &SecretRef) -> Result<RotationPolicy>;
}

#[derive(Clone, Debug)]
pub struct RotationPolicy {
    pub rotation_period: chrono::Duration,
    pub last_rotated: chrono::DateTime<chrono::Utc>,
    pub auto_rotate: bool,
}
```

### Authentication and Authorization

```rust
//! auth.rs - Authentication and authorization abstractions

use async_trait::async_trait;
use crate::{ClaimsIdentity, Result};

/// Authentication context
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub identity: ClaimsIdentity,
    pub authentication_method: String,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
    pub properties: HashMap<String, String>,
}

/// Trait for authentication
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate with credentials
    async fn authenticate(
        &self,
        credentials: &dyn Credentials,
    ) -> Result<AuthContext>;
    
    /// Validate an existing auth context
    async fn validate(&self, context: &AuthContext) -> Result<bool>;
    
    /// Refresh authentication
    async fn refresh(&self, context: &AuthContext) -> Result<AuthContext>;
}

/// Trait for authorization decisions
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check if a principal is authorized for an action
    async fn authorize(
        &self,
        context: &AuthContext,
        resource: &str,
        action: &str,
    ) -> Result<AuthDecision>;
}

/// Authorization decision
#[derive(Clone, Debug)]
pub struct AuthDecision {
    pub allowed: bool,
    pub reason: Option<String>,
    pub constraints: Vec<String>,
}

/// Trait for credentials
pub trait Credentials: Send + Sync {
    /// Get credential type
    fn credential_type(&self) -> &str;
    
    /// Convert to bytes for processing
    fn as_bytes(&self) -> Vec<u8>;
}
```

### Security Policy

```rust
//! policy.rs - Security policy abstractions

use async_trait::async_trait;
use crate::Result;

/// Security policy definition
#[derive(Clone, Debug)]
pub struct SecurityPolicy {
    pub id: String,
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub priority: i32,
}

/// A single policy rule
#[derive(Clone, Debug)]
pub struct PolicyRule {
    pub id: String,
    pub condition: PolicyCondition,
    pub effect: PolicyEffect,
}

/// Policy condition
#[derive(Clone, Debug)]
pub enum PolicyCondition {
    Always,
    Never,
    HasClaim(String),
    InRole(String),
    Custom(String),
}

/// Policy effect
#[derive(Clone, Debug)]
pub enum PolicyEffect {
    Allow,
    Deny,
    Audit,
}

/// Trait for policy evaluation
#[async_trait]
pub trait PolicyEvaluator: Send + Sync {
    /// Evaluate policies for a context
    async fn evaluate(
        &self,
        context: &AuthContext,
        policies: &[SecurityPolicy],
    ) -> Result<PolicyDecision>;
}

#[derive(Clone, Debug)]
pub struct PolicyDecision {
    pub effect: PolicyEffect,
    pub matched_rules: Vec<String>,
    pub audit_required: bool,
}
```

### Security Context

```rust
//! context.rs - Security context management

use crate::{AuthContext, ClaimsIdentity};
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
    pub fn new(correlation_id: String) -> Self {
        Self {
            inner: Arc::new(SecurityContextInner {
                auth_context: None,
                ambient_authority: ClaimsIdentity::default(),
                correlation_id,
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
}
```

## Integration with Other Core Modules

### With cim-domain
- Domain events can include security context
- Commands/queries validate against security policies
- Aggregates can enforce security constraints

### With cim-infrastructure
- Infrastructure implements concrete security providers
- NATS messages include security headers
- Transport-level security configuration

### With cim-subject
- Subject patterns can include security constraints
- Message routing based on security context
- Claims-based subject filtering

### With cim-ipld
- Content can be encrypted before storage
- CIDs can be signed for authenticity
- Access control for content retrieval

## Implementation Guidelines

1. **Keep It Abstract**: No concrete implementations in core
2. **Use Opaque Types**: Don't expose internal structures
3. **Async by Default**: All operations should be async
4. **Error Handling**: Rich error types for security failures
5. **Audit Trail**: All security operations should be auditable

## Migration from cim-keys

1. Move abstract traits to cim-security
2. Keep concrete implementations in cim-keys
3. Update cim-keys to implement cim-security traits
4. Update dependent modules to use cim-security

This design provides a clean, abstract security foundation that can be implemented by cim-keys or any other security provider while keeping the core minimal and focused on abstractions. 