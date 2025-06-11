//! Subject Algebra - compositional operations on subjects

use crate::error::{Result, SubjectError};
use crate::pattern::Pattern;
use crate::subject::{Subject, SubjectParts};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// The Subject Algebra system for compositional operations
#[derive(Clone)]
pub struct SubjectAlgebra {
    /// Registered composition rules
    rules: Arc<DashMap<String, CompositionRule>>,
    /// Registered transformations
    transformations: Arc<DashMap<String, Transformation>>,
}

impl Default for SubjectAlgebra {
    fn default() -> Self {
        Self::new()
    }
}

impl SubjectAlgebra {
    /// Create a new Subject Algebra instance
    pub fn new() -> Self {
        Self {
            rules: Arc::new(DashMap::new()),
            transformations: Arc::new(DashMap::new()),
        }
    }

    /// Register a composition rule
    pub fn register_rule(&self, name: impl Into<String>, rule: CompositionRule) {
        self.rules.insert(name.into(), rule);
    }

    /// Register a transformation
    pub fn register_transformation(&self, name: impl Into<String>, transform: Transformation) {
        self.transformations.insert(name.into(), transform);
    }

    /// Compose two subjects using a specific operation
    pub fn compose(
        &self,
        left: &Subject,
        right: &Subject,
        operation: AlgebraOperation,
    ) -> Result<Subject> {
        match operation {
            AlgebraOperation::Sequence => self.sequence(left, right),
            AlgebraOperation::Parallel => self.parallel(left, right),
            AlgebraOperation::Choice { condition } => self.choice(left, right, &condition),
            AlgebraOperation::Transform { name } => self.transform(left, &name),
            AlgebraOperation::Project { fields } => self.project(left, &fields),
            AlgebraOperation::Inject { context } => self.inject(left, &context),
        }
    }

    /// Sequential composition: left happens before right
    fn sequence(&self, left: &Subject, right: &Subject) -> Result<Subject> {
        // Create a new subject representing the sequence
        let parts = SubjectParts::new(
            format!("{}-{}", left.context(), right.context()),
            format!("{}-{}", left.aggregate(), right.aggregate()),
            "sequenced",
            "v1",
        );
        Ok(Subject::from_parts(parts))
    }

    /// Parallel composition: left and right happen concurrently
    fn parallel(&self, left: &Subject, right: &Subject) -> Result<Subject> {
        let parts = SubjectParts::new(
            format!("{}+{}", left.context(), right.context()),
            format!("{}+{}", left.aggregate(), right.aggregate()),
            "parallel",
            "v1",
        );
        Ok(Subject::from_parts(parts))
    }

    /// Choice composition: choose left or right based on condition
    fn choice(&self, left: &Subject, right: &Subject, condition: &str) -> Result<Subject> {
        let parts = SubjectParts::new(
            left.context(), // Use left's context as primary
            format!("{}|{}", left.aggregate(), right.aggregate()),
            format!("choice_{}", condition),
            "v1",
        );
        Ok(Subject::from_parts(parts))
    }

    /// Transform a subject using a named transformation
    fn transform(&self, subject: &Subject, transform_name: &str) -> Result<Subject> {
        let transform = self
            .transformations
            .get(transform_name)
            .ok_or_else(|| SubjectError::not_found(format!("Transformation '{}'", transform_name)))?;

        transform.apply(subject)
    }

    /// Project specific fields from a subject
    fn project(&self, subject: &Subject, fields: &[String]) -> Result<Subject> {
        // For now, we'll create a projected version
        let parts = SubjectParts::new(
            subject.context(),
            subject.aggregate(),
            format!("projected_{}", fields.join("_")),
            subject.version(),
        );
        Ok(Subject::from_parts(parts))
    }

    /// Inject a subject into a different context
    fn inject(&self, subject: &Subject, new_context: &str) -> Result<Subject> {
        let parts = SubjectParts::new(
            new_context,
            subject.aggregate(),
            subject.event_type(),
            subject.version(),
        );
        Ok(Subject::from_parts(parts))
    }

    /// Find all subjects matching a pattern
    pub fn find_matching(&self, pattern: &Pattern, subjects: &[Subject]) -> Vec<Subject> {
        subjects
            .iter()
            .filter(|s| pattern.matches(s))
            .cloned()
            .collect()
    }

    /// Create a subject lattice (partial order)
    pub fn create_lattice(&self, subjects: &[Subject]) -> SubjectLattice {
        SubjectLattice::new(subjects)
    }
}

/// Algebraic operations on subjects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgebraOperation {
    /// Sequential composition (happens-before)
    Sequence,
    /// Parallel composition (concurrent)
    Parallel,
    /// Choice between subjects based on condition
    Choice {
        /// The condition that determines which subject to choose
        condition: String
    },
    /// Transform subject using named transformation
    Transform {
        /// The name of the transformation to apply
        name: String
    },
    /// Project specific fields from subject
    Project {
        /// The fields to project from the subject
        fields: Vec<String>
    },
    /// Inject additional context into subject
    Inject {
        /// The context to inject into the subject
        context: String
    },
}

/// A composition rule defines how subjects can be composed
#[derive(Clone)]
pub struct CompositionRule {
    /// Name of the rule
    pub name: String,
    /// Pattern for left operand
    pub left_pattern: Pattern,
    /// Pattern for right operand
    pub right_pattern: Pattern,
    /// Function to compose subjects
    pub composer: Arc<dyn Fn(&Subject, &Subject) -> Result<Subject> + Send + Sync>,
}

