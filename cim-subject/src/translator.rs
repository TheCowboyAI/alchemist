//! Subject translation between different schemas

use crate::error::{Result, SubjectError};
use crate::pattern::Pattern;
use crate::subject::{Subject, SubjectParts};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Translator for converting subjects between different schemas
#[derive(Clone)]
pub struct Translator {
    /// Translation rules
    rules: Arc<DashMap<String, TranslationRule>>,
    /// Reverse translation cache
    reverse_cache: Arc<DashMap<String, String>>,
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

impl Translator {
    /// Create a new translator
    pub fn new() -> Self {
        Self {
            rules: Arc::new(DashMap::new()),
            reverse_cache: Arc::new(DashMap::new()),
        }
    }

    /// Register a translation rule
    pub fn register_rule(&self, name: impl Into<String>, rule: TranslationRule) {
        self.rules.insert(name.into(), rule);
    }

    /// Translate a subject using registered rules
    pub fn translate(&self, subject: &Subject) -> Result<Subject> {
        // Find matching rule
        for rule in self.rules.iter() {
            if rule.matches_source(subject) {
                return rule.translate(subject);
            }
        }

        // No rule found, return original
        Ok(subject.clone())
    }

    /// Reverse translate a subject
    pub fn reverse_translate(&self, subject: &Subject) -> Result<Subject> {
        // Check cache first
        if let Some(original) = self.reverse_cache.get(subject.as_str()) {
            return Subject::new(original.clone());
        }

        // Find matching reverse rule
        for rule in self.rules.iter() {
            if rule.matches_target(subject) {
                return rule.reverse_translate(subject);
            }
        }

        // No rule found, return original
        Ok(subject.clone())
    }

    /// Create a bidirectional translator
    pub fn bidirectional(
        forward_rules: Vec<TranslationRule>,
        reverse_rules: Vec<TranslationRule>,
    ) -> Self {
        let translator = Self::new();

        for (i, rule) in forward_rules.into_iter().enumerate() {
            translator.register_rule(format!("forward_{}", i), rule);
        }

        for (i, rule) in reverse_rules.into_iter().enumerate() {
            translator.register_rule(format!("reverse_{}", i), rule);
        }

        translator
    }
}

/// A translation rule
#[derive(Clone)]
pub struct TranslationRule {
    /// Name of the rule
    pub name: String,
    /// Source pattern
    pub source_pattern: Pattern,
    /// Target pattern (optional, for validation)
    pub target_pattern: Option<Pattern>,
    /// Translation function
    pub translate_fn: Arc<dyn Fn(&Subject) -> Result<Subject> + Send + Sync>,
    /// Reverse translation function (optional)
    pub reverse_fn: Option<Arc<dyn Fn(&Subject) -> Result<Subject> + Send + Sync>>,
}

impl TranslationRule {
    /// Create a new translation rule
    pub fn new(
        name: impl Into<String>,
        source_pattern: Pattern,
        translate_fn: Arc<dyn Fn(&Subject) -> Result<Subject> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.into(),
            source_pattern,
            target_pattern: None,
            translate_fn,
            reverse_fn: None,
        }
    }

    /// Add a target pattern for validation
    pub fn with_target_pattern(mut self, pattern: Pattern) -> Self {
        self.target_pattern = Some(pattern);
        self
    }

    /// Add a reverse translation function
    pub fn with_reverse(
        mut self,
        reverse_fn: Arc<dyn Fn(&Subject) -> Result<Subject> + Send + Sync>,
    ) -> Self {
        self.reverse_fn = Some(reverse_fn);
        self
    }

    /// Check if this rule matches a source subject
    pub fn matches_source(&self, subject: &Subject) -> bool {
        self.source_pattern.matches(subject)
    }

    /// Check if this rule matches a target subject
    pub fn matches_target(&self, subject: &Subject) -> bool {
        self.target_pattern
            .as_ref()
            .map(|p| p.matches(subject))
            .unwrap_or(false)
    }

    /// Translate a subject
    pub fn translate(&self, subject: &Subject) -> Result<Subject> {
        let result = (self.translate_fn)(subject)?;

        // Validate against target pattern if provided
        if let Some(target_pattern) = &self.target_pattern {
            if !target_pattern.matches(&result) {
                return Err(SubjectError::translation_error(format!(
                    "Translation result '{}' does not match target pattern '{}'",
                    result, target_pattern
                )));
            }
        }

        Ok(result)
    }

    /// Reverse translate a subject
    pub fn reverse_translate(&self, subject: &Subject) -> Result<Subject> {
        if let Some(reverse_fn) = &self.reverse_fn {
            (reverse_fn)(subject)
        } else {
            Err(SubjectError::translation_error(
                "No reverse translation available",
            ))
        }
    }
}

/// Trait for types that can translate messages
pub trait MessageTranslator<From, To> {
    /// Error type
    type Error;

    /// Translate from source to target
    fn translate(&self, from: From) -> std::result::Result<To, Self::Error>;

    /// Reverse translate from target to source
    fn reverse(&self, to: To) -> std::result::Result<From, Self::Error>;
}

/// Builder for creating translators
#[derive(Default)]
pub struct TranslatorBuilder {
    rules: Vec<(String, TranslationRule)>,
}

