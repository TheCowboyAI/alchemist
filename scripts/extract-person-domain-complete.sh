#!/usr/bin/env bash
set -euo pipefail

# Complete script to extract person domain from cim-domain

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "cim-domain" ]; then
    print_error "This script must be run from the alchemist repository root"
    exit 1
fi

# Create temporary directory for the new module
TEMP_DIR=$(mktemp -d)
print_status "Working in temporary directory: $TEMP_DIR"

# Clone the repository
cd "$TEMP_DIR"
git clone https://github.com/TheCowboyAI/cim-domain-person.git
cd cim-domain-person

# Create directory structure
mkdir -p src/{aggregate,commands,events,handlers,projections,queries,value_objects}

# Go back to original directory
cd "$OLDPWD"

# Extract person aggregate
print_status "Extracting person aggregate..."
cat > "$TEMP_DIR/cim-domain-person/src/aggregate/mod.rs" << 'EOF'
//! Person aggregate and related components
//!
//! A Person is an aggregate with an ID and various components that can be
//! composed to create different views (Employee, Customer, etc.)

use cim_core_domain::{AggregateRoot, Entity, EntityId, DomainError, DomainResult};
use cim_domain::{Component, ComponentStorage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::any::Any;

/// Person aggregate - represents an individual with composable components
#[derive(Debug, Clone)]
pub struct Person {
    /// Core entity data
    entity: Entity<PersonMarker>,

    /// Version for optimistic concurrency control
    version: u64,

    /// Components attached to this person
    components: ComponentStorage,

    /// Component metadata (when added, by whom, etc.)
    component_metadata: HashMap<String, ComponentMetadata>,
}

/// Marker type for Person entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersonMarker;

/// Metadata about when and why a component was added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// When this component was added
    pub added_at: std::time::SystemTime,

    /// Who added this component
    pub added_by: String,

    /// Reason or context for adding
    pub reason: Option<String>,
}

impl Person {
    /// Create a new person with basic identity
    pub fn new(id: EntityId<PersonMarker>, identity: IdentityComponent) -> Self {
        let mut components = ComponentStorage::new();
        components.add(identity).unwrap();

        let mut component_metadata = HashMap::new();
        component_metadata.insert(
            "Identity".to_string(),
            ComponentMetadata {
                added_at: std::time::SystemTime::now(),
                added_by: "system".to_string(),
                reason: Some("Initial identity".to_string()),
            },
        );

        Self {
            entity: Entity::with_id(id),
            version: 0,
            components,
            component_metadata,
        }
    }

    /// Add a component to this person
    pub fn add_component<C: Component + 'static>(
        &mut self,
        component: C,
        added_by: &str,
        reason: Option<String>,
    ) -> DomainResult<()> {
        let component_type = component.type_name().to_string();

        // Add the component
        self.components.add(component)?;

        // Add metadata
        self.component_metadata.insert(
            component_type,
            ComponentMetadata {
                added_at: std::time::SystemTime::now(),
                added_by: added_by.to_string(),
                reason,
            },
        );

        self.entity.touch();
        self.version += 1;

        Ok(())
    }

    /// Remove a component
    pub fn remove_component<C: Component + 'static>(&mut self) -> DomainResult<()> {
        let component_type = std::any::type_name::<C>();

        if self.components.remove::<C>().is_some() {
            self.component_metadata.remove(component_type);
            self.entity.touch();
            self.version += 1;
            Ok(())
        } else {
            Err(DomainError::ComponentNotFound(format!(
                "Component {} not found",
                component_type
            )))
        }
    }

    /// Get a component
    pub fn get_component<C: Component + 'static>(&self) -> Option<&C> {
        self.components.get::<C>()
    }

    /// Check if person has a component
    pub fn has_component<C: Component + 'static>(&self) -> bool {
        self.components.has::<C>()
    }

    /// Get all component types
    pub fn component_types(&self) -> Vec<String> {
        self.component_metadata.keys().cloned().collect()
    }
}

impl AggregateRoot for Person {
    type Id = EntityId<PersonMarker>;

    fn id(&self) -> Self::Id {
        self.entity.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
        self.entity.touch();
    }
}

