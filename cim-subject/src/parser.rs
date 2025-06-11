//! Subject parser with custom parsing rules

use crate::error::{Result, SubjectError};
use crate::subject::{Subject, SubjectParts};
use dashmap::DashMap;
use std::sync::Arc;

/// Parser for subjects with custom rules
#[derive(Clone)]
pub struct SubjectParser {
    /// Custom parsing rules by context
    rules: Arc<DashMap<String, ParseRule>>,
    /// Validation rules
    validators: Arc<DashMap<String, ValidationRule>>,
}

impl Default for SubjectParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SubjectParser {
    /// Create a new subject parser
    pub fn new() -> Self {
        Self {
            rules: Arc::new(DashMap::new()),
            validators: Arc::new(DashMap::new()),
        }
    }

    /// Register a custom parsing rule for a context
    pub fn register_rule(&self, context: impl Into<String>, rule: ParseRule) {
        self.rules.insert(context.into(), rule);
    }

    /// Register a validation rule
    pub fn register_validator(&self, name: impl Into<String>, validator: ValidationRule) {
        self.validators.insert(name.into(), validator);
    }

    /// Parse a subject string
    pub fn parse(&self, subject: &str) -> Result<Subject> {
        // Extract the context (first part) to check for custom rules
        let parts: Vec<&str> = subject.split('.').collect();
        if parts.is_empty() {
            return Err(SubjectError::invalid_format("Empty subject"));
        }

        let context = parts[0];

        // Check for custom parsing rules for this context
        if let Some(rule) = self.rules.get(context) {
            let custom_parts = rule.parse(subject)?;
            // Validate the parsed subject
            self.validate(&custom_parts)?;
            return Ok(Subject::from_parts(custom_parts));
        }

        // Fall back to standard parsing
        let standard_parts = SubjectParts::parse(subject)?;

        // Validate the parsed subject
        self.validate(&standard_parts)?;

        Ok(Subject::from_parts(standard_parts))
    }

    /// Validate subject parts
    fn validate(&self, parts: &SubjectParts) -> Result<()> {
        // Run all validators
        for validator in self.validators.iter() {
            validator.validate(parts)?;
        }
        Ok(())
    }

    /// Create a parser with standard rules
    pub fn with_standard_rules() -> Self {
        let parser = Self::new();

        // Add standard validation rules
        parser.register_validator(
            "version_format",
            ValidationRule::new(
                "Version Format",
                Arc::new(|parts| {
                    if !parts.version.starts_with('v') {
                        return Err(SubjectError::validation_error(
                            "Version must start with 'v'",
                        ));
                    }
                    Ok(())
                }),
            ),
        );

        parser.register_validator(
            "context_length",
            ValidationRule::new(
                "Context Length",
                Arc::new(|parts| {
                    if parts.context.len() > 32 {
                        return Err(SubjectError::validation_error(
                            "Context name too long (max 32 chars)",
                        ));
                    }
                    Ok(())
                }),
            ),
        );

        parser
    }
}

/// A custom parsing rule
#[derive(Clone)]
pub struct ParseRule {
    /// Name of the rule
    pub name: String,
    /// Description
    pub description: String,
    /// Parser function
    pub parser: Arc<dyn Fn(&str) -> Result<SubjectParts> + Send + Sync>,
}

impl ParseRule {
    /// Create a new parse rule
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parser: Arc<dyn Fn(&str) -> Result<SubjectParts> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parser,
        }
    }

    /// Parse a subject using this rule
    pub fn parse(&self, subject: &str) -> Result<SubjectParts> {
        (self.parser)(subject)
    }
}

/// A validation rule
#[derive(Clone)]
pub struct ValidationRule {
    /// Name of the rule
    pub name: String,
    /// Validator function
    pub validator: Arc<dyn Fn(&SubjectParts) -> Result<()> + Send + Sync>,
}

impl ValidationRule {
    /// Create a new validation rule
    pub fn new(
        name: impl Into<String>,
        validator: Arc<dyn Fn(&SubjectParts) -> Result<()> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.into(),
            validator,
        }
    }

    /// Validate subject parts
    pub fn validate(&self, parts: &SubjectParts) -> Result<()> {
        (self.validator)(parts)
    }
}

