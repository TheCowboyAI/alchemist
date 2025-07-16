//! Domain-based content partitioning for automatic bucket assignment
//!
//! This module provides intelligent content partitioning based on domain types
//! and content categories, enabling automatic routing to appropriate storage buckets.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Content domain categories for fine-grained partitioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentDomain {
    // Creative & Media
    Music,
    Video,
    Images,
    Graphics,
    
    // Documents & Office
    Documents,
    Spreadsheets,
    Presentations,
    Reports,
    
    // Legal & Business
    Contracts,
    Agreements,
    Policies,
    Compliance,
    
    // Social & Communication
    SocialMedia,
    Memes,
    Messages,
    Posts,
    
    // Technical & Development
    SourceCode,
    Configuration,
    Documentation,
    Schemas,
    
    // Personal & Private
    Personal,
    Private,
    Encrypted,
    Sensitive,
    
    // Research & Academic
    Research,
    Papers,
    Studies,
    Educational,
    
    // Financial & Accounting
    Financial,
    Invoices,
    Receipts,
    Statements,
    
    // Medical & Health
    Medical,
    HealthRecords,
    Prescriptions,
    LabResults,
    
    // Government & Public
    Government,
    PublicRecords,
    Licenses,
    Permits,
}

/// Partition strategy for content routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionStrategy {
    /// Domain to bucket mapping
    domain_mapping: HashMap<ContentDomain, String>,
    /// File extension to domain mapping
    extension_mapping: HashMap<String, ContentDomain>,
    /// MIME type to domain mapping
    mime_mapping: HashMap<String, ContentDomain>,
    /// Content pattern matchers
    pattern_matchers: Vec<PatternMatcher>,
}

/// Pattern matcher for content classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatcher {
    /// Pattern name
    pub name: String,
    /// Keywords to match
    pub keywords: Vec<String>,
    /// Target domain
    pub domain: ContentDomain,
    /// Priority (higher wins)
    pub priority: u32,
}

impl Default for PartitionStrategy {
    fn default() -> Self {
        let mut strategy = Self {
            domain_mapping: HashMap::new(),
            extension_mapping: HashMap::new(),
            mime_mapping: HashMap::new(),
            pattern_matchers: Vec::new(),
        };
        
        // Initialize default mappings
        strategy.init_domain_mappings();
        strategy.init_extension_mappings();
        strategy.init_mime_mappings();
        strategy.init_pattern_matchers();
        
        strategy
    }
}