// Re-export person-specific components
pub use crate::value_objects::{
    IdentityComponent, ContactComponent, EmailAddress, PhoneNumber,
    EmploymentComponent, PositionComponent, SkillsComponent,
    SkillProficiency, Certification, Education, AccessComponent,
    ExternalIdentifiersComponent
};

// Re-export person ID type
pub type PersonId = EntityId<PersonMarker>;
EOF

# Extract person-specific value objects
print_status "Extracting person value objects..."
cat > "$TEMP_DIR/cim-domain-person/src/value_objects/mod.rs" << 'EOF'
//! Value objects specific to the person domain

use cim_domain::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::any::Any;
use uuid::Uuid;

/// Basic identity information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityComponent {
    /// Legal name
    pub legal_name: String,

    /// Preferred name (if different from legal)
    pub preferred_name: Option<String>,

    /// Date of birth
    pub date_of_birth: Option<chrono::NaiveDate>,

    /// Government ID number (SSN, etc.)
    pub government_id: Option<String>,
}

/// Contact information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactComponent {
    /// Email addresses
    pub emails: Vec<EmailAddress>,

    /// Phone numbers
    pub phones: Vec<PhoneNumber>,

    /// Physical addresses
    pub addresses: Vec<Uuid>, // References to Location aggregates
}

/// Email address with type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    /// Email address
    pub email: String,

    /// Type (work, personal, etc.)
    pub email_type: String,

    /// Is this the primary email?
    pub is_primary: bool,

    /// Is this verified?
    pub is_verified: bool,
}

/// Phone number with type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhoneNumber {
    /// Phone number (E.164 format preferred)
    pub number: String,

    /// Type (mobile, work, home, etc.)
    pub phone_type: String,

    /// Is this the primary phone?
    pub is_primary: bool,

    /// Can receive SMS?
    pub sms_capable: bool,
}

/// Employment information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmploymentComponent {
    /// Organization ID
    pub organization_id: Uuid,

    /// Employee ID within the organization
    pub employee_id: String,

    /// Job title
    pub title: String,

    /// Department
    pub department: Option<String>,

    /// Manager's person ID
    pub manager_id: Option<Uuid>,

    /// Employment status (active, terminated, on_leave, etc.)
    pub status: String,

    /// Start date
    pub start_date: chrono::NaiveDate,

    /// End date (if terminated)
    pub end_date: Option<chrono::NaiveDate>,
}

/// Position/role information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionComponent {
    /// Position ID
    pub position_id: Uuid,

    /// Position title
    pub title: String,

    /// Level/grade
    pub level: Option<String>,

    /// Responsibilities
    pub responsibilities: Vec<String>,

    /// Required skills
    pub required_skills: Vec<String>,

    /// Start date in this position
    pub start_date: chrono::NaiveDate,

    /// End date (if no longer in position)
    pub end_date: Option<chrono::NaiveDate>,
}

/// Skills and qualifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillsComponent {
    /// Skills with proficiency levels
    pub skills: HashMap<String, SkillProficiency>,

    /// Certifications
    pub certifications: Vec<Certification>,

    /// Education
    pub education: Vec<Education>,
}

/// Skill proficiency level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillProficiency {
    /// Skill name
    pub skill: String,

    /// Proficiency level (1-5, beginner/intermediate/expert, etc.)
    pub level: String,

    /// Years of experience
    pub years_experience: Option<f32>,

    /// Last used date
    pub last_used: Option<chrono::NaiveDate>,
}

/// Certification information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certification {
    /// Certification name
    pub name: String,

    /// Issuing organization
    pub issuer: String,

    /// Issue date
    pub issue_date: chrono::NaiveDate,

    /// Expiry date (if applicable)
    pub expiry_date: Option<chrono::NaiveDate>,

    /// Credential ID
    pub credential_id: Option<String>,
}

/// Education information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Education {
    /// Institution name
    pub institution: String,

    /// Degree/qualification
    pub degree: String,

    /// Field of study
    pub field_of_study: Option<String>,

    /// Start date
    pub start_date: chrono::NaiveDate,

    /// End date
    pub end_date: Option<chrono::NaiveDate>,

    /// Grade/GPA
    pub grade: Option<String>,
}

/// Access control and permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessComponent {
    /// Roles assigned to this person
    pub roles: Vec<String>,

    /// Direct permissions
    pub permissions: Vec<String>,

    /// Groups this person belongs to
    pub groups: Vec<Uuid>,

    /// Access level/clearance
    pub access_level: Option<String>,
}

