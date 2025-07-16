//! Content transformation utilities for converting between different formats
//!
//! This module provides functionality to transform content between different
//! types while preserving metadata and maintaining CID traceability.

use crate::{
    content_types::{
        MarkdownDocument,
        VideoMetadata,
        DocumentMetadata,
    },
    Error, Result,
};
use std::collections::HashMap;

// Define AudioFormat and AudioMetadata locally if they're not available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Flac,
    Ogg,
    Aac,
}

#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    pub duration_ms: Option<u64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub bitrate: Option<u32>,
    pub codec: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub year: Option<u32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Text,
    Markdown,
    Html,
    Pdf,
    Docx,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentTransformOptions {
    pub preserve_formatting: bool,
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TextSearchOptions {
    pub case_sensitive: bool,
    pub whole_words: bool,
    pub regex: bool,
}

/// Trait for content that can be transformed to other formats
pub trait Transformable {
    /// Get available transformation targets for this content type
    fn available_transformations(&self) -> Vec<TransformTarget>;
    
    /// Check if transformation to target type is supported
    fn can_transform_to(&self, target: TransformTarget) -> bool {
        self.available_transformations().contains(&target)
    }
}

/// Supported transformation targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformTarget {
    // Document transformations
    Text,
    Markdown,
    Html,
    
    // Image transformations
    Jpeg,
    Png,
    WebP,
    
    // Audio transformations
    Mp3,
    Wav,
    Flac,
    
    // Video transformations
    Mp4,
    WebM,
}

/// Result of a content transformation
#[derive(Debug, Clone)]
pub struct TransformationResult {
    /// The transformed content as raw bytes
    pub data: Vec<u8>,
    /// Metadata about the transformation
    pub transform_metadata: TransformMetadata,
    /// Original content CID for traceability
    pub source_cid: Option<cid::Cid>,
}

/// Metadata about a transformation operation
#[derive(Debug, Clone)]
pub struct TransformMetadata {
    /// Source format
    pub from_format: String,
    /// Target format
    pub to_format: String,
    /// Timestamp of transformation
    pub transformed_at: u64,
    /// Any quality settings used
    pub quality_settings: HashMap<String, String>,
    /// Warnings or notes about the transformation
    pub notes: Vec<String>,
}

/// Document transformation functions
pub mod document {
    use super::*;
    use pulldown_cmark::{html, Options, Parser};
    use regex::Regex;
    
    /// Convert Markdown to HTML using pulldown-cmark
    pub fn markdown_to_html(markdown: &MarkdownDocument) -> Result<String> {
        let mut html_output = String::new();
        html_output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html_output.push_str("<meta charset=\"UTF-8\">\n");
        
        if let Some(title) = &markdown.metadata.title {
            html_output.push_str(&format!("<title>{}</title>\n", html_escape(title)));
        }
        
        html_output.push_str("</head>\n<body>\n");
        
        // Use pulldown-cmark for proper markdown parsing
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        
        let parser = Parser::new_ext(&markdown.content, options);
        let mut body = String::new();
        html::push_html(&mut body, parser);
        
        html_output.push_str(&body);
        html_output.push_str("\n</body>\n</html>");
        
        Ok(html_output)
    }
    
