# Identity Domain

## Overview

The Identity Domain manages the core concept of identity within CIM, handling both person and organization identities, their relationships, and lifecycle management. It serves as the central authority for identity verification, authentication coordination, and relationship mapping across the system.

## Key Concepts

### Identity
- **Definition**: A unique, verifiable entity representing a person or organization
- **Properties**: ID, type (person/organization), status, verification level
- **Lifecycle**: Created → Verified → Active → Suspended → Archived

### Identity Relationship
- **Definition**: Connections between identities (employment, ownership, partnership)
- **Types**: Employee-of, Owner-of, Partner-with, Member-of, Reports-to
- **Properties**: Source identity, target identity, relationship type, validity period

### Identity Verification
- **Definition**: Process of confirming identity claims
- **Levels**: Unverified, Email-verified, Document-verified, Biometric-verified
- **Methods**: Email, phone, document upload, third-party verification

### Identity Claim
- **Definition**: Assertions about an identity's attributes or capabilities
- **Examples**: Email ownership, phone number, professional certification
- **Status**: Pending, Verified, Rejected, Expired

## Domain Events

### Commands
- `cmd.identity.create_identity` - Register new identity
- `cmd.identity.verify_claim` - Verify an identity claim
- `cmd.identity.establish_relationship` - Create identity relationship
- `cmd.identity.suspend_identity` - Temporarily disable identity
- `cmd.identity.merge_identities` - Combine duplicate identities

### Events
- `event.identity.identity_created` - New identity registered
- `event.identity.claim_verified` - Identity claim confirmed
- `event.identity.relationship_established` - New relationship created
- `event.identity.identity_suspended` - Identity deactivated
- `event.identity.identities_merged` - Duplicates combined

### Queries
- `query.identity.find_by_claim` - Search by email/phone/etc
- `query.identity.get_relationships` - Retrieve identity connections
- `query.identity.verify_status` - Check verification level
- `query.identity.get_identity_graph` - Full relationship network

## API Reference

### IdentityAggregate
```rust
pub struct IdentityAggregate {
    pub id: IdentityId,
    pub identity_type: IdentityType,
    pub status: IdentityStatus,
    pub verification_level: VerificationLevel,
    pub claims: HashMap<ClaimId, IdentityClaim>,
    pub relationships: HashMap<RelationshipId, IdentityRelationship>,
}
```

### Key Methods
- `create_identity()` - Initialize new identity
- `add_claim()` - Add identity claim
- `verify_claim()` - Confirm claim validity
- `establish_relationship()` - Create connection
- `check_verification_level()` - Get current verification status

## Integration Examples

### Creating a Person Identity
```rust
// Create new person identity
let cmd = CreateIdentity {
    identity_type: IdentityType::Person,
    initial_claims: vec![
        IdentityClaim::Email("john@example.com".to_string()),
        IdentityClaim::Phone("+1234567890".to_string()),
    ],
};

// Verify email claim
let verify = VerifyClaim {
    identity_id,
    claim_id,
    verification_method: VerificationMethod::EmailToken(token),
};

// Establish employment relationship
let relate = EstablishRelationship {
    source_identity: person_id,
    target_identity: org_id,
    relationship_type: RelationshipType::EmployeeOf,
    metadata: EmploymentMetadata {
        role: "Software Engineer".to_string(),
        department: "Engineering".to_string(),
    },
};
```

### Identity Verification Flow
```rust
// Start verification process
let start = StartVerification {
    identity_id,
    verification_type: VerificationType::DocumentUpload,
    required_documents: vec![
        DocumentType::GovernmentId,
        DocumentType::ProofOfAddress,
    ],
};

// Submit verification evidence
let submit = SubmitVerificationEvidence {
    identity_id,
    verification_id,
    evidence: VerificationEvidence::Document(document_data),
};

// Complete verification
let complete = CompleteVerification {
    identity_id,
    verification_id,
    result: VerificationResult::Approved,
    verified_by: verifier_id,
};
```

## Relationship Management

### Relationship Types
- **Employment**: Person ↔ Organization
- **Ownership**: Person → Organization/Asset
- **Partnership**: Organization ↔ Organization
- **Membership**: Person → Group/Organization
- **Hierarchy**: Person → Person (reporting structure)

### Relationship Validation
- Circular relationship prevention
- Type compatibility checking
- Temporal validity enforcement
- Conflict detection (e.g., multiple exclusive roles)

## Security and Privacy

### Access Control
- Identity owners have full access to their data
- Relationship visibility based on both parties' consent
- Claim verification requires appropriate authority
- Audit trail for all identity operations

### Data Protection
- PII encryption at rest
- Claim data minimization
- Right to erasure support
- Consent management for data sharing

## Use Cases

### User Authentication
- Single sign-on coordination
- Multi-factor authentication
- Session management
- Access token generation

### Organization Management
- Corporate structure modeling
- Employee directory
- Vendor relationships
- Partnership networks

### Compliance and Audit
- Identity verification for regulations
- Relationship disclosure requirements
- Access audit trails
- Data retention policies

## Performance Characteristics

- **Identity Capacity**: 1M+ identities
- **Relationship Queries**: <10ms for direct connections
- **Verification Processing**: <1s for automated checks
- **Graph Traversal**: <100ms for 3-degree relationships

## Best Practices

1. **Claim Minimization**: Only collect necessary identity claims
2. **Verification Levels**: Match verification to risk requirements
3. **Relationship Modeling**: Use specific relationship types
4. **Privacy by Design**: Default to minimal data exposure
5. **Audit Everything**: Comprehensive logging for compliance

## Related Domains

- **Person Domain**: Extended personal information
- **Organization Domain**: Detailed org structure
- **Policy Domain**: Access control policies
- **Agent Domain**: AI agent identities 