# Core Entity Projections and Injections Plan

## Overview

This plan defines how core business entities (People, Organizations, Operators, Agents, Locations, Networks) are projected to and ingested from external systems through our domain modules.

## Core Entity Types

### 1. People/Person
**Domain Concept**: Individual human entities with identity, roles, and relationships

**Projections TO External Systems**:
- **LDAP/AD**: Sync person records as directory entries
- **Keycloak**: Create user identities for authentication
- **Graph Database**: Store as Person nodes with relationships
- **Search Index**: Index for people search and discovery

**Injections FROM External Systems**:
- **HR Systems**: Import employee records
- **CRM**: Import customer/contact information
- **Social Networks**: Import profile data (with consent)
- **Email Systems**: Extract person entities from communications

### 2. Organization/OrganizationalUnit
**Domain Concept**: Legal entities and their hierarchical structures

**Projections TO External Systems**:
- **LDAP/AD**: Organizational units in directory tree
- **ERP Systems**: Company and department structures
- **Graph Database**: Organization nodes with hierarchy edges
- **Document Systems**: Folder structures matching org hierarchy

**Injections FROM External Systems**:
- **Business Registries**: Import company information
- **ERP/CRM**: Import organizational structures
- **Partner APIs**: Import partner organization data
- **Public Data Sources**: Import org charts and structures

### 3. Operator/Account/User
**Domain Concept**: System access entities with authentication and authorization

**Projections TO External Systems**:
- **OAuth2/OIDC**: User accounts for SSO
- **RBAC Systems**: Role assignments
- **Audit Logs**: Access tracking
- **Session Stores**: Active user sessions

**Injections FROM External Systems**:
- **Identity Providers**: Import user accounts
- **Legacy Systems**: Migrate user credentials
- **Partner Systems**: Federated identity import
- **Access Logs**: Import historical access patterns

### 4. Agent
**Domain Concept**: Autonomous entities (AI or automated) that act on behalf of users

**Projections TO External Systems**:
- **API Gateways**: Agent API keys and tokens
- **Workflow Engines**: Agent task assignments
- **Monitoring Systems**: Agent activity metrics
- **Graph Database**: Agent nodes with capabilities

**Injections FROM External Systems**:
- **AI Platforms**: Import agent definitions
- **Automation Tools**: Import bot configurations
- **Service Registries**: Import service agents
- **IoT Platforms**: Import device agents

### 5. Location/GeoCoordinates/Address
**Domain Concept**: Physical and logical locations with spatial properties

**Projections TO External Systems**:
- **GIS Systems**: Spatial data layers
- **Mapping Services**: Location markers
- **Logistics Systems**: Delivery addresses
- **Graph Database**: Location nodes with proximity edges

**Injections FROM External Systems**:
- **Geocoding Services**: Address to coordinate conversion
- **GPS/IoT**: Real-time location updates
- **Mapping APIs**: POI (Point of Interest) data
- **Property Systems**: Building/facility information

### 6. Network/Relationship
**Domain Concept**: Connections and relationships between entities

**Projections TO External Systems**:
- **Graph Databases**: Relationship edges with properties
- **Analytics Platforms**: Network structure data
- **Visualization Tools**: Network diagrams
- **ML Platforms**: Training data for link prediction

**Injections FROM External Systems**:
- **Social Networks**: Connection graphs
- **Communication Systems**: Interaction patterns
- **Transaction Systems**: Business relationships
- **Collaboration Tools**: Team structures

## Implementation Architecture

### Event Flow for Entity Synchronization

```
External System → Ingest Handler → Domain Event → Entity Aggregate → Projection → External System
```

### Example: Person Entity Flow

```rust
// 1. Injection from LDAP
pub struct LDAPPersonIngester {
    ldap_client: LdapClient,
    identity_module: IdentityManagement,
}

impl LDAPPersonIngester {
    async fn sync_persons(&self) -> Result<()> {
        let ldap_entries = self.ldap_client.search("ou=people,dc=company,dc=com").await?;

        for entry in ldap_entries {
            let person = self.translate_ldap_to_person(entry)?;

            // Publish domain event
            self.publish_event(PersonImported {
                person_id: PersonId::new(),
                source: "LDAP",
                data: person,
            }).await?;
        }
        Ok(())
    }
}

// 2. Domain Processing
impl PersonAggregate {
    fn handle_import(&mut self, event: PersonImported) -> Result<Vec<DomainEvent>> {
        // Validate and enrich
        let enriched = self.enrich_person_data(event.data)?;

        // Generate events
        Ok(vec![
            PersonCreated { ... },
            PersonEnriched { ... },
        ])
    }
}

// 3. Projection to Graph Database
pub struct GraphPersonProjection {
    graph_client: GraphPersistence,
}

impl EventHandler for GraphPersonProjection {
    async fn handle_event(&self, event: DomainEvent) -> Result<()> {
        match event {
            DomainEvent::PersonCreated(e) => {
                self.graph_client.create_node(
                    NodeType::Person,
                    e.person_id,
                    e.attributes,
                ).await?;
            }
            DomainEvent::RelationshipEstablished(e) => {
                self.graph_client.create_edge(
                    e.source_id,
                    e.target_id,
                    e.relationship_type,
                ).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Bidirectional Sync Patterns

### 1. Master-Slave Pattern
- CIM is master: Changes flow outward only
- External is master: Changes flow inward only
- Use for: Authoritative data sources

### 2. Peer-to-Peer Pattern
- Changes flow both directions
- Conflict resolution required
- Use for: Collaborative systems

### 3. Event Sourcing Pattern
- All changes as events
- Eventual consistency
- Use for: Audit trails, history

## Configuration Schema

```yaml
entity_sync:
  identity_management:
    ldap:
      enabled: true
      sync_interval: 300s
      base_dn: "ou=people,dc=company,dc=com"
      attributes_map:
        cn: "full_name"
        mail: "email"
        telephoneNumber: "phone"

  access_control:
    oauth2:
      enabled: true
      provider: "keycloak"
      realm: "company"
      sync_roles: true

  location_intelligence:
    geocoding:
      provider: "mapbox"
      cache_results: true
      batch_size: 100

  network_analysis:
    graph_db:
      enabled: true
      relationship_types:
        - "reports_to"
        - "collaborates_with"
        - "located_at"
```

## Privacy and Compliance

### Data Protection
- PII handling according to GDPR/CCPA
- Encryption at rest and in transit
- Access control and audit logging
- Right to erasure implementation

### Consent Management
- Track consent for data usage
- Honor opt-out requests
- Implement data minimization
- Regular compliance audits

## Testing Strategy

### Unit Tests
- Entity translation logic
- Validation rules
- Conflict resolution

### Integration Tests
- End-to-end sync flows
- Error handling
- Performance under load

### Compliance Tests
- Privacy rule enforcement
- Data retention policies
- Access control verification

## Monitoring and Observability

### Metrics
- Sync success/failure rates
- Entity counts by type
- Sync latency
- Conflict resolution stats

### Alerts
- Sync failures
- Data quality issues
- Performance degradation
- Compliance violations

## Next Steps

1. Implement IdentityManagement module with LDAP connector
2. Create Person and Organization aggregates
3. Build projection handlers for graph database
4. Add privacy controls and consent tracking
5. Create comprehensive test suite
6. Deploy monitoring and alerting
