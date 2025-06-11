//! Core subject types and operations

use crate::error::{Result, SubjectError};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;

/// A NATS subject representing a hierarchical address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subject {
    /// The raw subject string
    raw: String,
    /// Parsed components
    parts: SubjectParts,
}

impl Subject {
    /// Create a new subject from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the subject format is invalid
    pub fn new(subject: impl Into<String>) -> Result<Self> {
        let raw = subject.into();
        let parts = SubjectParts::parse(&raw)?;
        Ok(Self { raw, parts })
    }

    /// Create a subject from pre-parsed parts
    pub fn from_parts(parts: SubjectParts) -> Self {
        let raw = parts.to_string();
        Self { raw, parts }
    }

    /// Get the raw subject string
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Get the parsed parts
    pub fn parts(&self) -> &SubjectParts {
        &self.parts
    }

    /// Decompose into parts
    pub fn into_parts(self) -> SubjectParts {
        self.parts
    }

    /// Get the context component
    pub fn context(&self) -> &str {
        &self.parts.context
    }

    /// Get the aggregate component
    pub fn aggregate(&self) -> &str {
        &self.parts.aggregate
    }

    /// Get the event type component
    pub fn event_type(&self) -> &str {
        &self.parts.event_type
    }

    /// Get the version component
    pub fn version(&self) -> &str {
        &self.parts.version
    }

    /// Create a new subject with a different event type
    pub fn with_event_type(&self, event_type: impl Into<String>) -> Self {
        let mut parts = self.parts.clone();
        parts.event_type = event_type.into();
        Self::from_parts(parts)
    }

    /// Create a new subject with a different version
    pub fn with_version(&self, version: impl Into<String>) -> Self {
        let mut parts = self.parts.clone();
        parts.version = version.into();
        Self::from_parts(parts)
    }
}

impl Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl FromStr for Subject {
    type Err = SubjectError;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

impl AsRef<str> for Subject {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

/// Components of a parsed subject
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubjectParts {
    /// Bounded context name (e.g., "people", "organizations")
    pub context: String,
    /// Aggregate root type (e.g., "person", "company")
    pub aggregate: String,
    /// Event type (e.g., "created", "updated", "deleted")
    pub event_type: String,
    /// Schema version (e.g., "v1", "v2")
    pub version: String,
}

impl SubjectParts {
    /// Create new subject parts
    pub fn new(
        context: impl Into<String>,
        aggregate: impl Into<String>,
        event_type: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            context: context.into(),
            aggregate: aggregate.into(),
            event_type: event_type.into(),
            version: version.into(),
        }
    }

    /// Parse a subject string into its components
    ///
    /// Expected format: `<context>.<aggregate>.<event_type>.<version>`
    pub fn parse(subject: &str) -> Result<Self> {
        let parts: Vec<&str> = subject.split('.').collect();

        if parts.len() != 4 {
            return Err(SubjectError::invalid_format(format!(
                "Subject must have exactly 4 parts separated by dots, got {}: '{}'",
                parts.len(),
                subject
            )));
        }

        // Validate each part
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                return Err(SubjectError::invalid_format(format!(
                    "Subject part {} cannot be empty in '{}'",
                    i + 1,
                    subject
                )));
            }
            if !part.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                return Err(SubjectError::invalid_format(format!(
                    "Subject part '{}' contains invalid characters in '{}'",
                    part, subject
                )));
            }
        }

        Ok(Self {
            context: parts[0].to_string(),
            aggregate: parts[1].to_string(),
            event_type: parts[2].to_string(),
            version: parts[3].to_string(),
        })
    }

    /// Convert back to a subject string
    pub fn to_subject(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.context, self.aggregate, self.event_type, self.version
        )
    }
}

impl Display for SubjectParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_subject())
    }
}

impl FromStr for SubjectParts {
    type Err = SubjectError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

/// Builder for constructing subjects
#[derive(Debug, Clone, Default)]
pub struct SubjectBuilder {
    context: Option<String>,
    aggregate: Option<String>,
    event_type: Option<String>,
    version: Option<String>,
}

impl SubjectBuilder {
    /// Create a new subject builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the context
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Set the aggregate
    pub fn aggregate(mut self, aggregate: impl Into<String>) -> Self {
        self.aggregate = Some(aggregate.into());
        self
    }

    /// Set the event type
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    /// Set the version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Build the subject
    ///
    /// # Errors
    ///
    /// Returns an error if any required component is missing
    pub fn build(self) -> Result<Subject> {
        let context = self
            .context
            .ok_or_else(|| SubjectError::validation_error("Context is required"))?;
        let aggregate = self
            .aggregate
            .ok_or_else(|| SubjectError::validation_error("Aggregate is required"))?;
        let event_type = self
            .event_type
            .ok_or_else(|| SubjectError::validation_error("Event type is required"))?;
        let version = self
            .version
            .ok_or_else(|| SubjectError::validation_error("Version is required"))?;

        let parts = SubjectParts::new(context, aggregate, event_type, version);
        Ok(Subject::from_parts(parts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_parsing() {
        let subject = Subject::new("people.person.created.v1").unwrap();
        assert_eq!(subject.context(), "people");
        assert_eq!(subject.aggregate(), "person");
        assert_eq!(subject.event_type(), "created");
        assert_eq!(subject.version(), "v1");
        assert_eq!(subject.as_str(), "people.person.created.v1");
    }

    #[test]
    fn test_subject_parts() {
        let parts = SubjectParts::new("orders", "order", "placed", "v2");
        assert_eq!(parts.to_subject(), "orders.order.placed.v2");

        let parsed = SubjectParts::parse("orders.order.placed.v2").unwrap();
        assert_eq!(parsed, parts);
    }

    #[test]
    fn test_invalid_subjects() {
        // Too few parts
        assert!(Subject::new("people.person").is_err());

        // Too many parts
        assert!(Subject::new("people.person.created.v1.extra").is_err());

        // Empty part
        assert!(Subject::new("people..created.v1").is_err());

        // Invalid characters
        assert!(Subject::new("people.per$on.created.v1").is_err());
    }

    #[test]
    fn test_subject_builder() {
        let subject = SubjectBuilder::new()
            .context("inventory")
            .aggregate("product")
            .event_type("restocked")
            .version("v1")
            .build()
            .unwrap();

        assert_eq!(subject.as_str(), "inventory.product.restocked.v1");
    }

    #[test]
    fn test_subject_builder_missing_fields() {
        let result = SubjectBuilder::new()
            .context("inventory")
            .aggregate("product")
            // Missing event_type and version
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_subject_modifications() {
        let subject = Subject::new("users.user.created.v1").unwrap();

        let updated = subject.with_event_type("updated");
        assert_eq!(updated.as_str(), "users.user.updated.v1");

        let v2 = subject.with_version("v2");
        assert_eq!(v2.as_str(), "users.user.created.v2");
    }
}
