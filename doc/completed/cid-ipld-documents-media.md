# Documents and Multimedia

> Part of the [CID/IPLD Architecture](./cid-ipld-architecture.md)

## Overview

CIM provides comprehensive support for document and multimedia content, enabling text extraction, semantic search, synchronized media playback, and multi-language support. All content is addressed through CIDs with type-specific handling.

## Document Support

### Supported Formats

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentFormat {
    // Text formats
    Markdown,
    PlainText,
    RichText,

    // Office formats
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Odt,
    Ods,
    Odp,

    // Web formats
    Html,
    Xml,

    // Data formats
    Json,
    Yaml,
    Toml,
    Csv,

    // Other formats
    Latex,
    Epub,
    Rtf,
}

impl DocumentFormat {
    pub fn supports_text_extraction(&self) -> bool {
        match self {
            DocumentFormat::Pdf | DocumentFormat::Docx |
            DocumentFormat::Odt | DocumentFormat::Html |
            DocumentFormat::Epub => true,
            _ => false,
        }
    }

    pub fn supports_structured_data(&self) -> bool {
        matches!(self,
            DocumentFormat::Json | DocumentFormat::Yaml |
            DocumentFormat::Toml | DocumentFormat::Xml
        )
    }
}
```

### Document Processing

```rust
pub struct DocumentProcessor {
    text_extractor: Box<dyn TextExtractor>,
    metadata_extractor: Box<dyn MetadataExtractor>,
    embedding_generator: Box<dyn EmbeddingGenerator>,
}