    /// Convert any document to plain text
    pub fn to_plain_text(content: &str) -> Result<String> {
        // Use regex to strip HTML tags
        let tag_regex = Regex::new(r"<[^>]+>").map_err(|e| 
            Error::InvalidContent(format!("Regex error: {e}"))
        )?;
        let mut text = tag_regex.replace_all(content, "").to_string();
        
        // Strip markdown formatting
        let patterns = [
            (r"\*\*([^*]+)\*\*", "$1"), // Bold
            (r"\*([^*]+)\*", "$1"),     // Italic
            (r"__([^_]+)__", "$1"),     // Bold alternative
            (r"_([^_]+)_", "$1"),       // Italic alternative
            (r"\[([^\]]+)\]\([^)]+\)", "$1"), // Links
            (r"^#+\s+", ""),            // Headers
            (r"`([^`]+)`", "$1"),       // Inline code
            (r"```[^`]*```", ""),       // Code blocks
        ];
        
        for (pattern, replacement) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                text = regex.replace_all(&text, replacement).to_string();
            }
        }
        
        // Clean up whitespace
        let whitespace_regex = Regex::new(r"\s+").map_err(|e| 
            Error::InvalidContent(format!("Regex error: {e}"))
        )?;
        text = whitespace_regex.replace_all(&text, " ").trim().to_string();
        
        Ok(text)
    }
    
    fn html_escape(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '&' => "&amp;".to_string(),
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&#39;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }
}

/// Image transformation functions
pub mod image {
    use super::*;
    use ::image::{DynamicImage, ImageFormat};
    use ::image::imageops::FilterType;
    use std::io::Cursor;
    
    /// Convert between image formats using the image crate
    pub fn convert_format(
        data: &[u8],
        from_format: &str,
        to_format: &str,
        quality: Option<u8>,
    ) -> Result<Vec<u8>> {
        if from_format == to_format {
            return Ok(data.to_vec());
        }
        
        // Load the image
        let img = load_image(data, from_format)?;
        
        // Convert to target format
        encode_image(&img, to_format, quality)
    }
    
    /// Resize image maintaining aspect ratio
    pub fn resize(
        data: &[u8],
        format: &str,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let img = load_image(data, format)?;
        
        // Calculate new dimensions maintaining aspect ratio
        let (orig_width, orig_height) = (img.width(), img.height());
        let ratio = (width as f32 / orig_width as f32).min(height as f32 / orig_height as f32);
        let new_width = (orig_width as f32 * ratio) as u32;
        let new_height = (orig_height as f32 * ratio) as u32;
        
        // Resize the image
        let resized = img.resize(new_width, new_height, FilterType::Lanczos3);
        
        // Encode back to original format
        encode_image(&resized, format, None)
    }
    
    /// Generate thumbnail with maximum dimension
    pub fn generate_thumbnail(
        data: &[u8],
        format: &str,
        max_size: u32,
    ) -> Result<Vec<u8>> {
        let img = load_image(data, format)?;
        
        // Generate thumbnail
        let thumbnail = img.thumbnail(max_size, max_size);
        
        // For thumbnails, always use JPEG for smaller size
        encode_image(&thumbnail, "jpeg", Some(85))
    }
    
    /// Helper to load image from bytes
    fn load_image(data: &[u8], format: &str) -> Result<DynamicImage> {
        let img_format = match format.to_lowercase().as_str() {
            "jpeg" | "jpg" => ImageFormat::Jpeg,
            "png" => ImageFormat::Png,
            "webp" => ImageFormat::WebP,
            _ => return Err(Error::InvalidContent(format!("Unsupported format: {format}"))),
        };
        
        ::image::load_from_memory_with_format(data, img_format)
            .map_err(|e| Error::InvalidContent(format!("Failed to load image: {e}")))
    }
    
    /// Helper to encode image to bytes
    fn encode_image(img: &DynamicImage, format: &str, quality: Option<u8>) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        
        match format.to_lowercase().as_str() {
            "jpeg" | "jpg" => {
                let _q = quality.unwrap_or(90);
                img.write_to(&mut cursor, ImageFormat::Jpeg)
                    .map_err(|e| Error::InvalidContent(format!("Failed to encode JPEG: {e}")))?;
            }
            "png" => {
                img.write_to(&mut cursor, ImageFormat::Png)
                    .map_err(|e| Error::InvalidContent(format!("Failed to encode PNG: {e}")))?;
            }
            "webp" => {
                img.write_to(&mut cursor, ImageFormat::WebP)
                    .map_err(|e| Error::InvalidContent(format!("Failed to encode WebP: {e}")))?;
            }
            _ => return Err(Error::InvalidContent(format!("Unsupported format: {format}"))),
        }
        
