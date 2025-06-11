//! Pattern matching for subjects with wildcard support

use crate::error::{Result, SubjectError};
use crate::subject::Subject;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;

/// A pattern for matching subjects with wildcards
///
/// Supports NATS wildcard syntax:
/// - `*` matches exactly one token
/// - `>` matches one or more tokens (must be at the end)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pattern {
    /// The raw pattern string
    raw: String,
    /// Parsed tokens
    tokens: Vec<Token>,
}

/// A token in a pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Token {
    /// Literal token that must match exactly
    Literal(String),
    /// Single wildcard (*)
    SingleWildcard,
    /// Multi wildcard (>)
    MultiWildcard,
}

impl Pattern {
    /// Create a new pattern
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid
    pub fn new(pattern: impl Into<String>) -> Result<Self> {
        let raw = pattern.into();
        let tokens = Self::parse_tokens(&raw)?;
        Ok(Self { raw, tokens })
    }

    /// Parse pattern tokens
    fn parse_tokens(pattern: &str) -> Result<Vec<Token>> {
        if pattern.is_empty() {
            return Err(SubjectError::invalid_pattern("Pattern cannot be empty"));
        }

        let parts: Vec<&str> = pattern.split('.').collect();
        let mut tokens = Vec::with_capacity(parts.len());

        for (i, part) in parts.iter().enumerate() {
            match *part {
                "" => {
                    return Err(SubjectError::invalid_pattern(format!(
                        "Empty token at position {} in pattern '{}'",
                        i + 1,
                        pattern
                    )));
                }
                "*" => tokens.push(Token::SingleWildcard),
                ">" => {
                    if i != parts.len() - 1 {
                        return Err(SubjectError::invalid_pattern(
                            "Multi-wildcard '>' can only appear at the end of a pattern",
                        ));
                    }
                    tokens.push(Token::MultiWildcard);
                }
                literal => {
                    // Validate literal token
                    if !literal
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                    {
                        return Err(SubjectError::invalid_pattern(format!(
                            "Token '{}' contains invalid characters",
                            literal
                        )));
                    }
                    tokens.push(Token::Literal(literal.to_string()));
                }
            }
        }

        Ok(tokens)
    }

    /// Check if a subject matches this pattern
    pub fn matches(&self, subject: &Subject) -> bool {
        self.matches_str(subject.as_str())
    }

    /// Check if a subject string matches this pattern
    pub fn matches_str(&self, subject: &str) -> bool {
        let subject_parts: Vec<&str> = subject.split('.').collect();
        self.matches_parts(&subject_parts)
    }

    /// Check if subject parts match this pattern
    fn matches_parts(&self, subject_parts: &[&str]) -> bool {
        let mut pattern_idx = 0;
        let mut subject_idx = 0;

        while pattern_idx < self.tokens.len() && subject_idx < subject_parts.len() {
            match &self.tokens[pattern_idx] {
                Token::MultiWildcard => {
                    // > matches everything remaining
                    return true;
                }
                Token::SingleWildcard => {
                    // * matches exactly one token
                    pattern_idx += 1;
                    subject_idx += 1;
                }
                Token::Literal(literal) => {
                    if literal != subject_parts[subject_idx] {
                        return false;
                    }
                    pattern_idx += 1;
                    subject_idx += 1;
                }
            }
        }

        // Both must be exhausted for a match
        pattern_idx == self.tokens.len() && subject_idx == subject_parts.len()
    }