/// A transformation on subjects
#[derive(Clone)]
pub struct Transformation {
    /// Name of the transformation
    pub name: String,
    /// Input pattern
    pub input_pattern: Pattern,
    /// Transformation function
    pub transform: Arc<dyn Fn(&Subject) -> Result<Subject> + Send + Sync>,
}

impl Transformation {
    /// Apply the transformation to a subject
    pub fn apply(&self, subject: &Subject) -> Result<Subject> {
        if !self.input_pattern.matches(subject) {
            return Err(SubjectError::validation_error(format!(
                "Subject '{}' does not match transformation pattern '{}'",
                subject,
                self.input_pattern
            )));
        }
        (self.transform)(subject)
    }
}

/// A lattice structure for subjects (partial order)
#[derive(Debug, Clone)]
pub struct SubjectLattice {
    /// Subjects in the lattice
    subjects: Vec<Subject>,
    /// Ordering relationships
    ordering: Vec<(usize, usize)>, // (less_idx, greater_idx)
}

impl SubjectLattice {
    /// Create a new subject lattice
    pub fn new(subjects: &[Subject]) -> Self {
        let mut lattice = Self {
            subjects: subjects.to_vec(),
            ordering: Vec::new(),
        };
        lattice.compute_ordering();
        lattice
    }

    /// Compute the ordering relationships
    fn compute_ordering(&mut self) {
        // Simple ordering based on specificity
        for i in 0..self.subjects.len() {
            for j in 0..self.subjects.len() {
                if i != j && self.is_less_specific(i, j) {
                    self.ordering.push((i, j));
                }
            }
        }
    }

    /// Check if subject at index i is less specific than j
    fn is_less_specific(&self, i: usize, j: usize) -> bool {
        let si = &self.subjects[i];
        let sj = &self.subjects[j];

        // Context hierarchy
        if si.context() != sj.context() {
            return false;
        }

        // Event type generalization
        matches!(
            (si.event_type(), sj.event_type()),
            ("*", _) | ("changed", "created" | "updated" | "deleted")
        )
    }

    /// Find the join (least upper bound) of two subjects
    pub fn join(&self, a: &Subject, b: &Subject) -> Option<Subject> {
        // Find common ancestors
        let a_idx = self.subjects.iter().position(|s| s == a)?;
        let b_idx = self.subjects.iter().position(|s| s == b)?;

        // Find minimal common ancestors
        for (i, subject) in self.subjects.iter().enumerate() {
            if self.is_ancestor(a_idx, i) && self.is_ancestor(b_idx, i) {
                return Some(subject.clone());
            }
        }
        None
    }

    /// Check if a is an ancestor of b in the ordering
    fn is_ancestor(&self, a: usize, b: usize) -> bool {
        self.ordering.iter().any(|(x, y)| *x == a && *y == b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_composition() {
        let algebra = SubjectAlgebra::new();
        let left = Subject::new("orders.order.created.v1").unwrap();
        let right = Subject::new("inventory.stock.reserved.v1").unwrap();

        let result = algebra
            .compose(&left, &right, AlgebraOperation::Sequence)
            .unwrap();

        assert_eq!(result.context(), "orders-inventory");
        assert_eq!(result.aggregate(), "order-stock");
        assert_eq!(result.event_type(), "sequenced");
    }

    #[test]
    fn test_parallel_composition() {
        let algebra = SubjectAlgebra::new();
        let left = Subject::new("users.user.created.v1").unwrap();
        let right = Subject::new("emails.welcome.sent.v1").unwrap();

        let result = algebra
            .compose(&left, &right, AlgebraOperation::Parallel)
            .unwrap();

        assert_eq!(result.context(), "users+emails");
        assert_eq!(result.aggregate(), "user+welcome");
        assert_eq!(result.event_type(), "parallel");
    }

    #[test]
    fn test_inject_operation() {
        let algebra = SubjectAlgebra::new();
        let subject = Subject::new("internal.user.created.v1").unwrap();

        let result = algebra
            .compose(
                &subject,
                &subject, // Right operand ignored for inject
                AlgebraOperation::Inject {
                    context: "public".to_string(),
                },
            )
            .unwrap();

        assert_eq!(result.context(), "public");
        assert_eq!(result.aggregate(), "user");
        assert_eq!(result.event_type(), "created");
    }

    #[test]
    fn test_transformation() {
        let algebra = SubjectAlgebra::new();

        // Register a transformation
        let transform = Transformation {
            name: "anonymize".to_string(),
            input_pattern: Pattern::new("users.*.*.v1").unwrap(),
            transform: Arc::new(|subject| {
                Ok(Subject::from_parts(SubjectParts::new(
                    subject.context(),
                    "anonymous",
                    subject.event_type(),
                    subject.version(),
                )))
            }),
        };

        algebra.register_transformation("anonymize", transform);

        let subject = Subject::new("users.person.created.v1").unwrap();
        let result = algebra
            .compose(
                &subject,
                &subject,
                AlgebraOperation::Transform {
                    name: "anonymize".to_string(),
                },
            )
            .unwrap();

        assert_eq!(result.aggregate(), "anonymous");
    }

    #[test]
    fn test_subject_lattice() {
        let subjects = vec![
            Subject::new("events.base.changed.v1").unwrap(),
            Subject::new("events.base.created.v1").unwrap(),
            Subject::new("events.base.updated.v1").unwrap(),
        ];

        let algebra = SubjectAlgebra::new();
        let lattice = algebra.create_lattice(&subjects);

        // The lattice should recognize "changed" as more general
        assert!(!lattice.ordering.is_empty());
    }
}
