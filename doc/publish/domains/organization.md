# Organization Domain

## Overview

The Organization Domain manages organizational structures, hierarchies, departments, teams, and roles within CIM. It provides comprehensive modeling of organizational relationships, reporting structures, and group dynamics while integrating with identity management and workflow systems.

## Key Concepts

### Organization
- **Definition**: A structured entity representing a company, institution, or group
- **Properties**: ID, name, type, structure, metadata
- **Types**: Corporation, non-profit, government, team, project
- **Hierarchy**: Parent-child relationships, divisions, subsidiaries

### Department
- **Definition**: A functional unit within an organization
- **Properties**: Name, purpose, head, members, budget
- **Types**: Engineering, sales, HR, finance, operations
- **Relationships**: Parent org, peer departments, sub-teams

### Team
- **Definition**: A collaborative group with shared objectives
- **Properties**: Name, purpose, lead, members, projects
- **Types**: Functional, cross-functional, project, virtual
- **Dynamics**: Formation, evolution, dissolution

### Role
- **Definition**: A set of responsibilities and permissions
- **Properties**: Title, responsibilities, permissions, requirements
- **Assignment**: Person-to-role mapping with time bounds
- **Hierarchy**: Reporting relationships, delegation chains

## Domain Events

### Commands
- `cmd.organization.create_org` - Establish new organization
- `cmd.organization.add_department` - Create department
- `cmd.organization.form_team` - Assemble team
- `cmd.organization.assign_role` - Grant role to person
- `cmd.organization.restructure` - Modify org structure

### Events
- `event.organization.org_created` - Organization established
- `event.organization.department_added` - Department created
- `event.organization.team_formed` - Team assembled
- `event.organization.role_assigned` - Role granted
- `event.organization.structure_changed` - Reorg occurred

### Queries
- `query.organization.get_structure` - Retrieve org chart
- `query.organization.find_teams` - Search teams
- `query.organization.get_members` - List personnel
- `query.organization.check_authority` - Verify permissions

## API Reference

### OrganizationAggregate
```rust
pub struct OrganizationAggregate {
    pub id: OrganizationId,
    pub name: String,
    pub org_type: OrganizationType,
    pub structure: OrgStructure,
    pub departments: HashMap<DepartmentId, Department>,
    pub teams: HashMap<TeamId, Team>,
    pub roles: HashMap<RoleId, Role>,
}
```

### Key Methods
- `create_organization()` - Initialize org
- `add_department()` - Create department
- `assign_member()` - Add person to org
- `define_role()` - Create role
- `get_hierarchy()` - Retrieve structure

## Organization Management

### Creating Organizations
```rust
// Create company
let company = CreateOrganization {
    name: "TechCorp Inc.".to_string(),
    org_type: OrganizationType::Corporation,
    structure: OrgStructure::Hierarchical,
    metadata: OrganizationMetadata {
        industry: "Technology".to_string(),
        size: OrgSize::Medium, // 50-250 employees
        founded: "2020-01-15".to_string(),
        headquarters: location_id,
    },
};

// Create subsidiary
let subsidiary = CreateSubsidiary {
    parent_org_id,
    name: "TechCorp Europe".to_string(),
    region: "EMEA".to_string(),
    autonomy_level: AutonomyLevel::SemiAutonomous,
};

// Non-profit organization
let nonprofit = CreateOrganization {
    name: "Community Foundation".to_string(),
    org_type: OrganizationType::NonProfit,
    structure: OrgStructure::Flat,
    metadata: OrganizationMetadata {
        mission: "Supporting local education".to_string(),
        tax_status: "501(c)(3)".to_string(),
        board_size: 12,
    },
};
```

### Department Structure
```rust
// Create departments
let engineering = CreateDepartment {
    organization_id,
    name: "Engineering".to_string(),
    purpose: "Product development and technical operations".to_string(),
    cost_center: "CC-100".to_string(),
    head: Some(person_id),
};

// Department hierarchy
let sub_department = CreateSubDepartment {
    parent_department_id: engineering_id,
    name: "Frontend Engineering".to_string(),
    specialization: "Web and mobile UI development".to_string(),
};

// Cross-functional structure
let product_org = MatrixOrganization {
    vertical_departments: vec![engineering_id, design_id, qa_id],
    horizontal_teams: vec![
        ProductTeam {
            name: "Checkout Team".to_string(),
            members_from: HashMap::from([
                (engineering_id, vec![eng1, eng2]),
                (design_id, vec![designer1]),
                (qa_id, vec![qa1]),
            ]),
        },
    ],
};
```

