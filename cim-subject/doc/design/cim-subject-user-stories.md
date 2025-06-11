# CIM Subject Module - Private Mortgage Lending User Stories

## Overview

The CIM Subject module enables sophisticated event-driven workflows in the Private Mortgage Lending domain through subject-based routing, permissions, and transformations. These user stories demonstrate how mortgage lenders, brokers, underwriters, and service providers collaborate through well-defined subject patterns.

## Epic 1: Loan Application Processing

### Background
A private mortgage lending platform processes loan applications from multiple brokers, validates documents, verifies borrower information, and coordinates with various service providers (appraisers, title companies, underwriters) while maintaining compliance with lending regulations.

### User Story 1.1: Multi-Broker Application Routing
**As a** Loan Operations Manager
**I want to** route loan applications from different brokers to appropriate processors
**So that** each broker's applications follow their agreed-upon workflows and SLAs

**Acceptance Criteria:**
- Applications use subject pattern: `lending.<broker>.<region>.application.<stage>.<event>.<version>`
- Example: `lending.premierbrokers.northeast.application.submitted.new.v1`
- Broker-specific processors handle their own applications
- VIP brokers get priority routing
- Compliance rules are enforced per jurisdiction

**Subject Algebra Usage:**
```rust
// Create broker-specific application subject
let application = SubjectBuilder::new()
    .context("lending.premierbrokers.northeast")
    .aggregate("application")
    .event_type("submitted")
    .version("v1")
    .build()?;

// Permission rules for broker isolation
let broker_permissions = PermissionRule::new()
    .allow("lending.premierbrokers.*.application.>")
    .deny("lending.*.*.application.*.internal.*")
    .require_attribute("broker_id", "premier-brokers-001");

// VIP broker priority routing
let vip_pattern = Pattern::new("lending.vip-*.*.application.submitted.>");

// Compose workflow based on broker tier
let workflow = if vip_pattern.matches(&application) {
    algebra.compose(
        &application,
        AlgebraOperation::Inject("priority", 3)
    )?
} else {
    application.clone()
};
```

### User Story 1.2: Document Collection and Validation
**As an** Underwriting Analyst
**I want to** track document collection and validation across multiple providers
**So that** I can ensure all required documents are received and validated before underwriting

**Acceptance Criteria:**
- Document events follow pattern: `lending.<broker>.<type>.document.<doc_type>.<status>.<version>`
- Validation results route to appropriate queues
- Missing documents trigger automated reminders
- Document expiration is tracked (e.g., pay stubs > 30 days old)

**Subject Algebra Usage:**
```rust
// Document submission event
let doc_event = Subject::new("lending.premierbrokers.income.document.paystub.received.v1")?;

// Parse document type and age for routing
let doc_parser = SubjectParser::new()
    .add_rule(ParseRule::new(
        "document_classifier",
        Pattern::new("lending.*.*.document.*.received.*"),
        Box::new(|subject| {
            let parts = subject.split('.').collect::<Vec<_>>();
            let doc_type = parts[4];

            match doc_type {
                "paystub" => Some("income-verification".to_string()),
                "w2" => Some("income-verification".to_string()),
                "bank-statement" => Some("asset-verification".to_string()),
                "appraisal" => Some("property-valuation".to_string()),
                "title-report" => Some("title-verification".to_string()),
                _ => Some("general-documents".to_string()),
            }
        }),
    ));

// Compose validation workflow
let validation_workflow = algebra.compose(
    &doc_event,
    AlgebraOperation::Sequence(vec![
        // OCR and data extraction
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.event_type = "ocr-requested".to_string();
            new_parts
        })),
        // Validation against requirements
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.aggregate = "validation".to_string();
            new_parts.event_type = "checking".to_string();
            new_parts
        })),
        // Route based on validation result
        Box::new(AlgebraOperation::Choice(vec![
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.event_type = "approved".to_string();
                new_parts
            })),
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.event_type = "rejected".to_string();
                new_parts
            })),
        ])),
    ])
)?;
```

### User Story 1.3: Borrower Identity Verification
**As a** Compliance Officer
**I want to** verify borrower identities across multiple data sources
**So that** we comply with KYC/AML requirements and prevent fraud

**Acceptance Criteria:**
- Identity verification uses pattern: `lending.<source>.verification.<type>.<entity>.<result>.<version>`
- Multiple verification sources are consulted (credit bureaus, government databases)
- Discrepancies trigger manual review
- All verification attempts are logged for audit

