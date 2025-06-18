# Implementation TODOs

## Summary

The CIM project has many designed but unimplemented functions across various domains. 

**UPDATE**: We've made significant progress on documentation and implementation:
- Reduced total warnings from 216+ to 83 (62% reduction)
- Eliminated ALL "missing documentation" warnings (previously 114 in cim-domain-nix alone)
- Added documentation to:
  - cim-domain-nix: commands, events, value objects, handlers, projections, services
  - Fixed several implementation issues in cim-keys and cim-domain-policy
  
**NEW IMPLEMENTATIONS (Round 1)**:
- Implemented DocumentAggregate and full command/event handlers for cim-domain-document
- Implemented WorkflowQueryHandler with projections for cim-domain-workflow
- Fixed Ollama provider to use all response fields (added logging)
- Implemented location repository usage in authentication event handler
- Implemented event publishing in location command handler
- Fixed various unused imports across multiple domains

**NEW IMPLEMENTATIONS (Round 2)**:
- Removed unused atomic counters in InMemoryGraphRepository
- Added role field logging in Ollama provider for debugging
- Fixed deprecated Bevy API usage (despawn_recursive → despawn)
- Implemented basic GPG key manager structure with all trait methods
- Fixed various unused imports in cim-domain-bevy, cim-keys, and cim-domain-organization
- Fixed module structure in cim-domain-organization

Remaining warnings are primarily:
- Unused variables/functions that need implementation
- Some deprecated Bevy APIs that still need updating
- Placeholder implementations that log warnings

Below is a comprehensive list of all unimplemented functionality that needs to be completed.

## Overview

As of the current state, we have 216+ warnings indicating unimplemented functions, unused variables, and missing documentation. These represent actual functionality that was designed but not yet implemented.

## Priority Areas

### 1. cim-keys (High Priority - Security Critical)
- **SSH Module**
  - ✅ `verify_ssh` - IMPLEMENTED
  - ❌ Encrypted key import/export with passphrase
  
- **TLS Module**  
  - ✅ `generate_self_signed` - IMPLEMENTED
  - ❌ `generate_csr` - Certificate Signing Request generation
  - ❌ `validate_certificate` - Certificate chain validation
  - ❌ Extract actual dates from X.509 certificates
  - ❌ Extract SANs from certificates
  - ❌ Extract key usage from certificates
  
- **PKI Module**
  - ❌ `create_root_ca` - Root Certificate Authority creation
  - ❌ `create_intermediate_ca` - Intermediate CA creation
  - ❌ `issue_certificate` - Certificate issuance from CSR
  - ❌ `create_certificate_chain` - Chain assembly
  - ❌ `verify_certificate_chain` - Chain verification
  - ❌ All KeyManager trait implementations
  - ❌ All Signer trait implementations
  - ❌ All Encryptor trait implementations
  
- **GPG Module**
  - ❌ Entire module implementation

### 2. cim-domain-nix (114 warnings - Core Domain)
- **Documentation**
  - ✅ Query struct field documentation - PARTIALLY DONE
  - ❌ Complete documentation for all public items
  
- **Functionality**
  - ❌ External nixpkgs search integration
  - ❌ Flake evaluation
  - ❌ Package building
  - ❌ Configuration activation
  - ❌ Generation management
  - ❌ Rollback functionality

### 3. cim-bridge (Provider Integration)
- **Ollama Provider**
  - ✅ Core implementation exists
  - ❌ Use all response fields (context, duration, etc.)
  - ❌ Implement generate endpoint (non-chat)
  - ❌ Model configuration options
  
- **Other Providers**
  - ❌ OpenAI provider
  - ❌ Anthropic provider
  - ❌ Local model providers

### 4. cim-domain-policy
- ✅ Repository usage in command handler - FIXED
- ❌ Additional command handlers
- ❌ Query handlers implementation
- ❌ Event handlers

### 5. cim-domain-workflow (12 warnings)
- ❌ Complete workflow execution engine
- ❌ State machine transitions
- ❌ Parallel execution support
- ❌ Conditional branching
- ❌ Error handling and recovery

### 6. cim-domain-graph
- ❌ Graph algorithms (shortest path, etc.)
- ❌ Layout algorithms
- ❌ Import/Export for various formats
- ❌ Graph analysis functions

### 7. cim-domain-location
- ❌ Geocoding integration
- ❌ Spatial queries
- ❌ Distance calculations
- ❌ Region management

## Implementation Strategy

1. **Phase 1: Security Critical (cim-keys)**
   - Complete PKI implementation for certificate management
   - Implement GPG support for signing
   - Add passphrase support for encrypted keys

2. **Phase 2: Core Domains**
   - Complete cim-domain-nix for system management
   - Implement workflow execution in cim-domain-workflow
   - Add graph algorithms to cim-domain-graph

3. **Phase 3: Integration**
   - Complete provider implementations in cim-bridge
   - Add remaining command/query handlers
   - Implement cross-domain integrations

4. **Phase 4: Polish**
   - Complete all documentation
   - Add comprehensive tests
   - Performance optimizations

## Testing Requirements

Each implemented function needs:
- Unit tests
- Integration tests where applicable
- Documentation with examples
- Error handling for edge cases

## Notes

- Many of these TODOs represent significant functionality that will require careful implementation
- Some may require additional dependencies or external service integration
- Prioritize based on user needs and security requirements 