### Team Management
```rust
// Form team
let team = FormTeam {
    organization_id,
    name: "Innovation Lab".to_string(),
    team_type: TeamType::CrossFunctional,
    purpose: "Explore emerging technologies".to_string(),
    duration: TeamDuration::Permanent,
    members: vec![
        TeamMember {
            person_id: alice_id,
            role: TeamRole::Lead,
            allocation: 1.0, // 100%
        },
        TeamMember {
            person_id: bob_id,
            role: TeamRole::Member,
            allocation: 0.5, // 50%
        },
    ],
};

// Project team
let project_team = FormProjectTeam {
    project_id,
    name: "Mobile App Launch".to_string(),
    duration: TeamDuration::Temporary {
        start: SystemTime::now(),
        end: SystemTime::now() + Duration::days(90),
    },
    required_skills: vec![
        "Mobile Development",
        "UI/UX Design",
        "Project Management",
    ],
};

// Virtual team
let virtual_team = FormVirtualTeam {
    name: "Global Support Team".to_string(),
    timezone_coverage: vec![
        TimezoneCoverage {
            timezone: "America/New_York".to_string(),
            members: vec![us_team_members],
        },
        TimezoneCoverage {
            timezone: "Europe/London".to_string(),
            members: vec![eu_team_members],
        },
        TimezoneCoverage {
            timezone: "Asia/Singapore".to_string(),
            members: vec![asia_team_members],
        },
    ],
};
```

## Role and Permission Management

### Role Definition
```rust
// Define role
let role = DefineRole {
    organization_id,
    title: "Senior Software Engineer".to_string(),
    level: OrgLevel::Individual Contributor,
    responsibilities: vec![
        "Design and implement software solutions",
        "Mentor junior developers",
        "Participate in architecture decisions",
    ],
    requirements: RoleRequirements {
        experience_years: 5,
        skills: vec!["Software Development", "System Design"],
        certifications: vec![],
    },
    permissions: vec![
        Permission::ReadCode,
        Permission::WriteCode,
        Permission::ReviewCode,
        Permission::DeployToStaging,
    ],
};

// Management role
let manager_role = DefineRole {
    title: "Engineering Manager".to_string(),
    level: OrgLevel::Management,
    direct_reports: ReportRange { min: 3, max: 8 },
    budget_authority: Some(BudgetAuthority {
        spending_limit: 50000.0,
        approval_required_above: 10000.0,
    }),
    additional_permissions: vec![
        Permission::ApproveTimeOff,
        Permission::ConductReviews,
        Permission::HireEmployees,
    ],
};

// Executive role
let executive_role = DefineRole {
    title: "Chief Technology Officer".to_string(),
    level: OrgLevel::Executive,
    scope: Scope::Organization,
    strategic_responsibilities: vec![
        "Set technical vision",
        "Drive innovation strategy",
        "Build engineering culture",
    ],
};
```

### Role Assignment
```rust
// Assign role
let assignment = AssignRole {
    person_id,
    role_id,
    effective_date: SystemTime::now(),
    end_date: None, // Permanent
    reporting_to: Some(manager_id),
    dotted_line_to: vec![], // No matrix reporting
};

// Temporary assignment
let temp_assignment = AssignTemporaryRole {
    person_id,
    role_id: acting_manager_role,
    reason: "Covering for parental leave".to_string(),
    start_date: SystemTime::now(),
    end_date: SystemTime::now() + Duration::days(90),
    retain_current_role: true,
};

// Role transition
let promotion = TransitionRole {
    person_id,
    from_role: current_role_id,
    to_role: new_role_id,
    transition_date: next_month,
    transition_plan: TransitionPlan {
        handover_period: Duration::weeks(2),
        knowledge_transfer_sessions: 5,
        successor: Some(successor_id),
    },
};
```

## Organizational Dynamics