/// Builder for creating parsers with rules
#[derive(Default)]
pub struct ParserBuilder {
    rules: Vec<(String, ParseRule)>,
    validators: Vec<(String, ValidationRule)>,
}

impl ParserBuilder {
    /// Create a new parser builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a parsing rule
    pub fn with_rule(mut self, context: impl Into<String>, rule: ParseRule) -> Self {
        self.rules.push((context.into(), rule));
        self
    }

    /// Add a validation rule
    pub fn with_validator(mut self, name: impl Into<String>, validator: ValidationRule) -> Self {
        self.validators.push((name.into(), validator));
        self
    }

    /// Add a simple context rule that allows flexible formats
    pub fn with_flexible_context(mut self, context: impl Into<String>) -> Self {
        let ctx = context.into();
        let rule = ParseRule::new(
            format!("{}_flexible", ctx),
            format!("Flexible parsing for {} context", ctx),
            Arc::new(move |subject| {
                let parts: Vec<&str> = subject.split('.').collect();
                if parts.len() < 3 {
                    return Err(SubjectError::invalid_format(
                        "Flexible format requires at least 3 parts",
                    ));
                }

                // Allow variable number of middle parts
                let context = parts[0];
                let aggregate = parts[1..parts.len() - 2].join(".");
                let event_type = parts[parts.len() - 2];
                let version = parts[parts.len() - 1];

                Ok(SubjectParts::new(context, aggregate, event_type, version))
            }),
        );

        self.rules.push((ctx, rule));
        self
    }

    /// Build the parser
    pub fn build(self) -> SubjectParser {
        let parser = SubjectParser::new();

        for (context, rule) in self.rules {
            parser.register_rule(context, rule);
        }

        for (name, validator) in self.validators {
            parser.register_validator(name, validator);
        }

        parser
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_parser() {
        let parser = SubjectParser::with_standard_rules();

        // Valid subject
        let subject = parser.parse("users.person.created.v1").unwrap();
        assert_eq!(subject.context(), "users");

        // Invalid version format
        let result = parser.parse("users.person.created.1");
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_rule() {
        let parser = ParserBuilder::new()
            .with_rule(
                "workflow",
                ParseRule::new(
                    "workflow_parser",
                    "Custom parser for workflow subjects",
                    Arc::new(|subject| {
                        // Custom format: workflow.<id>.<step>.<status>
                        let parts: Vec<&str> = subject.split('.').collect();
                        if parts.len() != 4 || parts[0] != "workflow" {
                            return Err(SubjectError::invalid_format("Not a workflow subject"));
                        }

                        Ok(SubjectParts::new(
                            "workflow",
                            parts[1], // workflow ID as aggregate
                            format!("{}_{}", parts[2], parts[3]), // step_status as event
                            "v1",
                        ))
                    }),
                ),
            )
            .build();

        let subject = parser.parse("workflow.order123.validation.completed").unwrap();
        assert_eq!(subject.aggregate(), "order123");
        assert_eq!(subject.event_type(), "validation_completed");
    }

    #[test]
    fn test_flexible_context() {
        let parser = ParserBuilder::new()
            .with_flexible_context("graph")
            .build();

        // Standard format still works
        let s1 = parser.parse("graph.node.created.v1").unwrap();
        assert_eq!(s1.aggregate(), "node");

        // Flexible format with nested aggregate
        let s2 = parser.parse("graph.workflow.step.node.updated.v2").unwrap();
        assert_eq!(s2.aggregate(), "workflow.step.node");
        assert_eq!(s2.event_type(), "updated");
        assert_eq!(s2.version(), "v2");
    }

    #[test]
    fn test_validation_rules() {
        let parser = ParserBuilder::new()
            .with_validator(
                "no_test_context",
                ValidationRule::new("No Test Context", Arc::new(|parts| {
                    if parts.context == "test" {
                        return Err(SubjectError::validation_error(
                            "Test context not allowed in production",
                        ));
                    }
                    Ok(())
                })),
            )
            .build();

        // Normal subject passes
        assert!(parser.parse("users.person.created.v1").is_ok());

        // Test context fails validation
        assert!(parser.parse("test.entity.created.v1").is_err());
    }
}
