# CIM Security

Core security abstractions for the Composable Information Machine (CIM).

## Overview

`cim-security` is a foundational module that provides abstract interfaces for all security concerns in CIM:

- **Cryptographic Operations**: Signing, verification, encryption, decryption, key derivation
- **Claims-Based Authentication**: Identity management through claims and validation
- **Secrets Management**: Secure storage, retrieval, and rotation of secrets
- **Authentication & Authorization**: Flexible auth patterns with policy-based access control
- **Security Policies**: Declarative security rules with conditional evaluation
- **Audit Trails**: Comprehensive logging of security-relevant events
- **Security Context**: Thread-safe context propagation for security information

## Design Principles

1. **Abstract Over Concrete**: This crate contains only traits and interfaces, no implementations
2. **Async by Default**: All operations are async for scalability
3. **Composable**: Security features can be mixed and matched as needed
4. **Type-Safe**: Leverages Rust's type system for security guarantees

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cim-security = { path = "../cim-security" }
```

### Example: Authentication

```rust
use cim_security::prelude::*;

async fn authenticate_user(
    authenticator: &impl Authenticator,
    username: &str,
    password: &str,
) -> Result<AuthContext> {
    let credentials = PasswordCredentials {
        username: username.to_string(),
        password: password.to_string(),
    };
    
    authenticator.authenticate(&credentials).await
}
```

### Example: Claims-Based Identity

```rust
use cim_security::claims::{Claim, ClaimsIdentity, claim_types};
use serde_json::json;

let mut identity = ClaimsIdentity::new();

identity.add_claim(Claim::new(
    "system",
    "user123",
    claim_types::NAME,
    json!("John Doe"),
));

identity.add_claim(Claim::new(
    "system",
    "user123",
    claim_types::ROLE,
    json!("admin"),
));

assert_eq!(identity.name(), Some("John Doe".to_string()));
assert!(identity.has_role("admin"));
```

### Example: Policy-Based Authorization

```rust
use cim_security::policy::*;

let policy = SecurityPolicy::new("read-policy", "Document Read Policy")
    .add_rule(
        PolicyRule::new(
            "allow-readers",
            PolicyCondition::InRole("reader".to_string()),
            PolicyEffect::Allow,
        )
        .with_resource("documents/*")
        .with_action("read")
    );

let evaluator = DefaultPolicyEvaluator;
let decision = evaluator.evaluate(&auth_context, &[policy], "documents/123", "read").await?;

if decision.effect == PolicyEffect::Allow {
    // Access granted
}
```

## Module Structure

- **`error`**: Error types and result aliases
- **`types`**: Common types (Signature, PublicKey, etc.)
- **`crypto`**: Cryptographic operation traits
- **`claims`**: Claims and identity management
- **`secrets`**: Secret storage and rotation
- **`auth`**: Authentication and authorization
- **`policy`**: Security policy evaluation
- **`audit`**: Audit logging traits
- **`context`**: Security context management

## Implementation

This crate provides only abstractions. For concrete implementations, see:

- `cim-keys`: Cryptographic implementations (GPG, SSH, TLS, YubiKey)
- `cim-infrastructure`: NATS-based security implementations

## License

MIT OR Apache-2.0 