impl DocumentProcessor {
    pub async fn process_document(
        &self,
        content: &[u8],
        format: DocumentFormat,
    ) -> Result<ProcessedDocument> {
        // Extract text content
        let text = if format.supports_text_extraction() {
            Some(self.text_extractor.extract(content, format).await?)
        } else {
            None
        };

        // Extract metadata
        let metadata = self.metadata_extractor.extract(content, format).await?;

        // Generate embeddings for semantic search
        let embeddings = if let Some(ref text) = text {
            Some(self.embedding_generator.generate(text).await?)
        } else {
            None
        };

        // Extract structure for supported formats
        let structure = if format.supports_structured_data() {
            Some(extract_document_structure(content, format)?)
        } else {
            None
        };

        Ok(ProcessedDocument {
            format,
            text,
            metadata,
            embeddings,
            structure,
            processing_timestamp: SystemTime::now(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ProcessedDocument {
    pub format: DocumentFormat,
    pub text: Option<String>,
    pub metadata: DocumentMetadata,
    pub embeddings: Option<Vec<f32>>,
    pub structure: Option<DocumentStructure>,
    pub processing_timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created_date: Option<DateTime<Utc>>,
    pub modified_date: Option<DateTime<Utc>>,
    pub page_count: Option<u32>,
    pub word_count: Option<u32>,
    pub language: Option<String>,
    pub keywords: Vec<String>,
    pub custom_properties: HashMap<String, Value>,
}
```

### Text Extraction

```rust
pub trait TextExtractor: Send + Sync {
    async fn extract(
        &self,
        content: &[u8],
        format: DocumentFormat,
    ) -> Result<String>;
}

pub struct UniversalTextExtractor {
    pdf_extractor: PdfTextExtractor,
    office_extractor: OfficeTextExtractor,
    html_extractor: HtmlTextExtractor,
}

impl TextExtractor for UniversalTextExtractor {
    async fn extract(
        &self,
        content: &[u8],
        format: DocumentFormat,
    ) -> Result<String> {
        match format {
            DocumentFormat::Pdf => {
                self.pdf_extractor.extract_text(content).await
            }
            DocumentFormat::Docx | DocumentFormat::Xlsx | DocumentFormat::Pptx => {
                self.office_extractor.extract_text(content, format).await
            }
            DocumentFormat::Html | DocumentFormat::Xml => {
                self.html_extractor.extract_text(content).await
            }
            DocumentFormat::PlainText | DocumentFormat::Markdown => {
                Ok(String::from_utf8_lossy(content).to_string())
            }
            _ => Err(ExtractionError::UnsupportedFormat(format)),
        }
    }
}
```

## Multimedia Support

### Media Types

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaBundle {
    pub primary_media: MediaContent,
    pub alternate_tracks: Vec<AlternateTrack>,
    pub subtitles: Vec<SubtitleTrack>,
    pub transcripts: Vec<Transcript>,
    pub thumbnails: Vec<Thumbnail>,
    pub metadata: MediaMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediaContent {
    Video {
        cid: Cid,
        format: VideoFormat,
        duration: Duration,
        resolution: Resolution,
        bitrate: u64,
    },
    Audio {
        cid: Cid,
        format: AudioFormat,
        duration: Duration,
        bitrate: u64,
        channels: u8,
        sample_rate: u32,
    },
    Image {
        cid: Cid,
        format: ImageFormat,
        dimensions: Dimensions,
        color_space: ColorSpace,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub cid: Cid,
    pub language: Language,
    pub format: SubtitleFormat,
    pub timing_entries: Vec<TimedText>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub cid: Cid,
    pub language: Language,
    pub format: TranscriptFormat,
    pub speaker_labels: bool,
    pub timestamps: bool,
    pub confidence_scores: Option<Vec<f32>>,
}
```

### Synchronized Media

```rust
pub struct SynchronizedMedia {
    pub media_bundle: MediaBundle,
    pub sync_points: Vec<SyncPoint>,
    pub chapters: Vec<Chapter>,
    pub annotations: Vec<TimedAnnotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPoint {
    pub timestamp: Duration,
    pub sync_type: SyncType,
    pub references: Vec<SyncReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    SubtitleCue(SubtitleCueId),
    TranscriptSegment(SegmentId),
    ChapterMarker(ChapterId),
    SceneChange,
    SpeakerChange(SpeakerId),
    Custom(String),
}

impl SynchronizedMedia {
    pub fn get_content_at_time(&self, timestamp: Duration) -> SyncedContent {
        let active_subtitle = self.find_active_subtitle(timestamp);
        let active_transcript = self.find_active_transcript_segment(timestamp);
        let current_chapter = self.find_current_chapter(timestamp);
        let nearby_annotations = self.find_nearby_annotations(timestamp);

        SyncedContent {
            timestamp,
            subtitle: active_subtitle,
            transcript_segment: active_transcript,
            chapter: current_chapter,
            annotations: nearby_annotations,
        }
    }

    pub fn search_synchronized(
        &self,
        query: &str,
    ) -> Vec<SearchResult> {
        let mut results = Vec::new();

        // Search in subtitles
        for subtitle in &self.media_bundle.subtitles {
            for entry in &subtitle.timing_entries {
                if entry.text.contains(query) {
                    results.push(SearchResult {
                        timestamp: entry.start_time,
                        duration: entry.duration,
                        content_type: ContentType::Subtitle,
                        text: entry.text.clone(),
                        confidence: 1.0,
                    });
                }
            }
        }

        // Search in transcripts
        for transcript in &self.media_bundle.transcripts {
            // Search implementation
        }

        results.sort_by_key(|r| r.timestamp);
        results
    }
}
```

### Multi-Language Support

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLanguageContent {
    pub primary_language: Language,
    pub translations: HashMap<Language, TranslatedContent>,
    pub auto_translated: HashSet<Language>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedContent {
    pub cid: Cid,
    pub language: Language,
    pub translation_method: TranslationMethod,
    pub quality_score: f32,
    pub reviewed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationMethod {
    Human,
    MachineTranslation(String), // Model name
    CommunityContributed,
    Professional,
}

impl MultiLanguageContent {
    pub fn get_best_language(&self, preferred: &[Language]) -> Language {
        for lang in preferred {
            if self.primary_language == *lang {
                return *lang;
            }
            if self.translations.contains_key(lang) {
                return *lang;
            }
        }

        // Fall back to primary language
        self.primary_language
    }

    pub fn add_translation(
        &mut self,
        language: Language,
        content_cid: Cid,
        method: TranslationMethod,
    ) -> Result<()> {
        let quality_score = match &method {
            TranslationMethod::Professional => 0.95,
            TranslationMethod::Human => 0.9,
            TranslationMethod::CommunityContributed => 0.8,
            TranslationMethod::MachineTranslation(_) => 0.7,
        };

        self.translations.insert(language, TranslatedContent {
            cid: content_cid,
            language,
            translation_method: method,
            quality_score,
            reviewed: false,
        });

        Ok(())
    }
}
```

## Document Collections

### Collection Management

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCollection {
    pub collection_id: Cid,
    pub name: String,
    pub collection_type: CollectionType,
    pub documents: Vec<DocumentEntry>,
    pub metadata: CollectionMetadata,
    pub index: Option<CollectionIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionType {
    Manual,           // User-curated collection
    Query(String),    // Dynamic collection based on search
    Topic(Vec<String>), // Topic-based collection
    Project(ProjectId), // Project-specific documents
    TimeBased(TimeRange), // Documents from time period
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionIndex {
    pub full_text_index: Cid,      // Points to search index
    pub semantic_index: Cid,       // Points to embedding index
    pub metadata_index: Cid,       // Points to metadata index
    pub relationship_graph: Cid,   // Points to document relationships
}

impl DocumentCollection {
    pub async fn analyze_collection(&self) -> CollectionAnalysis {
        let themes = extract_themes(&self.documents).await;
        let key_terms = extract_key_terms(&self.documents).await;
        let document_clusters = cluster_documents(&self.documents).await;
        let summary = generate_collection_summary(&self.documents).await;

        CollectionAnalysis {
            themes,
            key_terms,
            document_clusters,
            summary,
            statistics: calculate_statistics(&self.documents),
        }
    }
}
```

## Embedding Generation

### Semantic Search Support

```rust
pub trait EmbeddingGenerator: Send + Sync {
    async fn generate(&self, text: &str) -> Result<Vec<f32>>;
    fn embedding_dimension(&self) -> usize;
}

pub struct SemanticSearchIndex {
    embeddings: HashMap<Cid, Vec<f32>>,
    dimension: usize,
    index: HnswIndex,
}

impl SemanticSearchIndex {
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        // Generate query embedding
        let query_embedding = self.embedding_generator.generate(query).await?;

        // Find nearest neighbors
        let neighbors = self.index.search(&query_embedding, limit, threshold)?;

        // Convert to search results
        let mut results = Vec::new();
        for (cid, distance) in neighbors {
            let similarity = 1.0 - distance; // Convert distance to similarity
            results.push(SearchResult {
                cid,
                similarity,
                snippet: self.generate_snippet(cid, query).await?,
            });
        }

        Ok(results)
    }
}
```

## Media Processing Pipeline

```rust
pub struct MediaProcessingPipeline {
    stages: Vec<Box<dyn ProcessingStage>>,
}

impl MediaProcessingPipeline {
    pub async fn process_media(
        &self,
        input_cid: Cid,
        media_type: MediaType,
    ) -> Result<ProcessedMedia> {
        let mut context = ProcessingContext::new(input_cid, media_type);

        for stage in &self.stages {
            context = stage.process(context).await?;
        }

        Ok(ProcessedMedia {
            original_cid: input_cid,
            processed_cid: context.output_cid,
            derivatives: context.derivatives,
            metadata: context.metadata,
        })
    }
}

// Example stages
pub struct ThumbnailGenerator;
pub struct AudioExtractor;
pub struct TranscriptionService;
pub struct SubtitleGenerator;
```

## Related Documents

- [Content Types and Codecs](./cid-ipld-content-types.md) - Type definitions
- [Content Transformations](./cid-ipld-transformations.md) - Document conversions
- [MIME Type Intelligence](./cid-ipld-mime-filegroups.md) - Format detection
- [Business Intelligence Network](./cid-ipld-business-intelligence.md) - Document insights

## Next Steps

1. Configure text extractors for your document types
2. Set up embedding generation for semantic search
3. Implement media processing pipeline
4. Create document collections for organization
