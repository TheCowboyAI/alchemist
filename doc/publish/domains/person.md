# Person Domain

## Overview

The Person Domain manages extended personal information beyond core identity, including contact details, preferences, skills, and personal metadata. It complements the Identity Domain by handling the rich, detailed information about individuals while delegating identity verification and relationships to the Identity Domain.

## Key Concepts

### Person Profile
- **Definition**: Comprehensive information about an individual
- **Components**: Contact info, preferences, skills, biography
- **Privacy**: Configurable visibility and access controls
- **Versioning**: Historical tracking of profile changes

### Contact Information
- **Definition**: Methods to reach or communicate with a person
- **Types**: Email, phone, address, social media, messaging
- **Properties**: Type, value, verified status, preferred flag
- **Validation**: Format checking and verification status

### Personal Preferences
- **Definition**: Individual settings and choices
- **Categories**: Communication, privacy, UI, notifications
- **Examples**: Language, timezone, theme, contact methods
- **Application**: System-wide preference enforcement

### Skills and Expertise
- **Definition**: Professional capabilities and knowledge areas
- **Properties**: Skill name, proficiency level, years of experience
- **Validation**: Endorsements, certifications, assessments
- **Discovery**: Searchable skill inventory

## Domain Events

### Commands
- `cmd.person.create_profile` - Initialize person profile
- `cmd.person.update_contact` - Modify contact information
- `cmd.person.set_preferences` - Update preferences
- `cmd.person.add_skill` - Register new skill
- `cmd.person.verify_contact` - Confirm contact method

### Events
- `event.person.profile_created` - New profile initialized
- `event.person.contact_updated` - Contact info changed
- `event.person.preference_set` - Preference updated
- `event.person.skill_added` - New skill registered
- `event.person.contact_verified` - Contact confirmed

### Queries
- `query.person.get_profile` - Retrieve full profile
- `query.person.find_by_skill` - Search by expertise
- `query.person.get_contact_methods` - List contacts
- `query.person.check_preferences` - Get preferences

## API Reference

### PersonAggregate
```rust
pub struct PersonAggregate {
    pub id: PersonId,
    pub identity_id: IdentityId, // Link to Identity Domain
    pub profile: PersonProfile,
    pub contacts: HashMap<ContactId, ContactInfo>,
    pub preferences: PreferenceSet,
    pub skills: HashMap<SkillId, Skill>,
}
```

### Key Methods
- `create_profile()` - Initialize person data
- `add_contact_method()` - Add contact info
- `update_preference()` - Set preference
- `add_skill()` - Register expertise
- `get_preferred_contact()` - Best contact method

## Profile Management

### Creating a Person Profile
```rust
// Create new person profile
let profile = CreateProfile {
    identity_id, // From Identity Domain
    basic_info: BasicInfo {
        display_name: "Jane Smith".to_string(),
        bio: "Software engineer passionate about AI".to_string(),
        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
    },
    initial_contacts: vec![
        ContactInfo {
            contact_type: ContactType::Email,
            value: "jane@example.com".to_string(),
            is_primary: true,
            is_verified: false,
        },
    ],
};

// Set preferences
let preferences = SetPreferences {
    person_id,
    preferences: vec![
        Preference::Language("en-US".to_string()),
        Preference::Timezone("America/New_York".to_string()),
        Preference::Theme(Theme::Dark),
        Preference::NotificationChannel(Channel::Email),
    ],
};
```

### Contact Management
```rust
// Add multiple contact methods
let add_phone = AddContact {
    person_id,
    contact: ContactInfo {
        contact_type: ContactType::Phone,
        value: "+1-555-0123".to_string(),
        label: Some("Mobile".to_string()),
        is_primary: false,
        is_verified: false,
    },
};

let add_linkedin = AddContact {
    person_id,
    contact: ContactInfo {
        contact_type: ContactType::Social,
        value: "linkedin.com/in/janesmith".to_string(),
        label: Some("LinkedIn".to_string()),
        is_primary: false,
        is_verified: true,
    },
};

// Verify contact method
let verify = VerifyContact {
    person_id,
    contact_id,
    verification_code: "123456".to_string(),
};
```