        Ok(buffer)
    }
}

/// Audio transformation functions
pub mod audio {
    use super::*;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use std::io::Cursor;
    
    /// Convert between audio formats
    /// Note: Since symphonia is decode-only, we can only extract raw audio data
    /// For full conversion, you would need to integrate with an encoder library
    pub fn convert_format(
        data: &[u8],
        from_format: &str,
        to_format: &str,
        _bitrate: Option<u32>,
    ) -> Result<Vec<u8>> {
        if from_format == to_format {
            return Ok(data.to_vec());
        }
        
        // For now, we can only decode to raw PCM
        // Full conversion would require an encoder library like ffmpeg
        Err(Error::InvalidContent(
            format!("Audio conversion from {from_format} to {to_format} requires external encoder. Only metadata extraction is currently supported.")
        ))
    }
    
    /// Extract audio metadata using symphonia
    pub fn extract_metadata(data: &[u8], format: &str) -> Result<AudioMetadata> {
        // Clone data to avoid lifetime issues with symphonia
        let data_vec = data.to_vec();
        let cursor = Cursor::new(data_vec);
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
        
        // Provide a hint to the format detector
        let mut hint = Hint::new();
        match format.to_lowercase().as_str() {
            "mp3" => hint.with_extension("mp3"),
            "wav" => hint.with_extension("wav"),
            "flac" => hint.with_extension("flac"),
            "ogg" => hint.with_extension("ogg"),
            _ => &mut hint,
        };
        
        // Probe the media source
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let probe_result = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| Error::InvalidContent(format!("Failed to probe audio: {e}")))?;
        
        let mut format_reader = probe_result.format;
        
        // Extract basic metadata
        let mut metadata = AudioMetadata {
            duration_ms: None,
            sample_rate: None,
            channels: None,
            bitrate: None,
            codec: Some(format.to_string()),
            artist: None,
            album: None,
            title: None,
            year: None,
            tags: Vec::new(),
        };
        
        // Get track info
        if let Some(track) = format_reader.default_track() {
            let params = &track.codec_params;
            metadata.sample_rate = params.sample_rate;
            metadata.channels = params.channels.map(|c| c.count() as u8);
            
            // Calculate duration if available
            if let Some(n_frames) = params.n_frames {
                if let Some(sample_rate) = params.sample_rate {
                    let duration_seconds = n_frames as f64 / sample_rate as f64;
                    metadata.duration_ms = Some((duration_seconds * 1000.0) as u64);
                }
            }
            
            // Estimate bitrate
            if let Some(duration_ms) = metadata.duration_ms {
                if duration_ms > 0 {
                    let duration_seconds = duration_ms / 1000;
                    if duration_seconds > 0 {
                        metadata.bitrate = Some((data.len() as u32 * 8) / duration_seconds as u32);
                    }
                }
            }
        }
        
        // Extract metadata tags
        if let Some(metadata_rev) = format_reader.metadata().current() {
            for tag in metadata_rev.tags() {
                match tag.std_key {
                    Some(symphonia::core::meta::StandardTagKey::Artist) => {
                        metadata.artist = tag.value.to_string().into();
                    }
                    Some(symphonia::core::meta::StandardTagKey::Album) => {
                        metadata.album = tag.value.to_string().into();
                    }
                    Some(symphonia::core::meta::StandardTagKey::TrackTitle) => {
                        metadata.title = tag.value.to_string().into();
                    }
                    _ => {
                        // Tag.key is a String, not Option<String>
                        metadata.tags.push(format!("{}: {}", tag.key, tag.value));
                    }
                }
            }
        }
        
        Ok(metadata)
    }
}