impl PartitionStrategy {
    /// Initialize domain to bucket mappings
    fn init_domain_mappings(&mut self) {
        // Creative & Media
        self.domain_mapping.insert(ContentDomain::Music, "cim-media-music".to_string());
        self.domain_mapping.insert(ContentDomain::Video, "cim-media-video".to_string());
        self.domain_mapping.insert(ContentDomain::Images, "cim-media-images".to_string());
        self.domain_mapping.insert(ContentDomain::Graphics, "cim-media-graphics".to_string());
        
        // Documents & Office
        self.domain_mapping.insert(ContentDomain::Documents, "cim-docs-general".to_string());
        self.domain_mapping.insert(ContentDomain::Spreadsheets, "cim-docs-sheets".to_string());
        self.domain_mapping.insert(ContentDomain::Presentations, "cim-docs-presentations".to_string());
        self.domain_mapping.insert(ContentDomain::Reports, "cim-docs-reports".to_string());
        
        // Legal & Business
        self.domain_mapping.insert(ContentDomain::Contracts, "cim-legal-contracts".to_string());
        self.domain_mapping.insert(ContentDomain::Agreements, "cim-legal-agreements".to_string());
        self.domain_mapping.insert(ContentDomain::Policies, "cim-legal-policies".to_string());
        self.domain_mapping.insert(ContentDomain::Compliance, "cim-legal-compliance".to_string());
        
        // Social & Communication
        self.domain_mapping.insert(ContentDomain::SocialMedia, "cim-social-media".to_string());
        self.domain_mapping.insert(ContentDomain::Memes, "cim-social-memes".to_string());
        self.domain_mapping.insert(ContentDomain::Messages, "cim-social-messages".to_string());
        self.domain_mapping.insert(ContentDomain::Posts, "cim-social-posts".to_string());
        
        // Technical & Development
        self.domain_mapping.insert(ContentDomain::SourceCode, "cim-tech-code".to_string());
        self.domain_mapping.insert(ContentDomain::Configuration, "cim-tech-config".to_string());
        self.domain_mapping.insert(ContentDomain::Documentation, "cim-tech-docs".to_string());
        self.domain_mapping.insert(ContentDomain::Schemas, "cim-tech-schemas".to_string());
        
        // Personal & Private
        self.domain_mapping.insert(ContentDomain::Personal, "cim-personal-general".to_string());
        self.domain_mapping.insert(ContentDomain::Private, "cim-personal-private".to_string());
        self.domain_mapping.insert(ContentDomain::Encrypted, "cim-personal-encrypted".to_string());
        self.domain_mapping.insert(ContentDomain::Sensitive, "cim-personal-sensitive".to_string());
        
        // Research & Academic
        self.domain_mapping.insert(ContentDomain::Research, "cim-academic-research".to_string());
        self.domain_mapping.insert(ContentDomain::Papers, "cim-academic-papers".to_string());
        self.domain_mapping.insert(ContentDomain::Studies, "cim-academic-studies".to_string());
        self.domain_mapping.insert(ContentDomain::Educational, "cim-academic-educational".to_string());
        
        // Financial & Accounting
        self.domain_mapping.insert(ContentDomain::Financial, "cim-finance-general".to_string());
        self.domain_mapping.insert(ContentDomain::Invoices, "cim-finance-invoices".to_string());
        self.domain_mapping.insert(ContentDomain::Receipts, "cim-finance-receipts".to_string());
        self.domain_mapping.insert(ContentDomain::Statements, "cim-finance-statements".to_string());
        
        // Medical & Health
        self.domain_mapping.insert(ContentDomain::Medical, "cim-health-medical".to_string());
        self.domain_mapping.insert(ContentDomain::HealthRecords, "cim-health-records".to_string());
        self.domain_mapping.insert(ContentDomain::Prescriptions, "cim-health-prescriptions".to_string());
        self.domain_mapping.insert(ContentDomain::LabResults, "cim-health-lab".to_string());
        
        // Government & Public
        self.domain_mapping.insert(ContentDomain::Government, "cim-gov-general".to_string());
        self.domain_mapping.insert(ContentDomain::PublicRecords, "cim-gov-public".to_string());
        self.domain_mapping.insert(ContentDomain::Licenses, "cim-gov-licenses".to_string());
        self.domain_mapping.insert(ContentDomain::Permits, "cim-gov-permits".to_string());
    }
    
    /// Initialize file extension mappings
    fn init_extension_mappings(&mut self) {
        // Music
        for ext in &["mp3", "wav", "flac", "ogg", "m4a", "aac", "wma", "opus"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Music);
        }
        
        // Video
        for ext in &["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Video);
        }
        