**Subject Algebra Usage:**
```rust
// Identity verification request
let verification = Subject::new("lending.application.verification.identity.person.requested.v1")?;

// Translator for different verification providers
let provider_translator = Translator::new()
    .add_rule(TranslationRule {
        pattern: Pattern::new("lending.application.verification.identity.*.requested.*"),
        transform: Box::new(|subject| {
            // Route to Experian
            Ok(Subject::from_parts(SubjectParts::new(
                "lending.experian",
                "credit-check",
                "identity-verify",
                "v2"
            )))
        }),
    })
    .add_rule(TranslationRule {
        pattern: Pattern::new("lending.application.verification.identity.*.requested.*"),
        transform: Box::new(|subject| {
            // Route to LexisNexis
            Ok(Subject::from_parts(SubjectParts::new(
                "lending.lexisnexis",
                "person-search",
                "comprehensive",
                "v1"
            )))
        }),
    });

// Compose parallel verification
let verification_workflow = algebra.compose(
    &verification,
    AlgebraOperation::Parallel(vec![
        // Credit bureau check
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.equifax".to_string();
            new_parts.aggregate = "credit-report".to_string();
            new_parts
        })),
        // Government database check
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.ofac".to_string();
            new_parts.aggregate = "sanctions-check".to_string();
            new_parts
        })),
        // Previous lending history
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.internal".to_string();
            new_parts.aggregate = "history-check".to_string();
            new_parts
        })),
    ])
)?;
```

## Epic 2: Property Valuation and Title Processing

### Background
The lending platform coordinates property appraisals, title searches, and insurance verification across multiple vendors while ensuring accurate property valuation and clear title.

### User Story 2.1: Appraisal Coordination
**As a** Loan Processor
**I want to** coordinate property appraisals with multiple appraisal management companies
**So that** we get timely and accurate property valuations

**Acceptance Criteria:**
- Appraisal requests route to available AMCs based on geography and property type
- Rush orders get priority routing
- Appraisal reviews trigger if value differs significantly from AVM
- All appraisal documents are collected and validated

**Subject Algebra Usage:**
```rust
// Appraisal request
let appraisal_request = Subject::new("lending.property.valuation.appraisal.ordered.v1")?;

// Parse property location for AMC routing
let location_parser = SubjectParser::new()
    .add_rule(ParseRule::new(
        "property_location",
        Pattern::new("lending.property.*.appraisal.ordered.*"),
        Box::new(|subject| {
            // In practice, would extract from message metadata
            Some("northeast-urban".to_string())
        }),
    ));

// Route to appropriate AMC based on location and type
let amc_routing = algebra.compose(
    &appraisal_request,
    AlgebraOperation::Choice(vec![
        // Urban properties to AMC-A
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.amc-alpha".to_string();
            new_parts.aggregate = "appraisal-order".to_string();
            new_parts
        })),
        // Rural properties to AMC-B
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.amc-beta".to_string();
            new_parts.aggregate = "appraisal-order".to_string();
            new_parts
        })),
        // Complex properties to specialized AMC
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.amc-specialty".to_string();
            new_parts.aggregate = "complex-appraisal".to_string();
            new_parts
        })),
    ])
)?;

// Appraisal review workflow
let review_workflow = algebra.compose(
    &Subject::new("lending.property.valuation.appraisal.completed.v1")?,
    AlgebraOperation::Sequence(vec![
        // Compare with AVM
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.event_type = "avm-comparison".to_string();
            new_parts
        })),
        // Check for review triggers
        Box::new(AlgebraOperation::Choice(vec![
            // Significant variance
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.event_type = "review-required".to_string();
                new_parts
            })),
            // Within acceptable range
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.event_type = "approved".to_string();
                new_parts
            })),
        ])),
    ])
)?;
```

### User Story 2.2: Title Search and Insurance
**As a** Title Coordinator
**I want to** manage title searches and insurance across multiple title companies
**So that** we ensure clear title and proper insurance coverage

**Acceptance Criteria:**
- Title orders route to companies based on state and county
- Title defects trigger curative workflows
- Insurance quotes are collected from multiple providers
- Chain of title is validated against public records