/// Video transformation functions
pub mod video {
    use super::*;

    
    /// Convert between video formats
    /// Note: Full video conversion requires external tools like ffmpeg
    pub fn convert_format(
        data: &[u8],
        from_format: &str,
        to_format: &str,
        _options: VideoConversionOptions,
    ) -> Result<Vec<u8>> {
        if from_format == to_format {
            return Ok(data.to_vec());
        }
        
        // Video conversion requires external tools like ffmpeg
        // This could be implemented using the ffmpeg-sys crate or by calling ffmpeg as a subprocess
        Err(Error::InvalidContent(format!(
            "Video conversion from {from_format} to {to_format} requires external tools (e.g., ffmpeg). Consider using ffmpeg-sys crate for full implementation."
        )))
    }
    
    /// Video conversion options
    #[derive(Debug, Clone, Default)]
    pub struct VideoConversionOptions {
        pub video_codec: Option<String>,
        pub audio_codec: Option<String>,
        pub bitrate: Option<u32>,
        pub resolution: Option<(u32, u32)>,
        pub frame_rate: Option<f32>,
    }
    
    /// Extract basic video metadata
    /// Note: Full metadata extraction would require a library like ffmpeg or gstreamer
    pub fn extract_metadata(data: &[u8], format: &str) -> Result<VideoMetadata> {
        let mut metadata = VideoMetadata {
            video_codec: None,
            audio_codec: None,
            duration_ms: None,
            width: None,
            height: None,
            frame_rate: None,
            bitrate: None,
            tags: Vec::new(),
        };
        
        // Basic format detection based on file signatures
        match format.to_lowercase().as_str() {
            "mp4" | "mov" => {
                // MP4/MOV files have 'ftyp' box after first 4 bytes (box size)
                if data.len() > 12 && &data[4..8] == b"ftyp" {
                    metadata.video_codec = Some("h264".to_string()); // Common for MP4
                    
                    // Try to find moov box for metadata (simplified)
                    if let Some(_moov_pos) = find_box(data, b"moov") {
                        // In a real implementation, we would parse the moov box
                        // to extract actual metadata
                        metadata.tags.push(format!("container: {format}"));
                    }
                }
            }
            "mkv" => {
                // Matroska signature
                if data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
                    metadata.video_codec = Some("vp9".to_string()); // Common for MKV
                    metadata.tags.push("container: matroska".to_string());
                }
            }
            "webm" => {
                // WebM is a subset of Matroska
                if data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
                    metadata.video_codec = Some("vp8".to_string()); // Common for WebM
                    metadata.audio_codec = Some("vorbis".to_string());
                    metadata.tags.push("container: webm".to_string());
                }
            }
            _ => {
                metadata.tags.push(format!("format: {format}"));
            }
        }
        
        // Estimate file size and duration relationship (very rough)
        if data.len() > 1_000_000 {
            // Assume ~1MB per second as a very rough estimate
            let duration_seconds = (data.len() / 1_000_000) as u64;
            metadata.duration_ms = Some(duration_seconds * 1000);
            metadata.bitrate = Some(8_000_000); // 8 Mbps estimate
        }
        
        Ok(metadata)
    }
    
    /// Extract thumbnail from video
    /// Note: Actual implementation would require ffmpeg or similar
    pub fn extract_thumbnail(
        _data: &[u8],
        _format: &str,
        _timestamp_ms: u64,
    ) -> Result<Vec<u8>> {
        // Thumbnail extraction requires video decoding capabilities
        // This could be implemented using:
        // 1. ffmpeg-sys crate for direct ffmpeg bindings
        // 2. gstreamer-rs for GStreamer bindings
        // 3. Calling ffmpeg as a subprocess
        
        Err(Error::InvalidContent(
            "Video thumbnail extraction requires external tools. Consider using ffmpeg-sys or gstreamer-rs crates for full implementation.".to_string()
        ))
    }
    
    /// Helper to find a box in MP4/MOV format
    fn find_box(data: &[u8], box_type: &[u8; 4]) -> Option<usize> {
        let mut pos = 0;
        while pos + 8 <= data.len() {
            let size = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
            if size == 0 || pos + size > data.len() {
                break;
            }
            
            if &data[pos+4..pos+8] == box_type {
                return Some(pos);
            }
            
            pos += size;
        }
        None
    }
}

