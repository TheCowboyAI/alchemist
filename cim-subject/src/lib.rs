//! # CIM Subject Algebra
//!
//! This crate provides a comprehensive Subject Algebra system for NATS-based
//! domain routing and message translation. It implements the mathematical
//! foundations for subject-based messaging patterns used throughout CIM.
//!
//! ## Core Concepts
//!
//! - **Subject**: A hierarchical address for messages (e.g., `context.aggregate.event.version`)
//! - **Pattern**: Wildcard-based subject matching using `*` and `>` operators
//! - **Algebra**: Compositional operations on subjects and patterns
//! - **Translation**: Bidirectional mapping between different subject schemas
//!
//! ## Example
//!
//! ```rust
//! use cim_subject::{Subject, Pattern, SubjectAlgebra, AlgebraOperation};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a subject
//! let subject = Subject::new("people.person.created.v1")?;
//!
//! // Create patterns
//! let pattern = Pattern::new("people.*.created.>")?;
//! assert!(pattern.matches(&subject));
//!
//! // Use algebra operations
//! let algebra = SubjectAlgebra::new();
//! let workflow_subject = Subject::new("workflow.process.triggered.v1")?;
//! let composed = algebra.compose(&subject, &workflow_subject, AlgebraOperation::Sequence)?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod algebra;
pub mod error;
pub mod parser;
pub mod pattern;
pub mod permissions;
pub mod subject;
pub mod translator;

// Re-export main types
pub use algebra::{SubjectAlgebra, AlgebraOperation, CompositionRule};
pub use error::{SubjectError, Result};
pub use parser::{SubjectParser, ParseRule};
pub use pattern::{Pattern, PatternMatcher};
pub use permissions::{Permissions, PermissionRule};
pub use subject::{Subject, SubjectParts, SubjectBuilder};
pub use translator::{Translator, TranslationRule, MessageTranslator};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        Subject, SubjectParts, SubjectBuilder,
        Pattern, PatternMatcher,
        SubjectAlgebra, AlgebraOperation,
        Permissions, PermissionRule,
        Translator, TranslationRule,
        SubjectError, Result,
    };
}