/// External system identifiers (for projections)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalIdentifiersComponent {
    /// LDAP distinguished name
    pub ldap_dn: Option<String>,

    /// Active Directory SID
    pub ad_sid: Option<String>,

    /// OAuth subject identifiers
    pub oauth_subjects: HashMap<String, String>,

    /// Other system IDs
    pub external_ids: HashMap<String, String>,
}

// Component trait implementations

impl Component for IdentityComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Identity"
    }
}

impl Component for ContactComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Contact"
    }
}

impl Component for EmploymentComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Employment"
    }
}

impl Component for PositionComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Position"
    }
}

impl Component for SkillsComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Skills"
    }
}

impl Component for AccessComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Access"
    }
}

impl Component for ExternalIdentifiersComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "ExternalIdentifiers"
    }
}
EOF

# Extract person commands
print_status "Extracting person commands..."
cat > "$TEMP_DIR/cim-domain-person/src/commands/mod.rs" << 'EOF'
//! Commands for the person domain

use crate::aggregate::PersonId;
use crate::value_objects::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Commands that can be sent to a Person aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonCommand {
    /// Register a new person in the system
    RegisterPerson {
        /// Person's unique ID (generated by caller)
        person_id: Uuid,
        /// Identity information
        identity: IdentityComponent,
        /// Optional contact information
        contact: Option<ContactComponent>,
    },

    /// Update person's contact information
    UpdateContact {
        /// Person's ID
        person_id: Uuid,
        /// New contact information
        contact: ContactComponent,
    },

    /// Add employment information
    AddEmployment {
        /// Person's ID
        person_id: Uuid,
        /// Employment details
        employment: EmploymentComponent,
    },

    /// Update employment status
    UpdateEmploymentStatus {
        /// Person's ID
        person_id: Uuid,
        /// Organization ID
        organization_id: Uuid,
        /// New status
        status: String,
        /// End date if terminated
        end_date: Option<chrono::NaiveDate>,
    },

    /// Add position information
    AddPosition {
        /// Person's ID
        person_id: Uuid,
        /// Position details
        position: PositionComponent,
    },

    /// Update skills
    UpdateSkills {
        /// Person's ID
        person_id: Uuid,
        /// Skills information
        skills: SkillsComponent,
    },

    /// Grant access
    GrantAccess {
        /// Person's ID
        person_id: Uuid,
        /// Access details
        access: AccessComponent,
    },

    /// Add external identifier
    AddExternalIdentifier {
        /// Person's ID
        person_id: Uuid,
        /// System name
        system: String,
        /// Identifier value
        identifier: String,
    },
}

/// Component update structure for batch updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonComponentUpdates {
    /// Update identity
    pub identity: Option<IdentityComponent>,

    /// Update contact
    pub contact: Option<ContactComponent>,

    /// Update employment
    pub employment: Option<EmploymentComponent>,

    /// Update position
    pub position: Option<PositionComponent>,

    /// Update skills
    pub skills: Option<SkillsComponent>,

    /// Update access
    pub access: Option<AccessComponent>,

    /// Update external identifiers
    pub external_identifiers: Option<ExternalIdentifiersComponent>,
}
EOF

# Extract person events
print_status "Extracting person events..."
cat > "$TEMP_DIR/cim-domain-person/src/events/mod.rs" << 'EOF'
//! Events for the person domain

