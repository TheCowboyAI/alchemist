//! Content service integrating storage, indexing, and transformation
//!
//! This module provides a high-level service for managing content with
//! automatic indexing, transformation capabilities, and lifecycle management.

use crate::{
    content_types::{
        indexing::{ContentIndex, SearchQuery, SearchResult},
        transformers::{TransformTarget, TransformationResult, TransformOptions},
        PdfDocument, MarkdownDocument, TextDocument,
        JpegImage, PngImage,
        DocumentMetadata, ImageMetadata,
        content_type_name, codec,
    },
    object_store::{NatsObjectStore, PullOptions},
    TypedContent, ContentType, Cid, Result, Error,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// Serde support for CID
mod cid_serde {
    use cid::Cid;
    use serde::{Deserializer, Serializer, Deserialize};
    
    pub fn serialize<S>(cid: &Cid, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&cid.to_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Cid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// High-level content management service
pub struct ContentService {
    /// Object storage backend
    storage: Arc<NatsObjectStore>,
    /// Content search index
    index: Arc<ContentIndex>,
    /// Service configuration
    config: ContentServiceConfig,
    /// Content lifecycle hooks
    hooks: Arc<RwLock<LifecycleHooks>>,
}

/// Configuration for the content service
#[derive(Debug, Clone)]
pub struct ContentServiceConfig {
    /// Enable automatic indexing on store
    pub auto_index: bool,
    /// Enable content validation
    pub validate_on_store: bool,
    /// Maximum content size in bytes
    pub max_content_size: usize,
    /// Allowed content types (empty = all allowed)
    pub allowed_types: Vec<ContentType>,
    /// Enable content deduplication
    pub enable_deduplication: bool,
}

impl Default for ContentServiceConfig {
    fn default() -> Self {
        Self {
            auto_index: true,
            validate_on_store: true,
            max_content_size: 100 * 1024 * 1024, // 100MB
            allowed_types: Vec::new(), // All types allowed
            enable_deduplication: true,
        }
    }
}

// Type aliases for lifecycle hooks
type PreStoreHook = Box<dyn Fn(&[u8], &ContentType) -> Result<()> + Send + Sync>;
type PostStoreHook = Box<dyn Fn(&Cid, &ContentType) + Send + Sync>;
type PreRetrieveHook = Box<dyn Fn(&Cid) + Send + Sync>;
type PostRetrieveHook = Box<dyn Fn(&Cid, &[u8]) + Send + Sync>;

/// Lifecycle hooks for content operations
#[derive(Default)]
struct LifecycleHooks {
    /// Called before content is stored
    pre_store: Vec<PreStoreHook>,
    /// Called after content is stored
    #[allow(dead_code)]
    post_store: Vec<PostStoreHook>,
    /// Called before content is retrieved
    #[allow(dead_code)]
    pre_retrieve: Vec<PreRetrieveHook>,
    /// Called after content is retrieved
    #[allow(dead_code)]
    post_retrieve: Vec<PostRetrieveHook>,
}

/// Content storage result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreResult {
    /// Content CID
    #[serde(with = "cid_serde")]
    pub cid: Cid,
    /// Content type
    pub content_type: ContentType,
    /// Size in bytes
    pub size: usize,
    /// Whether content was deduplicated
    pub deduplicated: bool,
    /// Storage timestamp
    pub stored_at: u64,
}

/// Content retrieval result with metadata
#[derive(Debug, Clone)]
pub struct RetrieveResult<T> {
    /// The content
    pub content: T,
    /// Content CID
    pub cid: Cid,
    /// Retrieval timestamp
    pub retrieved_at: u64,
}

impl ContentService {
    /// Create a new content service
    pub fn new(
        storage: Arc<NatsObjectStore>,
        config: ContentServiceConfig,
    ) -> Self {
        Self {
            storage,
            index: Arc::new(ContentIndex::new()),
            config,
            hooks: Arc::new(RwLock::new(LifecycleHooks::default())),
        }
    }

    /// Store a document
    pub async fn store_document(
        &self,
        data: Vec<u8>,
        metadata: DocumentMetadata,
        format: &str,
    ) -> Result<StoreResult> {
        // Validate size
        if data.len() > self.config.max_content_size {
            return Err(Error::InvalidContent(format!("Content size {} exceeds maximum {}", 
                data.len(), self.config.max_content_size
            )));
        }

        // Detect or verify content type
        let content_type = match format {
            "pdf" => {
                let pdf = PdfDocument::new(data, metadata)?;
                self.store_typed_content(pdf).await?
            }
            "markdown" | "md" => {
                let content = String::from_utf8(data)
                    .map_err(|_| Error::InvalidContent("Invalid UTF-8".to_string()))?;
                let md = MarkdownDocument::new(content, metadata)?;
                
                // Index if enabled
                if self.config.auto_index {
                    self.index.index_document(
                        md.calculate_cid()?,
                        &md.metadata,
                        Some(&md.content),
                    ).await?;
                }
                
                self.store_typed_content(md).await?
            }
            "text" | "txt" => {
                let content = String::from_utf8(data)
                    .map_err(|_| Error::InvalidContent("Invalid UTF-8".to_string()))?;
                let txt = TextDocument::new(content, metadata)?;
                
                // Index if enabled
                if self.config.auto_index {
                    self.index.index_document(
                        txt.calculate_cid()?,
                        &txt.metadata,
                        Some(&txt.content),
                    ).await?;
                }
                
                self.store_typed_content(txt).await?
            }
            _ => {
                return Err(Error::InvalidContent(format!(
                    "Unsupported document format: {format}"
                )));
            }
        };

        Ok(content_type)
    }

    /// Store an image
    pub async fn store_image(
        &self,
        data: Vec<u8>,
        metadata: ImageMetadata,
        format: &str,
    ) -> Result<StoreResult> {
        // Validate size
        if data.len() > self.config.max_content_size {
            return Err(Error::InvalidContent(format!("Image size {} exceeds maximum {}", 
                data.len(), self.config.max_content_size
            )));
        }

        let content_type = match format {
            "jpeg" | "jpg" => {
                let jpeg = JpegImage::new(data, metadata)?;
                
                // Index if enabled
                if self.config.auto_index {
                    self.index.index_image(
                        jpeg.calculate_cid()?,
                        &jpeg.metadata,
                        ContentType::Custom(codec::JPEG),
                    ).await?;
                }
                
                self.store_typed_content(jpeg).await?
            }
            "png" => {
                let png = PngImage::new(data, metadata)?;
                
                // Index if enabled
                if self.config.auto_index {
                    self.index.index_image(
                        png.calculate_cid()?,
                        &png.metadata,
                        ContentType::Custom(codec::PNG),
                    ).await?;
                }
                
                self.store_typed_content(png).await?
            }
            _ => {
                return Err(Error::InvalidContent(format!(
                    "Unsupported image format: {format}"
                )));
            }
        };

        Ok(content_type)
    }

    /// Store typed content
    async fn store_typed_content<T: TypedContent>(
        &self,
        content: T,
    ) -> Result<StoreResult> {
        let content_type = T::CONTENT_TYPE;
        
        // Check allowed types
        if !self.config.allowed_types.is_empty() 
            && !self.config.allowed_types.contains(&content_type) {
            return Err(Error::InvalidContent(format!("Content type {} not allowed", content_type_name(content_type))));
        }

        // Calculate CID for deduplication check
        let cid = content.calculate_cid()?;
        
        // Check if already exists (deduplication)
        let deduplicated = if self.config.enable_deduplication {
            match self.storage.exists(&cid, content_type.codec()).await {
                Ok(true) => true,
                Ok(false) => false,
                Err(_) => false, // Assume doesn't exist on error
            }
        } else {
            false
        };

        // Store if not deduplicated
        let size = if !deduplicated {
            let stored_cid = self.storage.put(&content).await
                .map_err(|e| Error::InvalidContent(format!("Storage error: {e}")))?;
            assert_eq!(cid, stored_cid); // Verify CID consistency
            
            // Get actual size from storage
            match self.storage.info(&cid, content_type.codec()).await {
                Ok(info) => info.size,
                Err(_) => content.to_bytes().map(|b| b.len()).unwrap_or(0),
            }
        } else {
            // Get size from existing object
            match self.storage.info(&cid, content_type.codec()).await {
                Ok(info) => info.size,
                Err(_) => 0,
            }
        };

        Ok(StoreResult {
            cid,
            content_type,
            size,
            deduplicated,
            stored_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Retrieve content by CID
    pub async fn retrieve<T: TypedContent>(&self, cid: &Cid) -> Result<RetrieveResult<T>> {
        // Call pre-retrieve hooks
        {
            let hooks = self.hooks.read().await;
            for hook in &hooks.pre_retrieve {
                hook(cid);
            }
        }

        // Retrieve from storage
        let content: T = self.storage.get(cid).await
            .map_err(|e| Error::InvalidContent(format!("Storage error: {e}")))?;

        // Call post-retrieve hooks
        {
            let _hooks = self.hooks.read().await;
            // Note: In production, would serialize content to bytes for hook
        }

        Ok(RetrieveResult {
            content,
            cid: *cid,
            retrieved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Search for content
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        self.index.search(&query).await
    }

    /// Get content statistics
    pub async fn stats(&self) -> ContentStats {
        let index_stats = self.index.stats().await;
        
        ContentStats {
            total_documents: index_stats.total_documents,
            total_images: index_stats.total_images,
            total_audio: index_stats.total_audio,
            total_video: index_stats.total_video,
            unique_words: index_stats.unique_words,
            unique_tags: index_stats.unique_tags,
            content_types: index_stats.content_types,
        }
    }

    /// List content by type
    pub async fn list_by_type(
        &self,
        content_type: ContentType,
        options: PullOptions,
    ) -> Result<Vec<Cid>> {
        // Get objects from storage
        let objects = self.storage.list_by_content_type(content_type.codec(), None).await
            .map_err(|e| Error::InvalidContent(format!("Storage error: {e}")))?;

        // Extract CIDs
        let mut cids: Vec<Cid> = objects.into_iter().map(|info| info.cid).collect();

        // Apply filtering and sorting based on options
        if let Some(limit) = options.limit {
            cids.truncate(limit);
        }

        Ok(cids)
    }

    /// Transform content between formats
    pub async fn transform(
        &self,
        cid: &Cid,
        target: TransformTarget,
        options: TransformOptions,
    ) -> Result<TransformationResult> {
        use crate::content_types::transformers::{document, image, TransformMetadata};
        use std::time::SystemTime;
        
        // First, we need to determine the content type from the CID
        // In a real implementation, we'd store CID->ContentType mapping
        // For now, we'll try common types
        
        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        // Try to retrieve as different content types
        // This is a simplified approach - in production you'd have a CID->type mapping
        
        // Try as markdown document
        if let Ok(result) = self.retrieve::<MarkdownDocument>(cid).await {
            match target {
                TransformTarget::Html => {
                    let html = document::markdown_to_html(&result.content)?;
                    return Ok(TransformationResult {
                        data: html.into_bytes(),
                        transform_metadata: TransformMetadata {
                            from_format: "markdown".to_string(),
                            to_format: "html".to_string(),
                            transformed_at: timestamp,
                            quality_settings: std::collections::HashMap::new(),
                            notes: vec!["Converted using pulldown-cmark".to_string()],
                        },
                        source_cid: Some(*cid),
                    });
                }
                TransformTarget::Text => {
                    let text = document::to_plain_text(&result.content.content)?;
                    return Ok(TransformationResult {
                        data: text.into_bytes(),
                        transform_metadata: TransformMetadata {
                            from_format: "markdown".to_string(),
                            to_format: "text".to_string(),
                            transformed_at: timestamp,
                            quality_settings: std::collections::HashMap::new(),
                            notes: vec!["Stripped markdown formatting".to_string()],
                        },
                        source_cid: Some(*cid),
                    });
                }
                _ => {}
            }
        }
        
        // Try as JPEG image
        if let Ok(result) = self.retrieve::<JpegImage>(cid).await {
            match target {
                TransformTarget::Png => {
                    let png_data = image::convert_format(
                        &result.content.data,
                        "jpeg",
                        "png",
                        options.quality,
                    )?;
                    return Ok(TransformationResult {
                        data: png_data,
                        transform_metadata: TransformMetadata {
                            from_format: "jpeg".to_string(),
                            to_format: "png".to_string(),
                            transformed_at: timestamp,
                            quality_settings: std::collections::HashMap::new(),
                            notes: vec!["Lossless conversion to PNG".to_string()],
                        },
                        source_cid: Some(*cid),
                    });
                }
                TransformTarget::WebP => {
                    let webp_data = image::convert_format(
                        &result.content.data,
                        "jpeg",
                        "webp",
                        options.quality,
                    )?;
                    return Ok(TransformationResult {
                        data: webp_data,
                        transform_metadata: TransformMetadata {
                            from_format: "jpeg".to_string(),
                            to_format: "webp".to_string(),
                            transformed_at: timestamp,
                            quality_settings: std::collections::HashMap::new(),
                            notes: vec!["Converted to WebP format".to_string()],
                        },
                        source_cid: Some(*cid),
                    });
                }
                _ => {}
            }
        }
        
        // Try as PNG image
        if let Ok(result) = self.retrieve::<PngImage>(cid).await {
            if target == TransformTarget::Jpeg {
                let jpeg_data = image::convert_format(
                    &result.content.data,
                    "png",
                    "jpeg",
                    options.quality,
                )?;
                return Ok(TransformationResult {
                    data: jpeg_data,
                    transform_metadata: TransformMetadata {
                        from_format: "png".to_string(),
                        to_format: "jpeg".to_string(),
                        transformed_at: timestamp,
                        quality_settings: std::collections::HashMap::new(),
                        notes: vec!["Converted to JPEG with compression".to_string()],
                    },
                    source_cid: Some(*cid),
                });
            }
        }
        
        Err(Error::InvalidContent(format!(
            "Cannot transform content: either CID not found or transformation from source type to {target:?} not supported"
        )))
    }

    /// Add a pre-store hook
    pub async fn add_pre_store_hook<F>(&self, hook: F)
    where
        F: Fn(&[u8], &ContentType) -> Result<()> + Send + Sync + 'static,
    {
        let mut hooks = self.hooks.write().await;
        hooks.pre_store.push(Box::new(hook));
    }

    /// Add a post-store hook
    pub async fn add_post_store_hook<F>(&self, hook: F)
    where
        F: Fn(&Cid, &ContentType) + Send + Sync + 'static,
    {
        let mut hooks = self.hooks.write().await;
        hooks.post_store.push(Box::new(hook));
    }
}

/// Content statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStats {
    pub total_documents: usize,
    pub total_images: usize,
    pub total_audio: usize,
    pub total_video: usize,
    pub unique_words: usize,
    pub unique_tags: usize,
    pub content_types: usize,
}

/// Batch operation results
#[derive(Debug)]
pub struct BatchResult<T> {
    pub successful: Vec<T>,
    pub failed: Vec<(usize, Error)>,
}

impl ContentService {
    /// Batch store multiple items
    pub async fn batch_store<T: TypedContent + Send + 'static>(
        &self,
        items: Vec<T>,
    ) -> BatchResult<StoreResult> {
        use futures::stream::{self, StreamExt};
        
        let results = stream::iter(items.into_iter().enumerate())
            .map(|(idx, item)| {
                let service = self.clone();
                async move {
                    match service.store_typed_content(item).await {
                        Ok(result) => Ok((idx, result)),
                        Err(e) => Err((idx, e)),
                    }
                }
            })
            .buffer_unordered(10) // Process up to 10 concurrently
            .collect::<Vec<_>>()
            .await;
            
        let mut batch_result = BatchResult {
            successful: Vec::new(),
            failed: Vec::new(),
        };
        
        for result in results {
            match result {
                Ok((_, store_result)) => batch_result.successful.push(store_result),
                Err((idx, error)) => batch_result.failed.push((idx, error)),
            }
        }
        
        batch_result
    }
}

// Make ContentService cloneable
impl Clone for ContentService {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            index: Arc::clone(&self.index),
            config: self.config.clone(),
            hooks: Arc::clone(&self.hooks),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_content_service_store_and_retrieve() {
        // This would need a test harness with mock storage
        // For now, just verify compilation
    }
    
    #[test]
    fn test_config_defaults() {
        let config = ContentServiceConfig::default();
        assert!(config.auto_index);
        assert!(config.validate_on_store);
        assert_eq!(config.max_content_size, 100 * 1024 * 1024);
    }
} 