//! Content types for common file formats with verification
//!
//! This module provides typed content wrappers for common file formats
//! including documents, images, audio, and video files. Each type includes
//! verification to ensure the content matches the expected format.

pub mod indexing;
pub mod service;
pub mod transformers;

use crate::{TypedContent, ContentType, Error, Result};
use serde::{Deserialize, Serialize};

/// Magic bytes for file format detection
mod magic {
    pub const PDF: &[u8] = b"%PDF-";
    pub const DOCX: &[u8] = b"PK\x03\x04";
    pub const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";
    pub const JPEG: &[u8] = b"\xFF\xD8\xFF";
    pub const GIF: &[u8] = b"GIF8";
    pub const WEBP: &[u8] = b"RIFF";
    pub const MP3_ID3: &[u8] = b"ID3";
    pub const MP3_SYNC: &[u8] = b"\xFF\xFB";
    pub const WAV: &[u8] = b"RIFF";
    pub const FLAC: &[u8] = b"fLaC";
    pub const OGG: &[u8] = b"OggS";
    pub const AAC_ADTS: &[u8] = b"\xFF\xF1";
    // MP4 and MOV are detected by checking ftyp box at offset 4
    // pub const MP4: &[u8] = b"\x00\x00\x00\x20ftypmp4";
    // pub const MOV: &[u8] = b"\x00\x00\x00\x14ftyp";
    pub const MKV: &[u8] = b"\x1A\x45\xDF\xA3";
    pub const AVI: &[u8] = b"RIFF";
}

/// Verified PDF document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    /// The raw PDF data
    pub data: Vec<u8>,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Verified DOCX document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxDocument {
    /// The raw DOCX data
    pub data: Vec<u8>,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Verified Markdown document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownDocument {
    /// The markdown content as UTF-8 text
    pub content: String,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Verified plain text document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocument {
    /// The text content
    pub content: String,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Common document metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<u64>,
    pub modified_at: Option<u64>,
    pub tags: Vec<String>,
    pub language: Option<String>,
}

/// Image formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageContent {
    Jpeg(JpegImage),
    Png(PngImage),
    Gif(GifImage),
    WebP(WebPImage),
}

/// Verified JPEG image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JpegImage {
    pub data: Vec<u8>,
    pub metadata: ImageMetadata,
}

/// Verified PNG image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PngImage {
    pub data: Vec<u8>,
    pub metadata: ImageMetadata,
}

/// Verified GIF image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GifImage {
    pub data: Vec<u8>,
    pub metadata: ImageMetadata,
}

/// Verified WebP image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPImage {
    pub data: Vec<u8>,
    pub metadata: ImageMetadata,
}

/// Common image metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub color_space: Option<String>,
    pub compression: Option<String>,
    pub tags: Vec<String>,
}

/// Audio formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioContent {
    Mp3(Mp3Audio),
    Wav(WavAudio),
    Flac(FlacAudio),
    Aac(AacAudio),
    Ogg(OggAudio),
}

/// Verified MP3 audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mp3Audio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Verified WAV audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WavAudio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Verified FLAC audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlacAudio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Verified AAC audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AacAudio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Verified OGG audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OggAudio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Common audio metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioMetadata {
    pub duration_ms: Option<u64>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub codec: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub year: Option<u16>,
    pub tags: Vec<String>,
}

/// Video formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoContent {
    Mp4(Mp4Video),
    Mov(MovVideo),
    Mkv(MkvVideo),
    Avi(AviVideo),
}

/// Verified MP4 video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mp4Video {
    pub data: Vec<u8>,
    pub metadata: VideoMetadata,
}

/// Verified MOV video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovVideo {
    pub data: Vec<u8>,
    pub metadata: VideoMetadata,
}

/// Verified MKV video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MkvVideo {
    pub data: Vec<u8>,
    pub metadata: VideoMetadata,
}

/// Verified AVI video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AviVideo {
    pub data: Vec<u8>,
    pub metadata: VideoMetadata,
}

/// Common video metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoMetadata {
    pub duration_ms: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub bitrate: Option<u32>,
    pub tags: Vec<String>,
}

/// Content type codes for each format
pub mod codec {
    // Document types (0x600000 - 0x60FFFF)
    pub const PDF: u64 = 0x600001;
    pub const DOCX: u64 = 0x600002;
    pub const MARKDOWN: u64 = 0x600003;
    pub const TEXT: u64 = 0x600004;
    
    // Image types (0x610000 - 0x61FFFF)
    pub const JPEG: u64 = 0x610001;
    pub const PNG: u64 = 0x610002;
    pub const GIF: u64 = 0x610003;
    pub const WEBP: u64 = 0x610004;
    
    // Audio types (0x620000 - 0x62FFFF)
    pub const MP3: u64 = 0x620001;
    pub const WAV: u64 = 0x620002;
    pub const FLAC: u64 = 0x620003;
    pub const AAC: u64 = 0x620004;
    pub const OGG: u64 = 0x620005;
    
