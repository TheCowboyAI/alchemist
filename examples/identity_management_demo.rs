//! Identity Management Demo
//! 
//! This example demonstrates how to use the Identity Domain for:
//! - Creating and managing identities
//! - Establishing relationships
//! - Running verification workflows
//! - Cross-domain integration

use bevy_ecs::prelude::*;
use cim_domain_identity::*;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Demo application showing identity management
fn main() {
    // Create a Bevy app with identity domain
    let mut app = App::new();
    
    // Add identity domain plugin (hypothetical)
    app.add_plugins(MinimalPlugins)
        .add_event::<CreateIdentityCommand>()
        .add_event::<UpdateIdentityCommand>()
        .add_event::<EstablishRelationshipCommand>()
        .add_event::<StartVerificationCommand>()
        .add_event::<IdentityCreated>()
        .add_event::<RelationshipEstablished>()
        .add_event::<VerificationCompleted>()
        .add_systems(Update, (
            create_identity_system,
            update_identity_system,
            establish_relationship_system,
            start_verification_system,
        ))
        .add_systems(PostUpdate, (
            print_identity_events,
            print_relationship_events,
        ));

    // Run demo scenario
    run_demo_scenario(&mut app);
}

/// Run a complete identity management scenario
fn run_demo_scenario(app: &mut App) {
    println!("=== Identity Management Demo ===\n");

    // Scenario 1: Create person and organization identities
    println!("1. Creating identities...");
    let person_id = create_person_identity(app);
    let org_id = create_organization_identity(app);
    app.update();

    // Scenario 2: Establish employment relationship
    println!("\n2. Establishing employment relationship...");
    establish_employment(app, person_id, org_id);
    app.update();

    // Scenario 3: Start verification process
    println!("\n3. Starting verification workflow...");
    start_email_verification(app, person_id);
    app.update();

    // Scenario 4: Update verification status
    println!("\n4. Completing verification...");
    complete_verification(app, person_id);
    app.update();

    // Scenario 5: Query and display results
    println!("\n5. Querying identity state...");
    display_identity_state(&mut app.world, person_id);
    display_relationships(&mut app.world, person_id);
}

/// Create a person identity
fn create_person_identity(app: &mut App) -> Uuid {
    let person_id = Uuid::new_v4();
    let external_ref = Uuid::new_v4(); // Simulated person record ID

    app.world.send_event(CreateIdentityCommand {
        identity_type: IdentityType::Person,
        external_reference: external_ref.to_string(),
        initial_verification_level: VerificationLevel::Unverified,
        claims: vec![
            IdentityClaim {
                claim_id: Uuid::new_v4(),
                identity_id: person_id, // Will be replaced by system
                claim_type: ClaimType::Email,
                value: "john.doe@example.com".to_string(),
                verified: false,
                issued_at: SystemTime::now(),
                verified_at: None,
            },
            IdentityClaim {
                claim_id: Uuid::new_v4(),
                identity_id: person_id,
                claim_type: ClaimType::Phone,
                value: "+1-555-0123".to_string(),
                verified: false,
                issued_at: SystemTime::now(),
                verified_at: None,
            },
        ],
    });

    person_id
}

/// Create an organization identity
fn create_organization_identity(app: &mut App) -> Uuid {
    let org_id = Uuid::new_v4();
    let external_ref = Uuid::new_v4(); // Simulated org record ID

    app.world.send_event(CreateIdentityCommand {
        identity_type: IdentityType::Organization,
        external_reference: external_ref.to_string(),
        initial_verification_level: VerificationLevel::Enhanced, // Pre-verified org
        claims: vec![
            IdentityClaim {
                claim_id: Uuid::new_v4(),
                identity_id: org_id,
                claim_type: ClaimType::Domain,
                value: "example.com".to_string(),
                verified: true,
                issued_at: SystemTime::now(),
                verified_at: Some(SystemTime::now()),
            },
        ],
    });

    org_id
}