use crate::aggregate::PersonId;
use crate::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Events that can be emitted by the Person aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonEvent {
    /// Person was registered in the system
    PersonRegistered {
        /// Person's unique ID
        person_id: Uuid,
        /// Identity component that was registered
        identity: IdentityComponent,
        /// Contact component (if provided)
        contact: Option<ContactComponent>,
        /// When the person was registered
        registered_at: DateTime<Utc>,
    },

    /// Contact information was updated
    ContactUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old contact information
        old_contact: Option<ContactComponent>,
        /// New contact information
        new_contact: ContactComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Employment was added
    EmploymentAdded {
        /// Person's ID
        person_id: Uuid,
        /// Employment details
        employment: EmploymentComponent,
        /// When added
        added_at: DateTime<Utc>,
    },

    /// Employment status changed
    EmploymentStatusChanged {
        /// Person's ID
        person_id: Uuid,
        /// Organization ID
        organization_id: Uuid,
        /// Old status
        old_status: String,
        /// New status
        new_status: String,
        /// End date if terminated
        end_date: Option<chrono::NaiveDate>,
        /// When changed
        changed_at: DateTime<Utc>,
    },

    /// Position was added
    PositionAdded {
        /// Person's ID
        person_id: Uuid,
        /// Position details
        position: PositionComponent,
        /// When added
        added_at: DateTime<Utc>,
    },

    /// Skills were updated
    SkillsUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old skills
        old_skills: Option<SkillsComponent>,
        /// New skills
        new_skills: SkillsComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Access was granted
    AccessGranted {
        /// Person's ID
        person_id: Uuid,
        /// Access details
        access: AccessComponent,
        /// When granted
        granted_at: DateTime<Utc>,
    },

    /// External identifier was added
    ExternalIdentifierAdded {
        /// Person's ID
        person_id: Uuid,
        /// System name
        system: String,
        /// Identifier value
        identifier: String,
        /// When added
        added_at: DateTime<Utc>,
    },
}

impl PersonEvent {
    /// Get the aggregate ID this event relates to
    pub fn aggregate_id(&self) -> Uuid {
        match self {
            PersonEvent::PersonRegistered { person_id, .. } => *person_id,
            PersonEvent::ContactUpdated { person_id, .. } => *person_id,
            PersonEvent::EmploymentAdded { person_id, .. } => *person_id,
            PersonEvent::EmploymentStatusChanged { person_id, .. } => *person_id,
            PersonEvent::PositionAdded { person_id, .. } => *person_id,
            PersonEvent::SkillsUpdated { person_id, .. } => *person_id,
            PersonEvent::AccessGranted { person_id, .. } => *person_id,
            PersonEvent::ExternalIdentifierAdded { person_id, .. } => *person_id,
        }
    }

    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            PersonEvent::PersonRegistered { .. } => "PersonRegistered",
            PersonEvent::ContactUpdated { .. } => "ContactUpdated",
            PersonEvent::EmploymentAdded { .. } => "EmploymentAdded",
            PersonEvent::EmploymentStatusChanged { .. } => "EmploymentStatusChanged",
            PersonEvent::PositionAdded { .. } => "PositionAdded",
            PersonEvent::SkillsUpdated { .. } => "SkillsUpdated",
            PersonEvent::AccessGranted { .. } => "AccessGranted",
            PersonEvent::ExternalIdentifierAdded { .. } => "ExternalIdentifierAdded",
        }
    }

    /// Get the NATS subject for this event
    pub fn subject(&self) -> String {
        match self {
            PersonEvent::PersonRegistered { .. } => "people.person.registered.v1",
            PersonEvent::ContactUpdated { .. } => "people.person.contact_updated.v1",
            PersonEvent::EmploymentAdded { .. } => "people.person.employment_added.v1",
            PersonEvent::EmploymentStatusChanged { .. } => "people.person.employment_status_changed.v1",
            PersonEvent::PositionAdded { .. } => "people.person.position_added.v1",
            PersonEvent::SkillsUpdated { .. } => "people.person.skills_updated.v1",
            PersonEvent::AccessGranted { .. } => "people.person.access_granted.v1",
            PersonEvent::ExternalIdentifierAdded { .. } => "people.person.external_identifier_added.v1",
        }.to_string()
    }
}
EOF

# Create handlers module
print_status "Creating handlers module..."
cat > "$TEMP_DIR/cim-domain-person/src/handlers/mod.rs" << 'EOF'
//! Command and event handlers for the person domain

pub mod command_handlers;
pub mod query_handlers;

pub use command_handlers::*;
pub use query_handlers::*;
EOF

# Create command handlers
cat > "$TEMP_DIR/cim-domain-person/src/handlers/command_handlers.rs" << 'EOF'
//! Command handlers for person domain

use crate::{
    aggregate::{Person, PersonId, PersonMarker},
    commands::PersonCommand,
    events::PersonEvent,
    value_objects::*,
};
use cim_core_domain::{AggregateRoot, EntityId, DomainResult};
use chrono::Utc;