    /// Get the raw pattern string
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Check if this pattern is more specific than another
    ///
    /// A pattern is more specific if it has fewer wildcards or
    /// more literal tokens before wildcards
    pub fn is_more_specific_than(&self, other: &Pattern) -> bool {
        // First, check if one has a multi-wildcard and the other doesn't
        let self_has_multi = self.tokens.iter().any(|t| matches!(t, Token::MultiWildcard));
        let other_has_multi = other.tokens.iter().any(|t| matches!(t, Token::MultiWildcard));

        // Pattern without multi-wildcard is more specific than one with
        if self_has_multi != other_has_multi {
            return !self_has_multi;
        }

        // Count single wildcards
        let self_single_wildcards = self
            .tokens
            .iter()
            .filter(|t| matches!(t, Token::SingleWildcard))
            .count();
        let other_single_wildcards = other
            .tokens
            .iter()
            .filter(|t| matches!(t, Token::SingleWildcard))
            .count();

        // Fewer single wildcards is more specific
        if self_single_wildcards != other_single_wildcards {
            return self_single_wildcards < other_single_wildcards;
        }

        // Same number of wildcards, check position of first wildcard
        let self_first_wildcard = self
            .tokens
            .iter()
            .position(|t| matches!(t, Token::SingleWildcard | Token::MultiWildcard));
        let other_first_wildcard = other
            .tokens
            .iter()
            .position(|t| matches!(t, Token::SingleWildcard | Token::MultiWildcard));

        match (self_first_wildcard, other_first_wildcard) {
            (None, None) => false, // Both are literal, equally specific
            (None, Some(_)) => true, // Self is all literal, more specific
            (Some(_), None) => false, // Other is all literal, less specific
            (Some(a), Some(b)) => a > b, // Wildcard appears later in self
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl FromStr for Pattern {
    type Err = SubjectError;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

/// A trait for types that can match patterns
pub trait PatternMatcher {
    /// Check if this matches the given pattern
    fn matches_pattern(&self, pattern: &Pattern) -> bool;
}

impl PatternMatcher for Subject {
    fn matches_pattern(&self, pattern: &Pattern) -> bool {
        pattern.matches(self)
    }
}

impl PatternMatcher for str {
    fn matches_pattern(&self, pattern: &Pattern) -> bool {
        pattern.matches_str(self)
    }
}

impl PatternMatcher for String {
    fn matches_pattern(&self, pattern: &Pattern) -> bool {
        pattern.matches_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_pattern() {
        let pattern = Pattern::new("people.person.created.v1").unwrap();
        let subject = Subject::new("people.person.created.v1").unwrap();

        assert!(pattern.matches(&subject));
        assert!(!pattern.matches_str("people.person.updated.v1"));
    }

    #[test]
    fn test_single_wildcard() {
        let pattern = Pattern::new("people.*.created.v1").unwrap();

        assert!(pattern.matches_str("people.person.created.v1"));
        assert!(pattern.matches_str("people.employee.created.v1"));
        assert!(!pattern.matches_str("organizations.company.created.v1"));
        assert!(!pattern.matches_str("people.person.employee.created.v1")); // * matches exactly one
    }

    #[test]
    fn test_multi_wildcard() {
        let pattern = Pattern::new("people.>").unwrap();

        assert!(pattern.matches_str("people.person.created.v1"));
        assert!(pattern.matches_str("people.employee.manager.assigned.v2"));
        assert!(!pattern.matches_str("organizations.company.created.v1"));
    }

    #[test]
    fn test_combined_wildcards() {
        let pattern = Pattern::new("*.*.created.>").unwrap();

        assert!(pattern.matches_str("people.person.created.v1"));
        assert!(pattern.matches_str("orders.order.created.v2"));
        assert!(pattern.matches_str("inventory.product.created.v1.beta"));
        assert!(!pattern.matches_str("people.created.v1")); // Too few parts
    }

    #[test]
    fn test_invalid_patterns() {
        // Empty pattern
        assert!(Pattern::new("").is_err());

        // Empty token
        assert!(Pattern::new("people..created.v1").is_err());

        // > not at end
        assert!(Pattern::new("people.>.created.v1").is_err());

        // Invalid characters
        assert!(Pattern::new("people.per$on.*.v1").is_err());
    }

    #[test]
    fn test_specificity() {
        let p1 = Pattern::new("people.person.created.v1").unwrap();
        let p2 = Pattern::new("people.*.created.v1").unwrap();
        let p3 = Pattern::new("people.*.*.v1").unwrap();
        let p4 = Pattern::new("people.>").unwrap();

        assert!(p1.is_more_specific_than(&p2));
        assert!(p2.is_more_specific_than(&p3));
        assert!(p3.is_more_specific_than(&p4));
        assert!(!p4.is_more_specific_than(&p1));
    }

    #[test]
    fn test_pattern_matcher_trait() {
        let pattern = Pattern::new("events.*.completed.>").unwrap();
        let subject = Subject::new("events.workflow.completed.v1").unwrap();

        assert!(subject.matches_pattern(&pattern));
        assert!("events.task.completed.v2".matches_pattern(&pattern));
        assert!(String::from("events.job.completed.v1.final").matches_pattern(&pattern));
    }
}