### Skills and Expertise
```rust
// Add professional skills
let add_skill = AddSkill {
    person_id,
    skill: Skill {
        name: "Rust Programming".to_string(),
        category: SkillCategory::Technology,
        proficiency: Proficiency::Expert,
        years_experience: 5,
        certifications: vec![
            Certification {
                name: "Rust Certified Developer".to_string(),
                issuer: "Rust Foundation".to_string(),
                date_earned: "2023-01-15".to_string(),
                expiry: None,
            },
        ],
    },
};

// Search by skills
let query = FindBySkill {
    skills: vec!["Rust", "WebAssembly"],
    minimum_proficiency: Proficiency::Intermediate,
    require_all: false,
};
```

## Privacy and Access Control

### Visibility Settings
```rust
pub enum Visibility {
    Public,              // Anyone can see
    Authenticated,       // Logged-in users only
    Connections,         // Connected identities only
    Private,            // Only the person
    Custom(Vec<IdentityId>), // Specific identities
}

// Set profile visibility
let privacy = UpdatePrivacySettings {
    person_id,
    settings: PrivacySettings {
        profile_visibility: Visibility::Authenticated,
        contact_visibility: Visibility::Connections,
        skill_visibility: Visibility::Public,
        preference_visibility: Visibility::Private,
    },
};
```

### Data Minimization
```rust
// Get public view of profile
let public_view = person.get_public_profile();
// Returns only: display_name, bio, public skills

// Get connection view
let connection_view = person.get_connection_profile(viewer_id);
// Returns: public data + contact info + shared preferences

// Get full view (owner only)
let full_view = person.get_full_profile(owner_id);
// Returns: all data including private preferences
```

## Integration Patterns

### Identity Domain Integration
```rust
// Person profile linked to identity
impl PersonAggregate {
    pub fn from_identity(identity_id: IdentityId) -> Self {
        Self {
            id: PersonId::new(),
            identity_id,
            profile: PersonProfile::default(),
            // ... other fields
        }
    }

    pub fn verify_identity_ownership(&self, identity_id: &IdentityId) -> bool {
        self.identity_id == *identity_id
    }
}
```

### Workflow Integration
```rust
// Use preferences in workflows
let notification_step = WorkflowStep::SendNotification {
    recipient: person_id,
    use_preferred_channel: true, // Reads from person preferences
};

// Skill-based task assignment
let find_assignee = FindPersonWithSkills {
    required_skills: vec!["Project Management", "Agile"],
    preferred_skills: vec!["Scrum Master"],
    availability: true,
};
```

### Agent Integration
```rust
// Personalized agent interactions
let agent_config = PersonalizedAgentConfig {
    person_id,
    use_preferences: true, // Language, timezone, etc.
    access_skills: true,   // Agent can see person's expertise
    respect_privacy: true, // Honor visibility settings
};
```

## Preference System

### Preference Categories
```rust
pub enum PreferenceCategory {
    Communication {
        preferred_language: Language,
        preferred_channel: ContactType,
        notification_frequency: Frequency,
    },
    Display {
        theme: Theme,
        date_format: DateFormat,
        timezone: Timezone,
    },
    Privacy {
        data_retention: Duration,
        analytics_opt_out: bool,
        marketing_opt_out: bool,
    },
    Accessibility {
        high_contrast: bool,
        screen_reader: bool,
        font_size: FontSize,
    },
}
```

### Preference Inheritance
```rust
// System defaults → Organization defaults → Personal preferences
let effective_preferences = PreferenceResolver::resolve(
    system_defaults,
    org_defaults,
    person_preferences,
);
```

## Use Cases

### Employee Directory
- Searchable profiles
- Skill matching
- Contact lookup
- Team formation

### Customer Profiles
- Preference tracking
- Communication history
- Service personalization
- Privacy compliance

### Expert Networks
- Skill discovery
- Expertise matching
- Collaboration facilitation
- Knowledge sharing

### Personal Data Management
- Self-service updates
- Privacy controls
- Data portability
- Consent management

## Performance Characteristics

- **Profile Capacity**: 10M+ profiles
- **Skill Search**: <50ms with indices
- **Preference Lookup**: <5ms cached
- **Contact Validation**: <100ms

## Best Practices

1. **Privacy First**: Default to minimal data exposure
2. **Verification**: Verify critical contact methods
3. **Preference Caching**: Cache frequently accessed preferences
4. **Skill Taxonomy**: Use consistent skill categorization
5. **Audit Trail**: Track all profile modifications

## Related Domains

- **Identity Domain**: Core identity and relationships
- **Organization Domain**: Employment context
- **Agent Domain**: Personalized interactions
- **Workflow Domain**: Preference-aware processes 