**Subject Algebra Usage:**
```rust
// Title search request
let title_request = Subject::new("lending.property.title.search.initiated.v1")?;

// State-specific routing rules
let state_router = SubjectParser::new()
    .add_rule(ParseRule::new(
        "state_routing",
        Pattern::new("lending.property.title.*.initiated.*"),
        Box::new(|subject| {
            // Extract state from property data
            Some("NY".to_string())
        }),
    ));

// Compose title workflow with state-specific requirements
let title_workflow = algebra.compose(
    &title_request,
    AlgebraOperation::Sequence(vec![
        // Route to title company
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.firstamerican-ny".to_string();
            new_parts.aggregate = "title-order".to_string();
            new_parts
        })),
        // Parallel searches
        Box::new(AlgebraOperation::Parallel(vec![
            // Current owner search
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.event_type = "owner-search".to_string();
                new_parts
            })),
            // Lien search
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.event_type = "lien-search".to_string();
                new_parts
            })),
            // Tax search
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.event_type = "tax-search".to_string();
                new_parts
            })),
        ])),
    ])
)?;
```

## Epic 3: Rate Shopping and Loan Pricing

### Background
The platform shops for competitive rates across multiple private lenders, considering borrower profile, property characteristics, and market conditions.

### User Story 3.1: Multi-Lender Rate Shopping
**As a** Mortgage Broker
**I want to** get rate quotes from multiple private lenders simultaneously
**So that** I can offer borrowers the best available rates

**Acceptance Criteria:**
- Rate requests are sent to qualified lenders based on loan criteria
- Lender responses are normalized for comparison
- Rate locks are coordinated across lenders
- Historical rate data influences future routing

**Subject Algebra Usage:**
```rust
// Rate request
let rate_request = Subject::new("lending.rates.quote.standard.requested.v1")?;

// Parse loan characteristics for lender matching
let loan_parser = SubjectParser::new()
    .add_rule(ParseRule::new(
        "loan_profile",
        Pattern::new("lending.rates.quote.*.requested.*"),
        Box::new(|subject| {
            // Categorize loan type
            Some("non-qm-high-ltv".to_string())
        }),
    ));

// Broadcast to qualified lenders
let rate_shopping = algebra.compose(
    &rate_request,
    AlgebraOperation::Parallel(vec![
        // Lender A - specializes in non-QM
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.velocity-mortgage".to_string();
            new_parts.aggregate = "rate-quote".to_string();
            new_parts
        })),
        // Lender B - high LTV specialist
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.citadel-funding".to_string();
            new_parts.aggregate = "pricing-engine".to_string();
            new_parts
        })),
        // Lender C - competitive rates
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.pinnacle-capital".to_string();
            new_parts.aggregate = "rate-calculator".to_string();
            new_parts
        })),
    ])
)?;

// Normalize lender responses
let response_normalizer = Translator::new()
    .add_rule(TranslationRule {
        pattern: Pattern::new("lending.velocity-mortgage.rate-quote.*.calculated.*"),
        transform: Box::new(|subject| {
            Ok(Subject::from_parts(SubjectParts::new(
                "lending.rates",
                "normalized-quote",
                "received",
                "v2"
            )))
        }),
    })
    .add_rule(TranslationRule {
        pattern: Pattern::new("lending.*.*.*.calculated.*"),
        transform: Box::new(|subject| {
            // Generic normalization for other lenders
            Ok(Subject::from_parts(SubjectParts::new(
                "lending.rates",
                "normalized-quote",
                "received",
                "v2"
            )))
        }),
    });
```

### User Story 3.2: Dynamic Pricing Adjustments
**As a** Pricing Analyst
**I want to** adjust loan pricing based on risk factors and market conditions
**So that** we maintain profitability while remaining competitive

**Acceptance Criteria:**
- Risk factors automatically adjust base rates
- Market conditions trigger pricing updates
- Competitor rates influence pricing decisions
- All pricing decisions are auditable

**Subject Algebra Usage:**
```rust
// Base pricing event
let base_price = Subject::new("lending.pricing.loan.base-rate.calculated.v1")?;

// Risk adjustment workflow
let risk_adjustment = algebra.compose(
    &base_price,
    AlgebraOperation::Sequence(vec![
        // Credit risk adjustment
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.aggregate = "risk-adjustment".to_string();
            new_parts.event_type = "credit-scored".to_string();
            new_parts
        })),
        // Property risk adjustment
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.event_type = "property-scored".to_string();
            new_parts
        })),
        // Market conditions
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.event_type = "market-adjusted".to_string();
            new_parts
        })),
        // Final pricing
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.aggregate = "final-price".to_string();
            new_parts.event_type = "determined".to_string();
            new_parts
        })),
    ])
)?;
```

## Epic 4: Underwriting and Decision Engine

