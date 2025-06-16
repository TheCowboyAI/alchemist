# Identity Domain Rename and Authentication Features

## Overview

Renamed `cim-identity-context` to `cim-domain-identity` to follow consistent naming conventions and enhanced it with comprehensive authentication features for identifying and authenticating People and Organizations.

## Changes Made

### 1. Module Rename

- Renamed directory from `cim-identity-context` to `cim-domain-identity`
- Updated workspace `Cargo.toml` to reference new module name
- Updated module's own `Cargo.toml` package name
- Updated `.gitmodules` to point to new repository name
- Fixed all imports in test files

### 2. Authentication Value Objects Added

Added comprehensive authentication-related value objects in `domain/value_objects.rs`:

- **Credentials**: Username and password hash storage
- **AuthMethod**: Enum for authentication methods (Password, OAuth2, SAML, ApiKey, Certificate)
- **AuthStatus**: Current authentication state with login tracking and account locking
- **MfaSettings**: Multi-factor authentication configuration
- **MfaMethod**: Enum for MFA methods (TOTP, SMS, Email, App)
- **ApiKey**: Service authentication with permissions and expiration

### 3. Person Aggregate Enhancement

Enhanced the Person aggregate with authentication capabilities:

**New Fields:**
- `credentials: Option<Credentials>`
- `auth_status: AuthStatus`
- `mfa_settings: MfaSettings`

**New Commands:**
- `SetCredentials`: Set authentication credentials
- `Authenticate`: Authenticate with username/password
- `RecordFailedAuth`: Track failed authentication attempts
- `LockAccount`: Lock account after too many failures
- `UnlockAccount`: Unlock a locked account
- `EnableMfa`: Enable multi-factor authentication
- `DisableMfa`: Disable multi-factor authentication
- `UpdateLastLogin`: Update last login timestamp

**New Events:**
- `CredentialsSet`: Credentials were configured
- `AuthenticationSucceeded`: Successful authentication
- `AuthenticationFailed`: Failed authentication attempt
- `AccountLocked`: Account was locked
- `AccountUnlocked`: Account was unlocked
- `MfaEnabled`: MFA was enabled
- `MfaDisabled`: MFA was disabled

### 4. Organization Enhancement

Added API key support to Organization aggregate:
- `api_keys: Vec<ApiKey>` field for service authentication

### 5. Documentation Updates

Updated module documentation to reflect:
- New focus on identity and authentication
- Authentication features for both Person and Organization
- Security considerations (password hashing, account locking, audit trails)

## Architecture

The module now provides a complete identity and authentication domain following DDD principles:

```
cim-domain-identity/
├── src/
│   ├── domain/
│   │   ├── person/          # Person aggregate with auth
│   │   ├── organization/    # Organization with API keys
│   │   └── value_objects.rs # Auth value objects
│   ├── application/         # Command/query handlers
│   ├── infrastructure/      # Repository implementations
│   ├── ports/              # Auth interfaces
│   └── conceptual/         # Identity projections
└── tests/
```

## Testing

All existing tests pass with the rename. Authentication features are ready for testing with the new commands and events.

## Next Steps

1. Implement authentication command handlers in the application layer
2. Add password hashing utilities (bcrypt/argon2)
3. Implement JWT token generation for authenticated sessions
4. Add rate limiting for authentication attempts
5. Create integration tests for authentication flows