    // Video types (0x630000 - 0x63FFFF)
    pub const MP4: u64 = 0x630001;
    pub const MOV: u64 = 0x630002;
    pub const MKV: u64 = 0x630003;
    pub const AVI: u64 = 0x630004;
}

// Implement TypedContent for document types

impl TypedContent for PdfDocument {
    const CODEC: u64 = codec::PDF;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::PDF);
}

impl TypedContent for DocxDocument {
    const CODEC: u64 = codec::DOCX;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::DOCX);
}

impl TypedContent for MarkdownDocument {
    const CODEC: u64 = codec::MARKDOWN;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MARKDOWN);
}

impl TypedContent for TextDocument {
    const CODEC: u64 = codec::TEXT;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::TEXT);
}

// Implement TypedContent for image types

impl TypedContent for JpegImage {
    const CODEC: u64 = codec::JPEG;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::JPEG);
}

impl TypedContent for PngImage {
    const CODEC: u64 = codec::PNG;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::PNG);
}

impl TypedContent for GifImage {
    const CODEC: u64 = codec::GIF;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::GIF);
}

impl TypedContent for WebPImage {
    const CODEC: u64 = codec::WEBP;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::WEBP);
}

// Implement TypedContent for audio types

impl TypedContent for Mp3Audio {
    const CODEC: u64 = codec::MP3;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MP3);
}

impl TypedContent for WavAudio {
    const CODEC: u64 = codec::WAV;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::WAV);
}

impl TypedContent for FlacAudio {
    const CODEC: u64 = codec::FLAC;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::FLAC);
}

impl TypedContent for AacAudio {
    const CODEC: u64 = codec::AAC;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::AAC);
}

impl TypedContent for OggAudio {
    const CODEC: u64 = codec::OGG;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::OGG);
}

// Implement TypedContent for video types

impl TypedContent for Mp4Video {
    const CODEC: u64 = codec::MP4;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MP4);
}

impl TypedContent for MovVideo {
    const CODEC: u64 = codec::MOV;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MOV);
}

impl TypedContent for MkvVideo {
    const CODEC: u64 = codec::MKV;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MKV);
}

impl TypedContent for AviVideo {
    const CODEC: u64 = codec::AVI;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::AVI);
}

// Verification functions

impl PdfDocument {
    /// Create a new PDF document with verification
    pub fn new(data: Vec<u8>, metadata: DocumentMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid PDF file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    /// Verify that the data is a valid PDF
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::PDF)
    }
}

impl DocxDocument {
    /// Create a new DOCX document with verification
    pub fn new(data: Vec<u8>, metadata: DocumentMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid DOCX file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    /// Verify that the data is a valid DOCX (ZIP with specific structure)
    pub fn verify(data: &[u8]) -> bool {
        // DOCX files are ZIP files, check for PK header
        data.starts_with(magic::DOCX)
    }
}

impl MarkdownDocument {
    /// Create a new Markdown document
    pub fn new(content: String, metadata: DocumentMetadata) -> Result<Self> {
        // Markdown is plain text, no specific verification needed
        Ok(Self { content, metadata })
    }
    
    /// Create from UTF-8 bytes
    pub fn from_bytes(data: Vec<u8>, metadata: DocumentMetadata) -> Result<Self> {
        let content = String::from_utf8(data)
            .map_err(|_| Error::InvalidContent("Invalid UTF-8 in markdown".into()))?;
        Ok(Self { content, metadata })
    }
}

impl TextDocument {
    /// Create a new text document
    pub fn new(content: String, metadata: DocumentMetadata) -> Result<Self> {
        Ok(Self { content, metadata })
    }
    
    /// Create from UTF-8 bytes
    pub fn from_bytes(data: Vec<u8>, metadata: DocumentMetadata) -> Result<Self> {
        let content = String::from_utf8(data)
            .map_err(|_| Error::InvalidContent("Invalid UTF-8 in text".into()))?;
        Ok(Self { content, metadata })
    }
}

// Image verification

impl JpegImage {
    pub fn new(data: Vec<u8>, metadata: ImageMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid JPEG file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::JPEG)
    }
}

impl PngImage {
    pub fn new(data: Vec<u8>, metadata: ImageMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid PNG file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::PNG)
    }
}

impl GifImage {
    pub fn new(data: Vec<u8>, metadata: ImageMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid GIF file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::GIF)
    }
}

impl WebPImage {
    pub fn new(data: Vec<u8>, metadata: ImageMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid WebP file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::WEBP) && data.len() > 12 && &data[8..12] == b"WEBP"
    }
}

// Audio verification

impl Mp3Audio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid MP3 file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::MP3_ID3) || data.starts_with(magic::MP3_SYNC)
    }
}

impl WavAudio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid WAV file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::WAV) && data.len() > 12 && &data[8..12] == b"WAVE"
    }
}

impl FlacAudio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid FLAC file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::FLAC)
    }
}

impl AacAudio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid AAC file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::AAC_ADTS)
    }
}

impl OggAudio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid OGG file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::OGG)
    }
}

// Video verification

impl Mp4Video {
    pub fn new(data: Vec<u8>, metadata: VideoMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid MP4 file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.len() >= 8 && &data[4..8] == b"ftyp"
    }
}