### Background
The platform performs automated underwriting using multiple data sources and rule engines while maintaining human oversight for complex cases.

### User Story 4.1: Automated Underwriting Workflow
**As an** Underwriting Manager
**I want to** automate standard underwriting decisions
**So that** we can process loans faster while maintaining quality

**Acceptance Criteria:**
- Standard loans go through automated decisioning
- Complex loans trigger manual review
- All decisions are explainable and auditable
- Underwriting rules update without system changes

**Subject Algebra Usage:**
```rust
// Underwriting request
let underwriting = Subject::new("lending.underwriting.application.analysis.started.v1")?;

// Automated underwriting workflow
let auto_underwriting = algebra.compose(
    &underwriting,
    AlgebraOperation::Sequence(vec![
        // Income verification
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.aggregate = "income-analysis".to_string();
            new_parts.event_type = "calculating".to_string();
            new_parts
        })),
        // Asset verification
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.aggregate = "asset-analysis".to_string();
            new_parts.event_type = "verifying".to_string();
            new_parts
        })),
        // Credit analysis
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.aggregate = "credit-analysis".to_string();
            new_parts.event_type = "scoring".to_string();
            new_parts
        })),
        // Decision routing
        Box::new(AlgebraOperation::Choice(vec![
            // Auto-approve
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "decision".to_string();
                new_parts.event_type = "approved".to_string();
                new_parts
            })),
            // Auto-decline
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "decision".to_string();
                new_parts.event_type = "declined".to_string();
                new_parts
            })),
            // Manual review
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "manual-review".to_string();
                new_parts.event_type = "required".to_string();
                new_parts
            })),
        ])),
    ])
)?;

// Explainability tracking
let decision_audit = algebra.compose(
    &Subject::new("lending.underwriting.decision.approved.v1")?,
    AlgebraOperation::Parallel(vec![
        // Log decision factors
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.audit".to_string();
            new_parts.aggregate = "decision-log".to_string();
            new_parts
        })),
        // Update ML models
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.ml".to_string();
            new_parts.aggregate = "training-data".to_string();
            new_parts
        })),
    ])
)?;
```

### User Story 4.2: Exception Handling and Overrides
**As a** Senior Underwriter
**I want to** review and override automated decisions when necessary
**So that** we can handle unique situations appropriately

**Acceptance Criteria:**
- Exception queues route to appropriate underwriters
- Override reasons are documented
- Patterns in overrides update automation rules
- Approval hierarchies are enforced

**Subject Algebra Usage:**
```rust
// Exception routing
let exception = Subject::new("lending.underwriting.exception.identified.v1")?;

// Permission-based routing
let underwriter_permissions = Permissions::new()
    .add_rule(PermissionRule::new()
        .allow("lending.underwriting.exception.*.review.*")
        .require_attribute("role", "senior_underwriter")
        .require_attribute("loan_amount_limit", "5000000"))
    .add_rule(PermissionRule::new()
        .allow("lending.underwriting.exception.*.override.*")
        .require_attribute("role", "chief_underwriter"));

// Exception handling workflow
let exception_workflow = algebra.compose(
    &exception,
    AlgebraOperation::Sequence(vec![
        // Categorize exception
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.event_type = "categorized".to_string();
            new_parts
        })),
        // Route by complexity
        Box::new(AlgebraOperation::Choice(vec![
            // Simple override
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "simple-override".to_string();
                new_parts
            })),
            // Complex review
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "complex-review".to_string();
                new_parts
            })),
            // Committee decision
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "committee-review".to_string();
                new_parts
            })),
        ])),
    ])
)?;
```

## Epic 5: Closing and Post-Closing

### Background
The platform coordinates loan closing activities across multiple parties and manages post-closing quality control and document delivery.

### User Story 5.1: Closing Coordination
**As a** Closing Coordinator
**I want to** orchestrate closing activities across all parties
**So that** loans close on time with all requirements met

**Acceptance Criteria:**
- Closing tasks are tracked and routed appropriately
- Document preparation triggers automatically
- Funding is coordinated with lenders
- All parties receive timely notifications