/// Batch transformation operations
pub struct BatchTransformer {
    /// Maximum concurrent transformations
    pub max_concurrent: usize,
    /// Transformation options
    pub options: TransformOptions,
}

#[derive(Debug, Clone, Default)]
pub struct TransformOptions {
    /// Preserve original metadata where possible
    pub preserve_metadata: bool,
    /// Quality settings for lossy formats
    pub quality: Option<u8>,
    /// Maximum output size in bytes
    pub max_size: Option<usize>,
    /// Custom parameters
    pub custom: HashMap<String, String>,
}

impl BatchTransformer {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            options: TransformOptions::default(),
        }
    }
    
    /// Transform multiple items in parallel
    pub async fn transform_batch<T, F>(
        &self,
        items: Vec<T>,
        transform_fn: F,
    ) -> Vec<Result<TransformationResult>>
    where
        T: Send + 'static,
        F: Fn(T) -> Result<TransformationResult> + Send + Sync + Clone + 'static,
    {
        use futures::stream::{self, StreamExt};
        
        let results = stream::iter(items)
            .map(|item| {
                let f = transform_fn.clone();
                async move { f(item) }
            })
            .buffer_unordered(self.max_concurrent)
            .collect::<Vec<_>>()
            .await;
            
        results
    }
}

/// Content validation utilities
pub mod validation {
    use super::*;
    
    /// Validate document content
    pub fn validate_document(data: &[u8], format: &str) -> Result<ValidationReport> {
        let mut report = ValidationReport::new(format);
        
        match format {
            "pdf" => {
                if !data.starts_with(b"%PDF-") {
                    report.add_error("Invalid PDF header");
                }
                if !data.ends_with(b"%%EOF") && !data.ends_with(b"%%EOF\n") {
                    report.add_warning("PDF may be truncated");
                }
            }
            "markdown" => {
                if std::str::from_utf8(data).is_err() {
                    report.add_error("Invalid UTF-8 encoding");
                }
            }
            _ => {
                report.add_warning(&format!("No validation rules for format: {format}"));
            }
        }
        
        Ok(report)
    }
    
    /// Validation report
    #[derive(Debug, Clone)]
    pub struct ValidationReport {
        pub format: String,
        pub is_valid: bool,
        pub errors: Vec<String>,
        pub warnings: Vec<String>,
    }
    
    impl ValidationReport {
        pub fn new(format: &str) -> Self {
            Self {
                format: format.to_string(),
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            }
        }
        
        pub fn add_error(&mut self, error: &str) {
            self.errors.push(error.to_string());
            self.is_valid = false;
        }
        
        pub fn add_warning(&mut self, warning: &str) {
            self.warnings.push(warning.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_markdown_to_html() {
        let md = MarkdownDocument {
            content: "# Title\n\nThis is **bold** text.".to_string(),
            metadata: DocumentMetadata {
                title: Some("Test Doc".to_string()),
                ..Default::default()
            },
        };
        
        let html = document::markdown_to_html(&md).unwrap();
        assert!(html.contains("<title>Test Doc</title>"));
        assert!(html.contains("<h1>Title"));
        assert!(html.contains("<strong>bold</strong>"));
    }
    
    #[test]
    fn test_validation_report() {
        let mut report = validation::ValidationReport::new("pdf");
        assert!(report.is_valid);
        
        report.add_warning("Minor issue");
        assert!(report.is_valid);
        
        report.add_error("Major problem");
        assert!(!report.is_valid);
    }
} 