/// Establish employment relationship
fn establish_employment(app: &mut App, employee_id: Uuid, company_id: Uuid) {
    app.world.send_event(EstablishRelationshipCommand {
        source_identity: employee_id,
        target_identity: company_id,
        relationship_type: RelationshipType::EmployedBy,
        metadata: HashMap::from([
            ("department".to_string(), "Engineering".to_string()),
            ("role".to_string(), "Senior Developer".to_string()),
            ("start_date".to_string(), "2024-01-15".to_string()),
            ("employment_type".to_string(), "Full-time".to_string()),
        ]),
        expires_at: None, // Permanent employment
    });
}

/// Start email verification
fn start_email_verification(app: &mut App, identity_id: Uuid) {
    let admin_id = Uuid::new_v4(); // System admin

    app.world.send_event(StartVerificationCommand {
        identity_id,
        verification_method: VerificationMethod::Email,
        initiated_by: admin_id,
    });
}

/// Complete verification (simulated)
fn complete_verification(app: &mut App, identity_id: Uuid) {
    app.world.send_event(UpdateIdentityCommand {
        identity_id,
        verification_level: Some(VerificationLevel::Basic),
        add_claims: vec![],
        remove_claim_ids: vec![],
    });
}

/// Display identity state
fn display_identity_state(world: &mut World, identity_id: Uuid) {
    if let Some(identity) = find_identity_by_id(world, identity_id) {
        println!("Identity Details:");
        println!("  ID: {}", identity.id);
        println!("  Type: {:?}", identity.identity_type);
        println!("  External Reference: {}", identity.external_reference);
        println!("  Verification Level: {:?}", identity.verification_level);
        println!("  Created: {:?}", identity.created_at);
        
        // Display claims
        let claims: Vec<_> = world.query::<&IdentityClaim>()
            .iter(world)
            .filter(|claim| claim.identity_id == identity_id)
            .collect();
            
        println!("  Claims ({}):", claims.len());
        for claim in claims {
            println!("    - {} ({}): {} [{}]",
                match claim.claim_type {
                    ClaimType::Email => "Email",
                    ClaimType::Phone => "Phone",
                    ClaimType::Domain => "Domain",
                    _ => "Other",
                },
                claim.claim_id,
                claim.value,
                if claim.verified { "Verified" } else { "Unverified" }
            );
        }
    } else {
        println!("Identity {} not found", identity_id);
    }
}

/// Display relationships
fn display_relationships(world: &mut World, identity_id: Uuid) {
    let relationships = find_relationships_by_identity(world, identity_id);
    
    println!("\nRelationships ({}):", relationships.len());
    for rel in relationships {
        let direction = if rel.source_identity == identity_id {
            "outgoing"
        } else {
            "incoming"
        };
        
        println!("  - {} ({}):", rel.relationship_type.to_string(), direction);
        println!("    ID: {}", rel.id);
        println!("    Source: {}", rel.source_identity);
        println!("    Target: {}", rel.target_identity);
        
        if !rel.metadata.is_empty() {
            println!("    Metadata:");
            for (key, value) in &rel.metadata {
                println!("      {}: {}", key, value);
            }
        }
    }
}

/// System to print identity events
fn print_identity_events(mut events: EventReader<IdentityCreated>) {
    for event in events.read() {
        println!("  [EVENT] Identity created: {} (Type: {:?})",
            event.identity_id,
            event.identity_type
        );
    }
}

/// System to print relationship events
fn print_relationship_events(mut events: EventReader<RelationshipEstablished>) {
    for event in events.read() {
        println!("  [EVENT] Relationship established: {} -> {} (Type: {:?})",
            event.source_identity,
            event.target_identity,
            event.relationship_type
        );
    }
}