**Subject Algebra Usage:**
```rust
// Closing initiation
let closing = Subject::new("lending.closing.loan.scheduled.v1")?;

// Closing coordination workflow
let closing_workflow = algebra.compose(
    &closing,
    AlgebraOperation::Parallel(vec![
        // Document preparation
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.docprep".to_string();
            new_parts.aggregate = "closing-package".to_string();
            new_parts.event_type = "generating".to_string();
            new_parts
        })),
        // Title company coordination
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.title".to_string();
            new_parts.aggregate = "closing-order".to_string();
            new_parts.event_type = "scheduled".to_string();
            new_parts
        })),
        // Funding coordination
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.funding".to_string();
            new_parts.aggregate = "wire-transfer".to_string();
            new_parts.event_type = "preparing".to_string();
            new_parts
        })),
        // Borrower notification
        Box::new(AlgebraOperation::Transform(|parts| {
            let mut new_parts = parts.clone();
            new_parts.context = "lending.notifications".to_string();
            new_parts.aggregate = "borrower-comm".to_string();
            new_parts.event_type = "closing-reminder".to_string();
            new_parts
        })),
    ])
)?;
```

### User Story 5.2: Post-Closing Quality Control
**As a** QC Manager
**I want to** perform quality control on closed loans
**So that** we identify and correct any issues before selling loans

**Acceptance Criteria:**
- Closed loans are sampled for QC review
- Document completeness is verified
- Data accuracy is validated
- Issues trigger remediation workflows

**Subject Algebra Usage:**
```rust
// Post-closing QC
let qc_review = Subject::new("lending.qc.loan.review.initiated.v1")?;

// QC workflow with sampling
let qc_workflow = algebra.compose(
    &qc_review,
    AlgebraOperation::Sequence(vec![
        // Document review
        Box::new(AlgebraOperation::Parallel(vec![
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "document-audit".to_string();
                new_parts
            })),
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "data-validation".to_string();
                new_parts
            })),
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "compliance-check".to_string();
                new_parts
            })),
        ])),
        // Issue identification
        Box::new(AlgebraOperation::Choice(vec![
            // Clean file
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.event_type = "passed".to_string();
                new_parts
            })),
            // Minor issues
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "remediation".to_string();
                new_parts.event_type = "minor-issues".to_string();
                new_parts
            })),
            // Major issues
            Box::new(AlgebraOperation::Transform(|parts| {
                let mut new_parts = parts.clone();
                new_parts.aggregate = "escalation".to_string();
                new_parts.event_type = "major-issues".to_string();
                new_parts
            })),
        ])),
    ])
)?;
```

## Testing Patterns

### Pattern 1: Document Validation Testing
```rust
#[test]
fn test_document_validation_routing() {
    let algebra = SubjectAlgebra::new();
    let paystub = Subject::new("lending.broker.income.document.paystub.received.v1").unwrap();

    let workflow = compose_validation_workflow(&algebra, &paystub);

    // Should route to income verification
    assert!(workflow.contains_step("income-verification"));
    // Should trigger OCR
    assert!(workflow.contains_step("ocr-requested"));
}
```

### Pattern 2: Multi-Lender Rate Shopping
```rust
#[test]
fn test_rate_shopping_broadcast() {
    let rate_request = Subject::new("lending.rates.quote.jumbo.requested.v1").unwrap();
    let qualified_lenders = identify_qualified_lenders(&rate_request);

    // Should identify jumbo lenders
    assert!(qualified_lenders.contains("lending.jumbo-specialist"));
    assert!(!qualified_lenders.contains("lending.conforming-only"));
}
```

### Pattern 3: Compliance Routing
```rust
#[test]
fn test_state_specific_compliance() {
    let ny_loan = Subject::new("lending.application.ny.loan.submitted.v1").unwrap();
    let tx_loan = Subject::new("lending.application.tx.loan.submitted.v1").unwrap();

    let ny_workflow = compose_compliance_workflow(&ny_loan);
    let tx_workflow = compose_compliance_workflow(&tx_loan);

    // Different states have different requirements
    assert!(ny_workflow.contains("cema-review"));
    assert!(tx_workflow.contains("homestead-verification"));
}
```

## Summary

These user stories demonstrate how CIM Subject algebra enables sophisticated private mortgage lending workflows:

1. **Multi-Party Coordination** - Brokers, lenders, vendors, and service providers collaborate through subject-based routing
2. **Document Intelligence** - OCR, validation, and cross-referencing through composed workflows
3. **Regulatory Compliance** - State-specific rules and audit trails through permission-based routing
4. **Dynamic Pricing** - Real-time rate shopping and risk-based pricing adjustments
5. **Quality Control** - Automated QC with exception handling and remediation workflows

The subject algebra provides the foundation for building a flexible, compliant, and efficient mortgage lending platform that can adapt to changing regulations and business requirements.
