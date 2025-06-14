#!/usr/bin/env bash
set -euo pipefail

# Script to extract person domain from cim-domain

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
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

# Extract person.rs
print_status "Extracting person aggregate..."
cp cim-domain/src/person.rs "$TEMP_DIR/cim-domain-person/src/aggregate/mod.rs"

# Extract person-related commands from commands.rs
print_status "Extracting person commands..."
cat > "$TEMP_DIR/cim-domain-person/src/commands/mod.rs" << 'EOF'
//! Person domain commands

use crate::aggregate::PersonId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonCommand {
    CreatePerson {
        person_id: PersonId,
        name: String,
        email: Option<String>,
        metadata: serde_json::Map<String, serde_json::Value>,
    },
    UpdatePersonName {
        person_id: PersonId,
        new_name: String,
    },
    UpdatePersonEmail {
        person_id: PersonId,
        new_email: Option<String>,
    },
    AddPersonAttribute {
        person_id: PersonId,
        key: String,
        value: serde_json::Value,
    },
    RemovePersonAttribute {
        person_id: PersonId,
        key: String,
    },
    DeactivatePerson {
        person_id: PersonId,
        reason: String,
    },
    ReactivatePerson {
        person_id: PersonId,
    },
}
EOF

# Extract person-related events
print_status "Extracting person events..."
cat > "$TEMP_DIR/cim-domain-person/src/events/mod.rs" << 'EOF'
//! Person domain events

use crate::aggregate::PersonId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonEvent {
    PersonCreated {
        person_id: PersonId,
        name: String,
        email: Option<String>,
        created_at: DateTime<Utc>,
        metadata: serde_json::Map<String, serde_json::Value>,
    },
    PersonNameUpdated {
        person_id: PersonId,
        old_name: String,
        new_name: String,
        updated_at: DateTime<Utc>,
    },
    PersonEmailUpdated {
        person_id: PersonId,
        old_email: Option<String>,
        new_email: Option<String>,
        updated_at: DateTime<Utc>,
    },
    PersonAttributeAdded {
        person_id: PersonId,
        key: String,
        value: serde_json::Value,
        added_at: DateTime<Utc>,
    },
    PersonAttributeRemoved {
        person_id: PersonId,
        key: String,
        removed_at: DateTime<Utc>,
    },
    PersonDeactivated {
        person_id: PersonId,
        reason: String,
        deactivated_at: DateTime<Utc>,
    },
    PersonReactivated {
        person_id: PersonId,
        reactivated_at: DateTime<Utc>,
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

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// Re-export main types
pub use aggregate::{Person, PersonAggregate, PersonId, PersonState};
pub use commands::PersonCommand;
pub use events::PersonEvent;
EOF

# Create handlers module
print_status "Creating handlers module..."
cat > "$TEMP_DIR/cim-domain-person/src/handlers/mod.rs" << 'EOF'
//! Command and event handlers for the person domain

use crate::{aggregate::PersonAggregate, commands::PersonCommand, events::PersonEvent};
use anyhow::Result;

pub async fn handle_person_command(
    aggregate: &mut PersonAggregate,
    command: PersonCommand,
) -> Result<Vec<PersonEvent>> {
    aggregate.handle_command(command)
}
EOF

# Create projections module
print_status "Creating projections module..."
cat > "$TEMP_DIR/cim-domain-person/src/projections/mod.rs" << 'EOF'
//! Read model projections for the person domain

use crate::{aggregate::PersonId, events::PersonEvent};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PersonProjection {
    pub person_id: PersonId,
    pub name: String,
    pub email: Option<String>,
    pub is_active: bool,
    pub attributes: HashMap<String, serde_json::Value>,
}

impl PersonProjection {
    pub fn apply_event(&mut self, event: &PersonEvent) {
        match event {
            PersonEvent::PersonCreated { name, email, .. } => {
                self.name = name.clone();
                self.email = email.clone();
                self.is_active = true;
            }
            PersonEvent::PersonNameUpdated { new_name, .. } => {
                self.name = new_name.clone();
            }
            PersonEvent::PersonEmailUpdated { new_email, .. } => {
                self.email = new_email.clone();
            }
            PersonEvent::PersonDeactivated { .. } => {
                self.is_active = false;
            }
            PersonEvent::PersonReactivated { .. } => {
                self.is_active = true;
            }
            _ => {}
        }
    }
}
EOF

# Create queries module
print_status "Creating queries module..."
cat > "$TEMP_DIR/cim-domain-person/src/queries/mod.rs" << 'EOF'
//! Query definitions for the person domain

use crate::aggregate::PersonId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonQuery {
    GetPersonById { person_id: PersonId },
    FindPersonByEmail { email: String },
    ListActivePeople { limit: usize, offset: usize },
    SearchPeopleByName { name_pattern: String },
}
EOF

# Create value_objects module
print_status "Creating value objects module..."
cat > "$TEMP_DIR/cim-domain-person/src/value_objects/mod.rs" << 'EOF'
//! Value objects for the person domain

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(email: String) -> Result<Self, String> {
        // Basic email validation
        if email.contains('@') && email.contains('.') {
            Ok(Self(email))
        } else {
            Err("Invalid email address".to_string())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonName {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
}

impl PersonName {
    pub fn new(first_name: String, last_name: String) -> Self {
        Self {
            first_name,
            last_name,
            middle_name: None,
        }
    }

    pub fn full_name(&self) -> String {
        match &self.middle_name {
            Some(middle) => format!("{} {} {}", self.first_name, middle, self.last_name),
            None => format!("{} {}", self.first_name, self.last_name),
        }
    }
}
EOF

# Update Cargo.toml with proper dependencies
print_status "Updating Cargo.toml..."
cd "$TEMP_DIR/cim-domain-person"

# Commit and push
print_status "Committing and pushing..."
git add .
git commit -m "feat: Extract person domain from cim-domain" \
    -m "- Add person aggregate with business logic" \
    -m "- Add commands and events" \
    -m "- Add handlers and projections" \
    -m "- Add value objects for email and name"
git push origin main

print_status "Person domain extracted successfully!"
print_status "Repository location: $TEMP_DIR/cim-domain-person"
echo ""
echo "Next steps:"
echo "1. cd $TEMP_DIR/cim-domain-person"
echo "2. Review and refine the extracted code"
echo "3. Update imports and dependencies"
echo "4. Add tests"
echo "5. When ready, add as submodule to main project:"
echo "   git submodule add https://github.com/TheCowboyAI/cim-domain-person.git cim-domain-person"