/// Example: Complex verification workflow
fn complex_verification_workflow(app: &mut App, identity_id: Uuid) {
    println!("\n=== Complex Verification Workflow ===");
    
    // Step 1: Email verification
    println!("Step 1: Email verification");
    app.world.send_event(StartVerificationCommand {
        identity_id,
        verification_method: VerificationMethod::Email,
        initiated_by: Uuid::new_v4(),
    });
    app.update();
    
    // Step 2: Phone verification
    println!("Step 2: Phone verification");
    app.world.send_event(StartVerificationCommand {
        identity_id,
        verification_method: VerificationMethod::Phone,
        initiated_by: Uuid::new_v4(),
    });
    app.update();
    
    // Step 3: Document verification
    println!("Step 3: Document verification");
    app.world.send_event(StartVerificationCommand {
        identity_id,
        verification_method: VerificationMethod::Document,
        initiated_by: Uuid::new_v4(),
    });
    app.update();
    
    // Update to full verification
    app.world.send_event(UpdateIdentityCommand {
        identity_id,
        verification_level: Some(VerificationLevel::Full),
        add_claims: vec![],
        remove_claim_ids: vec![],
    });
    app.update();
}

/// Example: Building an organization hierarchy
fn build_organization_hierarchy(app: &mut App) {
    println!("\n=== Building Organization Hierarchy ===");
    
    // Create organization identities
    let parent_org = create_identity(app, IdentityType::Organization, "ACME Corp");
    let subsidiary1 = create_identity(app, IdentityType::Organization, "ACME Tech");
    let subsidiary2 = create_identity(app, IdentityType::Organization, "ACME Finance");
    let department = create_identity(app, IdentityType::Organization, "Tech R&D");
    
    app.update();
    
    // Establish parent-subsidiary relationships
    establish_relationship(app, subsidiary1, parent_org, RelationshipType::SubsidiaryOf);
    establish_relationship(app, subsidiary2, parent_org, RelationshipType::SubsidiaryOf);
    establish_relationship(app, department, subsidiary1, RelationshipType::DepartmentOf);
    
    app.update();
    
    // Create employee identities
    let ceo = create_identity(app, IdentityType::Person, "CEO");
    let cto = create_identity(app, IdentityType::Person, "CTO");
    let developer = create_identity(app, IdentityType::Person, "Developer");
    
    app.update();
    
    // Establish employment relationships
    establish_relationship(app, ceo, parent_org, RelationshipType::EmployedBy);
    establish_relationship(app, cto, subsidiary1, RelationshipType::EmployedBy);
    establish_relationship(app, developer, department, RelationshipType::EmployedBy);
    
    // Establish reporting relationships
    establish_relationship(app, cto, ceo, RelationshipType::ReportsTo);
    establish_relationship(app, developer, cto, RelationshipType::ReportsTo);
    
    app.update();
}

/// Helper: Create identity with name
fn create_identity(app: &mut App, identity_type: IdentityType, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    app.world.send_event(CreateIdentityCommand {
        identity_type,
        external_reference: name.to_string(),
        initial_verification_level: VerificationLevel::Basic,
        claims: vec![],
    });
    id
}

/// Helper: Establish relationship
fn establish_relationship(
    app: &mut App,
    source: Uuid,
    target: Uuid,
    rel_type: RelationshipType,
) {
    app.world.send_event(EstablishRelationshipCommand {
        source_identity: source,
        target_identity: target,
        relationship_type: rel_type,
        metadata: HashMap::new(),
        expires_at: None,
    });
}

// Additional helper implementations for RelationshipType
impl ToString for RelationshipType {
    fn to_string(&self) -> String {
        match self {
            RelationshipType::EmployedBy => "Employed By",
            RelationshipType::MemberOf => "Member Of",
            RelationshipType::PartnerOf => "Partner Of",
            RelationshipType::SubsidiaryOf => "Subsidiary Of",
            RelationshipType::DepartmentOf => "Department Of",
            RelationshipType::ReportsTo => "Reports To",
            RelationshipType::Custom(s) => s,
        }.to_string()
    }
} 