        // Images
        for ext in &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "ico"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Images);
        }
        
        // Graphics
        for ext in &["svg", "ai", "psd", "xcf", "sketch", "fig", "xd"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Graphics);
        }
        
        // Documents
        for ext in &["doc", "docx", "odt", "rtf", "txt", "md", "tex"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Documents);
        }
        
        // Spreadsheets
        for ext in &["xls", "xlsx", "ods", "csv", "tsv"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Spreadsheets);
        }
        
        // Presentations
        for ext in &["ppt", "pptx", "odp", "key"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Presentations);
        }
        
        // Source Code
        for ext in &["rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "hpp", "cs", "rb", "php"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::SourceCode);
        }
        
        // Configuration
        for ext in &["json", "yaml", "yml", "toml", "ini", "conf", "cfg", "xml"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Configuration);
        }
        
        // Financial
        for ext in &["ofx", "qfx", "qif", "aba"] {
            self.extension_mapping.insert(ext.to_string(), ContentDomain::Financial);
        }
    }
    
    /// Initialize MIME type mappings
    fn init_mime_mappings(&mut self) {
        // Audio
        self.mime_mapping.insert("audio/mpeg".to_string(), ContentDomain::Music);
        self.mime_mapping.insert("audio/wav".to_string(), ContentDomain::Music);
        self.mime_mapping.insert("audio/flac".to_string(), ContentDomain::Music);
        self.mime_mapping.insert("audio/ogg".to_string(), ContentDomain::Music);
        
        // Video
        self.mime_mapping.insert("video/mp4".to_string(), ContentDomain::Video);
        self.mime_mapping.insert("video/x-msvideo".to_string(), ContentDomain::Video);
        self.mime_mapping.insert("video/quicktime".to_string(), ContentDomain::Video);
        self.mime_mapping.insert("video/webm".to_string(), ContentDomain::Video);
        
        // Images
        self.mime_mapping.insert("image/jpeg".to_string(), ContentDomain::Images);
        self.mime_mapping.insert("image/png".to_string(), ContentDomain::Images);
        self.mime_mapping.insert("image/gif".to_string(), ContentDomain::Images);
        self.mime_mapping.insert("image/webp".to_string(), ContentDomain::Images);
        self.mime_mapping.insert("image/svg+xml".to_string(), ContentDomain::Graphics);
        
        // Documents
        self.mime_mapping.insert("application/msword".to_string(), ContentDomain::Documents);
        self.mime_mapping.insert("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(), ContentDomain::Documents);
        self.mime_mapping.insert("application/pdf".to_string(), ContentDomain::Documents);
        self.mime_mapping.insert("text/plain".to_string(), ContentDomain::Documents);
        self.mime_mapping.insert("text/markdown".to_string(), ContentDomain::Documents);
        
        // Spreadsheets
        self.mime_mapping.insert("application/vnd.ms-excel".to_string(), ContentDomain::Spreadsheets);
        self.mime_mapping.insert("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(), ContentDomain::Spreadsheets);
        self.mime_mapping.insert("text/csv".to_string(), ContentDomain::Spreadsheets);
        
        // Presentations
        self.mime_mapping.insert("application/vnd.ms-powerpoint".to_string(), ContentDomain::Presentations);
        self.mime_mapping.insert("application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(), ContentDomain::Presentations);
    }
    
    /// Initialize pattern matchers
    fn init_pattern_matchers(&mut self) {
        // Contract patterns
        self.pattern_matchers.push(PatternMatcher {
            name: "contract_detector".to_string(),
            keywords: vec![
                "contract".to_string(),
                "agreement".to_string(),
                "terms and conditions".to_string(),
                "hereby agree".to_string(),
                "party of the first part".to_string(),
            ],
            domain: ContentDomain::Contracts,
            priority: 100,
        });
        
        // Invoice patterns
        self.pattern_matchers.push(PatternMatcher {
            name: "invoice_detector".to_string(),
            keywords: vec![
                "invoice".to_string(),
                "bill to".to_string(),
                "payment due".to_string(),
                "invoice number".to_string(),
                "subtotal".to_string(),
                "tax".to_string(),
                "total due".to_string(),
            ],
            domain: ContentDomain::Invoices,
            priority: 90,
        });
        
        // Medical patterns
        self.pattern_matchers.push(PatternMatcher {
            name: "medical_detector".to_string(),
            keywords: vec![
                "patient".to_string(),
                "diagnosis".to_string(),
                "prescription".to_string(),
                "medical record".to_string(),
                "lab results".to_string(),
                "treatment".to_string(),
            ],
            domain: ContentDomain::Medical,
            priority: 95,
        });
        
        // Social media patterns
        self.pattern_matchers.push(PatternMatcher {
            name: "social_detector".to_string(),
            keywords: vec![
                "#".to_string(),      // Any hashtag
                "@".to_string(),      // Any mention
                "retweet".to_string(),
                "like".to_string(),
                "share".to_string(),
                "comment".to_string(),
                "post".to_string(),   // Common social media term
                "follow".to_string(), // Common social media term
            ],
            domain: ContentDomain::SocialMedia,
            priority: 70,
        });
        
        // Meme patterns
        self.pattern_matchers.push(PatternMatcher {
            name: "meme_detector".to_string(),
            keywords: vec![
                "meme".to_string(),
                "lol".to_string(),
                "funny".to_string(),
                "viral".to_string(),
                "trending".to_string(),
            ],
            domain: ContentDomain::Memes,
            priority: 60,
        });
    }
    
    /// Determine content domain based on metadata
    pub fn determine_domain(
        &self,
        filename: Option<&str>,
        mime_type: Option<&str>,
        content_preview: Option<&str>,
        metadata: Option<&HashMap<String, String>>,
    ) -> ContentDomain {
        // Priority 1: Check metadata hints
        if let Some(meta) = metadata {
            if let Some(domain_hint) = meta.get("content_domain") {
                if let Ok(domain) = serde_json::from_str::<ContentDomain>(domain_hint) {
                    return domain;
                }
            }
        }
        
        // Priority 2: Pattern matching on content
        if let Some(preview) = content_preview {
            let preview_lower = preview.to_lowercase();
            let mut best_match: Option<(&PatternMatcher, usize)> = None;
            
            for matcher in &self.pattern_matchers {
                let match_count = matcher.keywords.iter()
                    .filter(|keyword| preview_lower.contains(&keyword.to_lowercase()))
                    .count();
                
                if match_count > 0 {
                    match best_match {
                        None => best_match = Some((matcher, match_count)),
                        Some((_, count)) if match_count > count => {
                            best_match = Some((matcher, match_count));
                        }
                        Some((current, count)) if match_count == count && matcher.priority > current.priority => {
                            best_match = Some((matcher, match_count));
                        }
                        _ => {}
                    }
                }
            }
            
            if let Some((matcher, _)) = best_match {
                return matcher.domain;
            }
        }
        
        // Priority 3: MIME type
        if let Some(mime) = mime_type {
            if let Some(domain) = self.mime_mapping.get(mime) {
                return *domain;
            }
        }
        
        // Priority 4: File extension
        if let Some(name) = filename {
            if let Some(ext) = name.split('.').next_back() {
                if let Some(domain) = self.extension_mapping.get(&ext.to_lowercase()) {
                    return *domain;
                }
            }
        }
        
        // Default to general documents
        ContentDomain::Documents
    }
    
    /// Get bucket name for a domain
    pub fn get_bucket_for_domain(&self, domain: ContentDomain) -> &str {
        self.domain_mapping.get(&domain)
            .map(|s| s.as_str())
            .unwrap_or("cim-general")
    }
    
    /// Add custom domain mapping
    pub fn add_domain_mapping(&mut self, domain: ContentDomain, bucket: String) {
        self.domain_mapping.insert(domain, bucket);
    }
    
    /// Add custom extension mapping
    pub fn add_extension_mapping(&mut self, extension: String, domain: ContentDomain) {
        self.extension_mapping.insert(extension.to_lowercase(), domain);
    }
    
    /// Add custom MIME mapping
    pub fn add_mime_mapping(&mut self, mime_type: String, domain: ContentDomain) {
        self.mime_mapping.insert(mime_type, domain);
    }
    
    /// Add custom pattern matcher
    pub fn add_pattern_matcher(&mut self, matcher: PatternMatcher) {
        self.pattern_matchers.push(matcher);
        // Sort by priority (descending)
        self.pattern_matchers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
}

/// Domain-aware content info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainContentInfo {
    /// The content domain
    pub domain: ContentDomain,
    /// The target bucket
    pub bucket: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Detection method used
    pub detection_method: DetectionMethod,
}

/// How the domain was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMethod {
    Metadata,
    PatternMatch { patterns_matched: usize },
    MimeType,
    FileExtension,
    Default,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_music_detection() {
        let strategy = PartitionStrategy::default();
        
        // Test by extension
        let domain = strategy.determine_domain(
            Some("song.mp3"),
            None,
            None,
            None,
        );
        assert_eq!(domain, ContentDomain::Music);
        
        // Test by MIME type
        let domain = strategy.determine_domain(
            None,
            Some("audio/mpeg"),
            None,
            None,
        );
        assert_eq!(domain, ContentDomain::Music);
    }
    
    #[test]
    fn test_contract_detection() {
        let strategy = PartitionStrategy::default();
        
        let domain = strategy.determine_domain(
            Some("agreement.pdf"),
            Some("application/pdf"),
            Some("This contract is entered into between Party A and Party B"),
            None,
        );
        assert_eq!(domain, ContentDomain::Contracts);
    }
    
    #[test]
    fn test_invoice_detection() {
        let strategy = PartitionStrategy::default();
        
        let domain = strategy.determine_domain(
            Some("invoice_2024.pdf"),
            None,
            Some("Invoice Number: INV-2024-001\nBill To: Customer Name\nTotal Due: $1,000"),
            None,
        );
        assert_eq!(domain, ContentDomain::Invoices);
    }
    
    #[test]
    fn test_social_media_detection() {
        let strategy = PartitionStrategy::default();
        
        let domain = strategy.determine_domain(
            None,  // No filename to avoid .json mapping to Configuration
            None,
            Some("Check out this #awesome post! @friend what do you think?"),
            None,
        );
        assert_eq!(domain, ContentDomain::SocialMedia);
    }
    
    #[test]
    fn test_bucket_mapping() {
        let strategy = PartitionStrategy::default();
        
        assert_eq!(strategy.get_bucket_for_domain(ContentDomain::Music), "cim-media-music");
        assert_eq!(strategy.get_bucket_for_domain(ContentDomain::Contracts), "cim-legal-contracts");
        assert_eq!(strategy.get_bucket_for_domain(ContentDomain::Memes), "cim-social-memes");
    }
} 