### Restructuring
```rust
// Department merger
let merger = MergeDepartments {
    departments: vec![dept_a_id, dept_b_id],
    new_name: "Unified Operations".to_string(),
    new_head: leader_id,
    integration_plan: IntegrationPlan {
        phases: vec![
            Phase {
                name: "Leadership Alignment".to_string(),
                duration: Duration::weeks(2),
            },
            Phase {
                name: "Team Integration".to_string(),
                duration: Duration::weeks(4),
            },
        ],
    },
};

// Spin-off
let spinoff = CreateSpinoff {
    from_organization: parent_org_id,
    new_organization: "InnovateCo".to_string(),
    departments_to_transfer: vec![innovation_dept_id],
    employees_to_transfer: vec![/* employee ids */],
    effective_date: future_date,
};

// Reorganization
let reorg = Reorganize {
    organization_id,
    changes: vec![
        OrgChange::MoveDepartment {
            department_id,
            new_parent: different_dept_id,
        },
        OrgChange::SplitTeam {
            team_id,
            into: vec![team_a_spec, team_b_spec],
        },
        OrgChange::CreateDepartment(new_dept_spec),
    ],
    communication_plan: CommsPlan {
        announcement_date: tomorrow,
        all_hands_meeting: next_week,
        one_on_ones_complete_by: two_weeks,
    },
};
```

### Organizational Analytics
```rust
// Analyze org structure
let analysis = AnalyzeOrganization {
    organization_id,
    metrics: vec![
        OrgMetric::SpanOfControl,
        OrgMetric::HierarchyDepth,
        OrgMetric::TeamSizeDistribution,
        OrgMetric::SkillCoverage,
    ],
};

// Analysis results
let org_health = OrganizationHealth {
    average_span_of_control: 5.2,
    max_hierarchy_depth: 6,
    team_sizes: TeamSizeDistribution {
        small: 15,  // 2-5 members
        medium: 8,  // 6-10 members
        large: 3,   // 11+ members
    },
    skill_gaps: vec![
        SkillGap {
            skill: "Machine Learning".to_string(),
            required: 5,
            available: 2,
        },
    ],
    succession_risks: vec![
        SuccessionRisk {
            role: "VP Engineering".to_string(),
            ready_candidates: 1,
            development_needed: Duration::days(180),
        },
    ],
};
```

## Integration Features

### Identity Integration
```rust
// Link person to organization
let membership = CreateMembership {
    person_id,
    organization_id,
    start_date: SystemTime::now(),
    employee_id: "EMP-12345".to_string(),
    primary_department: engineering_id,
    primary_role: engineer_role_id,
};

// Multi-org membership
let consultant = CreateConsultantRelationship {
    person_id,
    organizations: vec![
        OrgEngagement {
            org_id: client_a_id,
            role: "Technical Advisor".to_string(),
            allocation: 0.3,
        },
        OrgEngagement {
            org_id: client_b_id,
            role: "Architecture Consultant".to_string(),
            allocation: 0.5,
        },
    ],
};
```

### Workflow Integration
```rust
// Approval chains
let approval_chain = DefineApprovalChain {
    organization_id,
    approval_type: ApprovalType::PurchaseOrder,
    rules: vec![
        ApprovalRule {
            condition: "amount <= 1000".to_string(),
            approvers: vec![ApproverType::DirectManager],
        },
        ApprovalRule {
            condition: "amount <= 10000".to_string(),
            approvers: vec![
                ApproverType::DirectManager,
                ApproverType::DepartmentHead,
            ],
        },
        ApprovalRule {
            condition: "amount > 10000".to_string(),
            approvers: vec![
                ApproverType::DirectManager,
                ApproverType::DepartmentHead,
                ApproverType::CFO,
            ],
        },
    ],
};
```

## Use Cases

### HR Management
- Employee onboarding
- Org chart maintenance
- Succession planning
- Team formation

### Project Management
- Resource allocation
- Cross-functional teams
- Skill matching
- Capacity planning

### Compliance
- Reporting structures
- Authority verification
- Audit trails
- Policy enforcement

### Business Operations
- Department budgeting
- Performance tracking
- Communication flows
- Decision routing

## Performance Characteristics

- **Org Size**: Support 100,000+ members
- **Hierarchy Depth**: Unlimited levels
- **Query Speed**: <50ms for org chart
- **Update Speed**: <100ms for structure changes

## Best Practices

1. **Clear Hierarchies**: Avoid circular reporting
2. **Role Clarity**: Well-defined responsibilities
3. **Team Sizing**: Optimal team sizes (5-9)
4. **Succession Planning**: Identify key roles
5. **Regular Reviews**: Periodic structure optimization

## Related Domains

- **Identity Domain**: Person-organization links
- **Workflow Domain**: Approval processes
- **Policy Domain**: Organizational policies
- **Person Domain**: Member information