/// Handle person commands
pub async fn handle_person_command(
    aggregate: &mut Person,
    command: PersonCommand,
) -> DomainResult<Vec<PersonEvent>> {
    match command {
        PersonCommand::RegisterPerson { person_id, identity, contact } => {
            // This would typically be handled at aggregate creation
            Ok(vec![PersonEvent::PersonRegistered {
                person_id,
                identity,
                contact,
                registered_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateContact { person_id, contact } => {
            let old_contact = aggregate.get_component::<ContactComponent>().cloned();
            aggregate.remove_component::<ContactComponent>().ok();
            aggregate.add_component(contact.clone(), "system", Some("Contact update".to_string()))?;

            Ok(vec![PersonEvent::ContactUpdated {
                person_id,
                old_contact,
                new_contact: contact,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::AddEmployment { person_id, employment } => {
            aggregate.add_component(employment.clone(), "system", Some("Employment added".to_string()))?;

            Ok(vec![PersonEvent::EmploymentAdded {
                person_id,
                employment,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateEmploymentStatus { person_id, organization_id, status, end_date } => {
            if let Some(mut employment) = aggregate.get_component::<EmploymentComponent>().cloned() {
                let old_status = employment.status.clone();
                employment.status = status.clone();
                employment.end_date = end_date;

                aggregate.remove_component::<EmploymentComponent>()?;
                aggregate.add_component(employment, "system", Some("Status update".to_string()))?;

                Ok(vec![PersonEvent::EmploymentStatusChanged {
                    person_id,
                    organization_id,
                    old_status,
                    new_status: status,
                    end_date,
                    changed_at: Utc::now(),
                }])
            } else {
                Err(cim_core_domain::DomainError::ValidationError(
                    "No employment found for organization".to_string()
                ))
            }
        }

        PersonCommand::AddPosition { person_id, position } => {
            aggregate.add_component(position.clone(), "system", Some("Position added".to_string()))?;

            Ok(vec![PersonEvent::PositionAdded {
                person_id,
                position,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateSkills { person_id, skills } => {
            let old_skills = aggregate.get_component::<SkillsComponent>().cloned();
            aggregate.remove_component::<SkillsComponent>().ok();
            aggregate.add_component(skills.clone(), "system", Some("Skills update".to_string()))?;

            Ok(vec![PersonEvent::SkillsUpdated {
                person_id,
                old_skills,
                new_skills: skills,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::GrantAccess { person_id, access } => {
            aggregate.add_component(access.clone(), "system", Some("Access granted".to_string()))?;

            Ok(vec![PersonEvent::AccessGranted {
                person_id,
                access,
                granted_at: Utc::now(),
            }])
        }

        PersonCommand::AddExternalIdentifier { person_id, system, identifier } => {
            let mut external_ids = aggregate.get_component::<ExternalIdentifiersComponent>()
                .cloned()
                .unwrap_or_else(|| ExternalIdentifiersComponent {
                    ldap_dn: None,
                    ad_sid: None,
                    oauth_subjects: HashMap::new(),
                    external_ids: HashMap::new(),
                });

            external_ids.external_ids.insert(system.clone(), identifier.clone());

            aggregate.remove_component::<ExternalIdentifiersComponent>().ok();
            aggregate.add_component(external_ids, "system", Some("External ID added".to_string()))?;

            Ok(vec![PersonEvent::ExternalIdentifierAdded {
                person_id,
                system,
                identifier,
                added_at: Utc::now(),
            }])
        }
    }
}
EOF

# Create query handlers
cat > "$TEMP_DIR/cim-domain-person/src/handlers/query_handlers.rs" << 'EOF'
//! Query handlers for person domain

use crate::{
    aggregate::{Person, PersonId},
    projections::{PersonProjection, EmployeeView, LdapProjection},
    queries::PersonQuery,
};
use cim_core_domain::DomainResult;
use std::collections::HashMap;

/// Person read model for queries
pub struct PersonReadModel {
    projections: HashMap<PersonId, PersonProjection>,
}

impl PersonReadModel {
    pub fn new() -> Self {
        Self {
            projections: HashMap::new(),
        }
    }

    /// Handle a person query
    pub async fn handle_query(&self, query: PersonQuery) -> DomainResult<PersonQueryResult> {
        match query {
            PersonQuery::GetPersonById { person_id } => {
                if let Some(projection) = self.projections.get(&person_id) {
                    Ok(PersonQueryResult::Person(projection.clone()))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::FindPersonByEmail { email } => {
                let found: Vec<_> = self.projections.values()
                    .filter(|p| p.emails.iter().any(|e| e.email == email))
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(found))
            }

            PersonQuery::ListActivePeople { limit, offset } => {
                let people: Vec<_> = self.projections.values()
                    .filter(|p| p.is_active)
                    .skip(offset)
                    .take(limit)
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(people))
            }

            PersonQuery::SearchPeopleByName { name_pattern } => {
                let pattern = name_pattern.to_lowercase();
                let found: Vec<_> = self.projections.values()
                    .filter(|p| p.name.to_lowercase().contains(&pattern))
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(found))
            }
        }
    }
}

/// Result types for person queries
#[derive(Debug, Clone)]
pub enum PersonQueryResult {
    Person(PersonProjection),
    People(Vec<PersonProjection>),
    EmployeeView(EmployeeView),
    LdapProjection(LdapProjection),
    NotFound,
}
EOF

# Create projections module
print_status "Creating projections module..."
cat > "$TEMP_DIR/cim-domain-person/src/projections/mod.rs" << 'EOF'
//! Read model projections for the person domain

use crate::{
    aggregate::{Person, PersonId},
    events::PersonEvent,
    value_objects::*,
};
use cim_core_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Person projection for read models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonProjection {
    pub person_id: PersonId,
    pub name: String,
    pub emails: Vec<EmailAddress>,
    pub phones: Vec<PhoneNumber>,
    pub is_active: bool,
    pub employment: Option<EmploymentComponent>,
    pub position: Option<PositionComponent>,
    pub skills: Option<SkillsComponent>,
    pub access: Option<AccessComponent>,
}

impl PersonProjection {
    /// Apply an event to update the projection
    pub fn apply_event(&mut self, event: &PersonEvent) {
        match event {
            PersonEvent::PersonRegistered { person_id, identity, contact, .. } => {
                self.person_id = PersonId::from_uuid(*person_id);
                self.name = identity.preferred_name.as_ref()
                    .unwrap_or(&identity.legal_name)
                    .clone();
                if let Some(contact) = contact {
                    self.emails = contact.emails.clone();
                    self.phones = contact.phones.clone();
                }
                self.is_active = true;
            }

            PersonEvent::ContactUpdated { new_contact, .. } => {
                self.emails = new_contact.emails.clone();
                self.phones = new_contact.phones.clone();
            }

            PersonEvent::EmploymentAdded { employment, .. } => {
                self.employment = Some(employment.clone());
                self.is_active = employment.status == "active";
            }

            PersonEvent::EmploymentStatusChanged { new_status, .. } => {
                if let Some(ref mut emp) = self.employment {
                    emp.status = new_status.clone();
                    self.is_active = new_status == "active";
                }
            }

            PersonEvent::PositionAdded { position, .. } => {
                self.position = Some(position.clone());
            }

            PersonEvent::SkillsUpdated { new_skills, .. } => {
                self.skills = Some(new_skills.clone());
            }

            PersonEvent::AccessGranted { access, .. } => {
                self.access = Some(access.clone());
            }

            _ => {}
        }
    }
}

/// Employee view of a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeView {
    /// The person's unique identifier
    pub person_id: PersonId,
    /// Identity information (name, DOB, etc.)
    pub identity: IdentityComponent,
    /// Contact information (email, phone, address)
    pub contact: ContactComponent,
    /// Employment details (organization, title, department)
    pub employment: EmploymentComponent,
    /// Current position information if available
    pub position: Option<PositionComponent>,
    /// Skills and certifications if available
    pub skills: Option<SkillsComponent>,
}

impl EmployeeView {
    /// Create employee view from person
    pub fn from_person(person: &Person) -> DomainResult<Self> {
        let identity = person.get_component::<IdentityComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Person missing identity component".to_string()
            ))?
            .clone();

        let contact = person.get_component::<ContactComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Employee missing contact component".to_string()
            ))?
            .clone();

        let employment = person.get_component::<EmploymentComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Employee missing employment component".to_string()
            ))?
            .clone();

        let position = person.get_component::<PositionComponent>().cloned();
        let skills = person.get_component::<SkillsComponent>().cloned();

        Ok(Self {
            person_id: person.id(),
            identity,
            contact,
            employment,
            position,
            skills,
        })
    }
}

/// LDAP projection for directory services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapProjection {
    /// Distinguished Name (full LDAP path)
    pub dn: String,
    /// Common Name (typically the preferred name)
    pub cn: String,
    /// Surname (last name)
    pub sn: String,
    /// Given name (first name)
    pub given_name: String,
    /// Email addresses
    pub mail: Vec<String>,
    /// Phone numbers
    pub telephone_number: Vec<String>,
    /// Job title if employed
    pub title: Option<String>,
    /// Department if employed
    pub department: Option<String>,
    /// Manager's DN if applicable
    pub manager: Option<String>,
}

impl LdapProjection {
    /// Create LDAP projection from person
    pub fn from_person(person: &Person, base_dn: &str) -> DomainResult<Self> {
        let identity = person.get_component::<IdentityComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Person missing identity component".to_string()
            ))?;

        let contact = person.get_component::<ContactComponent>();
        let employment = person.get_component::<EmploymentComponent>();

        // Parse name (simple split for now)
        let name_parts: Vec<&str> = identity.legal_name.split_whitespace().collect();
        let given_name = name_parts.first().unwrap_or(&"").to_string();
        let sn = name_parts.last().unwrap_or(&"").to_string();

        let cn = identity.preferred_name.as_ref()
            .unwrap_or(&identity.legal_name)
            .clone();

        let dn = format!("cn={},ou=people,{}", cn, base_dn);

        let mail = contact.map(|c| c.emails.iter()
            .map(|e| e.email.clone())
            .collect())
            .unwrap_or_default();

        let telephone_number = contact.map(|c| c.phones.iter()
            .map(|p| p.number.clone())
            .collect())
            .unwrap_or_default();

        let (title, department) = employment.map(|e| (Some(e.title), e.department))
            .unwrap_or((None, None));

        Ok(Self {
            dn,
            cn,
            sn,
            given_name,
            mail,
            telephone_number,
            title,
            department,
            manager: None, // Would need to resolve manager's DN
        })
    }
}
EOF

# Create queries module
print_status "Creating queries module..."
cat > "$TEMP_DIR/cim-domain-person/src/queries/mod.rs" << 'EOF'
//! Query definitions for the person domain

use crate::aggregate::PersonId;
use serde::{Deserialize, Serialize};

/// Queries that can be executed against the person domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonQuery {
    /// Get a person by their ID
    GetPersonById {
        person_id: PersonId
    },

    /// Find a person by email address
    FindPersonByEmail {
        email: String
    },

    /// List all active people with pagination
    ListActivePeople {
        limit: usize,
        offset: usize
    },

    /// Search people by name pattern
    SearchPeopleByName {
        name_pattern: String
    },

    /// Get employee view for a person
    GetEmployeeView {
        person_id: PersonId
    },

    /// Get LDAP projection for a person
    GetLdapProjection {
        person_id: PersonId,
        base_dn: String,
    },

    /// Find people by organization
    FindPeopleByOrganization {
        organization_id: uuid::Uuid,
        include_inactive: bool,
    },

    /// Find people by skill
    FindPeopleBySkill {
        skill_name: String,
        min_proficiency: Option<String>,
    },

    /// Find people by role
    FindPeopleByRole {
        role: String,
    },
}
EOF

# Create lib.rs
print_status "Creating lib.rs..."
cat > "$TEMP_DIR/cim-domain-person/src/lib.rs" << 'EOF'
//! Person/People domain for the Composable Information Machine
//!
//! This crate provides the person domain implementation, including:
//! - Person aggregate with business logic
//! - Commands for person operations
//! - Events representing person state changes
//! - Command and query handlers
//! - Projections for read models
//! - Value objects specific to people

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// Re-export main types
pub use aggregate::{Person, PersonId, PersonMarker};
pub use commands::{PersonCommand, PersonComponentUpdates};
pub use events::PersonEvent;
pub use projections::{PersonProjection, EmployeeView, LdapProjection};
pub use queries::PersonQuery;
pub use value_objects::{
    IdentityComponent, ContactComponent, EmailAddress, PhoneNumber,
    EmploymentComponent, PositionComponent, SkillsComponent,
    SkillProficiency, Certification, Education, AccessComponent,
    ExternalIdentifiersComponent
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
EOF

# Update Cargo.toml with proper metadata
print_status "Updating Cargo.toml..."
cd "$TEMP_DIR/cim-domain-person"
cat > Cargo.toml << 'EOF'
[package]
name = "cim-domain-person"
version = "0.1.0"
edition = "2021"
authors = ["The Cowboy AI"]
description = "Person/People domain for the Composable Information Machine"
license = "MIT OR Apache-2.0"
repository = "https://github.com/TheCowboyAI/cim-domain-person"
keywords = ["domain-driven-design", "event-sourcing", "cqrs", "person", "identity"]
categories = ["data-structures", "web-programming"]

[dependencies]
# Core dependencies
cim-core-domain = { path = "../cim-core-domain" }
cim-domain = { path = "../cim-domain" }

# Async runtime
tokio = { version = "1.42", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
tracing = "0.1"

# UUID generation
uuid = { version = "1.11", features = ["v4", "serde"] }

# Time handling
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio-test = "0.4"
pretty_assertions = "1.4"
tracing-subscriber = "0.3"

[features]
default = []
# Enable test utilities
test-utils = []
EOF

# Create a basic test
print_status "Creating basic test..."
mkdir -p tests
cat > tests/person_aggregate_test.rs << 'EOF'
use cim_domain_person::{
    Person, PersonId, PersonCommand, PersonEvent,
    IdentityComponent, ContactComponent, EmailAddress,
};
use cim_core_domain::{EntityId, AggregateRoot};
use uuid::Uuid;

#[test]
fn test_person_creation() {
    let person_id = PersonId::from_uuid(Uuid::new_v4());
    let identity = IdentityComponent {
        legal_name: "John Doe".to_string(),
        preferred_name: Some("Johnny".to_string()),
        date_of_birth: None,
        government_id: None,
    };

    let person = Person::new(person_id, identity.clone());

    assert_eq!(person.id(), person_id);
    assert_eq!(person.version(), 0);
    assert!(person.has_component::<IdentityComponent>());
}

#[test]
fn test_add_contact_component() {
    let person_id = PersonId::from_uuid(Uuid::new_v4());
    let identity = IdentityComponent {
        legal_name: "Jane Doe".to_string(),
        preferred_name: None,
        date_of_birth: None,
        government_id: None,
    };

    let mut person = Person::new(person_id, identity);

    let contact = ContactComponent {
        emails: vec![EmailAddress {
            email: "jane@example.com".to_string(),
            email_type: "work".to_string(),
            is_primary: true,
            is_verified: false,
        }],
        phones: vec![],
        addresses: vec![],
    };

    let result = person.add_component(contact.clone(), "test", Some("Adding contact".to_string()));
    assert!(result.is_ok());
    assert!(person.has_component::<ContactComponent>());
    assert_eq!(person.version(), 1);
}
EOF

# Commit and push
print_status "Committing and pushing..."
git add .
git commit -m "feat: Extract person domain from cim-domain" \
    -m "- Add person aggregate with component-based design" \
    -m "- Add commands and events for person operations" \
    -m "- Add command and query handlers" \
    -m "- Add projections (PersonProjection, EmployeeView, LdapProjection)" \
    -m "- Add value objects specific to people" \
    -m "- Add basic tests"
git push origin main

print_status "Person domain extracted successfully!"
print_status "Repository location: $TEMP_DIR/cim-domain-person"
echo ""
echo "Next steps:"
echo "1. cd $TEMP_DIR/cim-domain-person"
echo "2. Review the extracted code"
echo "3. Run tests: cargo test"
echo "4. When ready, add as submodule to main project:"
echo "   git submodule add https://github.com/TheCowboyAI/cim-domain-person.git cim-domain-person"
echo ""
echo "5. Update cim-domain to remove person-specific code:"
echo "   - Remove person.rs"
echo "   - Remove person-specific commands from commands.rs"
echo "   - Remove person-specific events from events.rs and domain_events.rs"
echo "   - Remove person-specific handlers"
echo "   - Update lib.rs exports"
</rewritten_file>