impl TranslatorBuilder {
    /// Create a new translator builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a simple mapping rule
    pub fn map(
        mut self,
        source_pattern: &str,
        target_template: &str,
    ) -> Result<Self> {
        let pattern = Pattern::new(source_pattern)?;
        let template = target_template.to_string();

        let rule = TranslationRule::new(
            format!("map_{}", source_pattern),
            pattern,
            Arc::new(move |subject| {
                // Simple template replacement
                let mut result = template.clone();
                result = result.replace("{context}", subject.context());
                result = result.replace("{aggregate}", subject.aggregate());
                result = result.replace("{event}", subject.event_type());
                result = result.replace("{version}", subject.version());
                Subject::new(result)
            }),
        );

        self.rules.push((rule.name.clone(), rule));
        Ok(self)
    }

    /// Add a context translation rule
    pub fn translate_context(
        mut self,
        from_context: &str,
        to_context: &str,
    ) -> Result<Self> {
        let pattern = Pattern::new(&format!("{}.>", from_context))?;
        let to_ctx = to_context.to_string();

        let rule = TranslationRule::new(
            format!("context_{}_{}", from_context, to_context),
            pattern,
            Arc::new(move |subject| {
                let parts = SubjectParts::new(
                    to_ctx.clone(),
                    subject.aggregate(),
                    subject.event_type(),
                    subject.version(),
                );
                Ok(Subject::from_parts(parts))
            }),
        );

        self.rules.push((rule.name.clone(), rule));
        Ok(self)
    }

    /// Add a custom translation rule
    pub fn custom(mut self, name: impl Into<String>, rule: TranslationRule) -> Self {
        self.rules.push((name.into(), rule));
        self
    }

    /// Build the translator
    pub fn build(self) -> Translator {
        let translator = Translator::new();

        for (name, rule) in self.rules {
            translator.register_rule(name, rule);
        }

        translator
    }
}

/// Schema mapping for complex translations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMapping {
    /// Name of the mapping
    pub name: String,
    /// Source schema identifier
    pub source_schema: String,
    /// Target schema identifier
    pub target_schema: String,
    /// Field mappings
    pub field_mappings: Vec<FieldMapping>,
}

/// Field mapping between schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Source field path
    pub source_path: String,
    /// Target field path
    pub target_path: String,
    /// Optional transformation
    pub transform: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_translation() {
        let translator = TranslatorBuilder::new()
            .map("internal.*.*.v1", "public.{aggregate}.{event}.v1")
            .unwrap()
            .build();

        let subject = Subject::new("internal.user.created.v1").unwrap();
        let translated = translator.translate(&subject).unwrap();

        assert_eq!(translated.as_str(), "public.user.created.v1");
    }

    #[test]
    fn test_context_translation() {
        let translator = TranslatorBuilder::new()
            .translate_context("dev", "prod")
            .unwrap()
            .translate_context("staging", "prod")
            .unwrap()
            .build();

        let dev_subject = Subject::new("dev.service.deployed.v1").unwrap();
        let prod_subject = translator.translate(&dev_subject).unwrap();

        assert_eq!(prod_subject.context(), "prod");
        assert_eq!(prod_subject.aggregate(), "service");
    }

    #[test]
    fn test_custom_translation() {
        let translator = TranslatorBuilder::new()
            .custom(
                "anonymize",
                TranslationRule::new(
                    "anonymize_user",
                    Pattern::new("users.*.*.v1").unwrap(),
                    Arc::new(|subject| {
                        let parts = SubjectParts::new(
                            "public",
                            "anonymous",
                            subject.event_type(),
                            subject.version(),
                        );
                        Ok(Subject::from_parts(parts))
                    }),
                ),
            )
            .build();

        let subject = Subject::new("users.john_doe.updated.v1").unwrap();
        let translated = translator.translate(&subject).unwrap();

        assert_eq!(translated.as_str(), "public.anonymous.updated.v1");
    }

    #[test]
    fn test_bidirectional_translation() {
        let forward = TranslationRule::new(
            "forward",
            Pattern::new("internal.>").unwrap(),
            Arc::new(|subject| {
                let parts = SubjectParts::new(
                    "external",
                    subject.aggregate(),
                    subject.event_type(),
                    subject.version(),
                );
                Ok(Subject::from_parts(parts))
            }),
        )
        .with_target_pattern(Pattern::new("external.>").unwrap())
        .with_reverse(Arc::new(|subject| {
            let parts = SubjectParts::new(
                "internal",
                subject.aggregate(),
                subject.event_type(),
                subject.version(),
            );
            Ok(Subject::from_parts(parts))
        }));

        let translator = Translator::bidirectional(vec![forward], vec![]);

        let internal = Subject::new("internal.service.started.v1").unwrap();
        let external = translator.translate(&internal).unwrap();
        assert_eq!(external.context(), "external");

        let back = translator.reverse_translate(&external).unwrap();
        assert_eq!(back.as_str(), internal.as_str());
    }

    #[test]
    fn test_no_matching_rule() {
        let translator = TranslatorBuilder::new()
            .translate_context("dev", "prod")
            .unwrap()
            .build();

        // Subject that doesn't match any rule
        let subject = Subject::new("test.service.created.v1").unwrap();
        let result = translator.translate(&subject).unwrap();

        // Should return original
        assert_eq!(result.as_str(), subject.as_str());
    }
}