impl MovVideo {
    pub fn new(data: Vec<u8>, metadata: VideoMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid MOV file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.len() >= 12 && &data[4..8] == b"ftyp" && &data[8..12] == b"qt  "
    }
}

impl MkvVideo {
    pub fn new(data: Vec<u8>, metadata: VideoMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid MKV file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::MKV)
    }
}

impl AviVideo {
    pub fn new(data: Vec<u8>, metadata: VideoMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid AVI file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::AVI) && data.len() > 12 && &data[8..12] == b"AVI "
    }
}

/// Helper to detect content type from data
pub fn detect_content_type(data: &[u8]) -> Option<ContentType> {
    // Documents
    if data.starts_with(magic::PDF) {
        return Some(ContentType::Custom(codec::PDF));
    }
    if data.starts_with(magic::DOCX) {
        return Some(ContentType::Custom(codec::DOCX));
    }
    
    // Images
    if data.starts_with(magic::PNG) {
        return Some(ContentType::Custom(codec::PNG));
    }
    if data.starts_with(magic::JPEG) {
        return Some(ContentType::Custom(codec::JPEG));
    }
    if data.starts_with(magic::GIF) {
        return Some(ContentType::Custom(codec::GIF));
    }
    if data.starts_with(magic::WEBP) && data.len() > 12 && &data[8..12] == b"WEBP" {
        return Some(ContentType::Custom(codec::WEBP));
    }
    
    // Audio
    if data.starts_with(magic::MP3_ID3) || data.starts_with(magic::MP3_SYNC) {
        return Some(ContentType::Custom(codec::MP3));
    }
    if data.starts_with(magic::WAV) && data.len() > 12 && &data[8..12] == b"WAVE" {
        return Some(ContentType::Custom(codec::WAV));
    }
    if data.starts_with(magic::FLAC) {
        return Some(ContentType::Custom(codec::FLAC));
    }
    if data.starts_with(magic::AAC_ADTS) {
        return Some(ContentType::Custom(codec::AAC));
    }
    if data.starts_with(magic::OGG) {
        return Some(ContentType::Custom(codec::OGG));
    }
    
    // Video
    if data.len() > 12 && &data[4..8] == b"ftyp" {
        if &data[8..12] == b"qt  " {
            return Some(ContentType::Custom(codec::MOV));
        } else {
            return Some(ContentType::Custom(codec::MP4));
        }
    }
    if data.starts_with(magic::MKV) {
        return Some(ContentType::Custom(codec::MKV));
    }
    if data.starts_with(magic::AVI) && data.len() > 12 && &data[8..12] == b"AVI " {
        return Some(ContentType::Custom(codec::AVI));
    }
    
    None
}

/// Get human-readable name for content type
pub fn content_type_name(content_type: ContentType) -> &'static str {
    match content_type {
        ContentType::Custom(codec::PDF) => "PDF Document",
        ContentType::Custom(codec::DOCX) => "DOCX Document",
        ContentType::Custom(codec::MARKDOWN) => "Markdown Document",
        ContentType::Custom(codec::TEXT) => "Text Document",
        ContentType::Custom(codec::JPEG) => "JPEG Image",
        ContentType::Custom(codec::PNG) => "PNG Image",
        ContentType::Custom(codec::GIF) => "GIF Image",
        ContentType::Custom(codec::WEBP) => "WebP Image",
        ContentType::Custom(codec::MP3) => "MP3 Audio",
        ContentType::Custom(codec::WAV) => "WAV Audio",
        ContentType::Custom(codec::FLAC) => "FLAC Audio",
        ContentType::Custom(codec::AAC) => "AAC Audio",
        ContentType::Custom(codec::OGG) => "OGG Audio",
        ContentType::Custom(codec::MP4) => "MP4 Video",
        ContentType::Custom(codec::MOV) => "MOV Video",
        ContentType::Custom(codec::MKV) => "MKV Video",
        ContentType::Custom(codec::AVI) => "AVI Video",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_verification() {
        let valid_pdf = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3";
        assert!(PdfDocument::verify(valid_pdf));
        
        let invalid_pdf = b"Not a PDF";
        assert!(!PdfDocument::verify(invalid_pdf));
    }

    #[test]
    fn test_image_detection() {
        let png_data = b"\x89PNG\r\n\x1a\nsome data";
        let detected = detect_content_type(png_data);
        assert_eq!(detected, Some(ContentType::Custom(codec::PNG)));
        
        let jpeg_data = b"\xFF\xD8\xFF\xE0some jpeg data";
        let detected = detect_content_type(jpeg_data);
        assert_eq!(detected, Some(ContentType::Custom(codec::JPEG)));
    }

    #[test]
    fn test_content_type_names() {
        assert_eq!(content_type_name(ContentType::Custom(codec::PDF)), "PDF Document");
        assert_eq!(content_type_name(ContentType::Custom(codec::MP3)), "MP3 Audio");
        assert_eq!(content_type_name(ContentType::Custom(codec::MP4)), "MP4 Video");
    }
} 