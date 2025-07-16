//! Tests for CIM-IPLD content types
//!
//! These tests verify:
//! 1. Content type verification works correctly
//! 2. CIDs are consistent for typed content
//! 3. Storage and retrieval preserves type safety
//! 4. Content detection functions properly

mod common;

use cim_ipld::{
    content_types::{
        PdfDocument, MarkdownDocument, TextDocument,
        JpegImage, PngImage, WebPImage,
        Mp3Audio, WavAudio,
        Mp4Video, MovVideo, MkvVideo,
        DocumentMetadata, ImageMetadata, AudioMetadata, VideoMetadata,
        detect_content_type, content_type_name, codec,
    },
    TypedContent, ContentType,
};
use common::TestContext;

/// Test PDF document verification and CID consistency
///
/// ```mermaid
/// graph LR
///     A[Create PDF] --> B[Verify Format]
///     B --> C[Store with CID]
///     C --> D[Retrieve by CID]
///     D --> E[Verify Content]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_pdf_document() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Create valid PDF
    let pdf_data = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\nTest PDF content".to_vec();
    let metadata = DocumentMetadata {
        title: Some("Test PDF".to_string()),
        author: Some("Test Author".to_string()),
        ..Default::default()
    };
    
    let pdf = PdfDocument::new(pdf_data.clone(), metadata.clone())?;
    
    // Verify it's detected as PDF
    assert!(PdfDocument::verify(&pdf_data));
    
    // Store and retrieve
    let cid = context.storage.put(&pdf).await?;
    let retrieved: PdfDocument = context.storage.get(&cid).await?;
    
    // Verify content matches
    assert_eq!(retrieved.data, pdf.data);
    assert_eq!(retrieved.metadata.title, metadata.title);
    
    // Verify CID consistency
    let recalculated_cid = retrieved.calculate_cid()?;
    assert_eq!(recalculated_cid, cid);
    
    println!("✓ PDF document test passed, CID: {cid}");
    
    Ok(())
}

/// Test invalid PDF rejection
#[test]
fn test_invalid_pdf() {
    let invalid_data = b"Not a PDF file".to_vec();
    let metadata = DocumentMetadata::default();
    
    let result = PdfDocument::new(invalid_data, metadata);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not a valid PDF"));
}

/// Test Markdown document handling
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_markdown_document() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    let content = r#"# Test Document

This is a **test** markdown document with:
- Lists
- *Emphasis*
- [Links](https://example.com)
"#;
    
    let metadata = DocumentMetadata {
        title: Some("Test Markdown".to_string()),
        language: Some("en".to_string()),
        tags: vec!["test".to_string(), "markdown".to_string()],
        ..Default::default()
    };
    
    let md = MarkdownDocument::new(content.to_string(), metadata)?;
    
    // Store and retrieve
    let cid = context.storage.put(&md).await?;
    let retrieved: MarkdownDocument = context.storage.get(&cid).await?;
    
    assert_eq!(retrieved.content, content);
    assert_eq!(retrieved.metadata.tags.len(), 2);
    
    println!("✓ Markdown document test passed, CID: {cid}");
    
    Ok(())
}

/// Test image type verification
///
/// ```mermaid
/// graph TD
///     A[PNG Data] --> B[Verify Signature]
///     C[JPEG Data] --> D[Verify Signature]
///     E[Invalid Data] --> F[Reject]
/// ```
#[test]
fn test_image_verification() {
    // Valid PNG
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        // Rest of PNG data...
    ];
    assert!(PngImage::verify(&png_data));
    
    // Valid JPEG
    let jpeg_data = vec![
        0xFF, 0xD8, 0xFF, 0xE0,
        // Rest of JPEG data...
    ];
    assert!(JpegImage::verify(&jpeg_data));
    
    // Invalid image
    let invalid_data = b"Not an image".to_vec();
    assert!(!PngImage::verify(&invalid_data));
    assert!(!JpegImage::verify(&invalid_data));
}

/// Test audio format handling
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_audio_formats() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Test MP3
    let mp3_data = vec![
        0x49, 0x44, 0x33, // ID3
        0x04, 0x00, 0x00, // Version and flags
        // Minimal MP3 data
    ];
    
    let mp3_metadata = AudioMetadata {
        duration_ms: Some(180000),
        bitrate: Some(320),
        artist: Some("Test Artist".to_string()),
        title: Some("Test Track".to_string()),
        ..Default::default()
    };
    
    let mp3 = Mp3Audio::new(mp3_data, mp3_metadata)?;
    let mp3_cid = context.storage.put(&mp3).await?;
    
    // Test WAV
    let wav_data = vec![
        0x52, 0x49, 0x46, 0x46, // RIFF
        0x24, 0x00, 0x00, 0x00, // Size
        0x57, 0x41, 0x56, 0x45, // WAVE
        // Minimal WAV data
    ];
    
    let wav_metadata = AudioMetadata {
        duration_ms: Some(5000),
        sample_rate: Some(44100),
        channels: Some(2),
        ..Default::default()
    };
    
    let wav = WavAudio::new(wav_data, wav_metadata)?;
    let wav_cid = context.storage.put(&wav).await?;
    
    // Retrieve and verify
    let retrieved_mp3: Mp3Audio = context.storage.get(&mp3_cid).await?;
    assert_eq!(retrieved_mp3.metadata.artist, Some("Test Artist".to_string()));
    
    let retrieved_wav: WavAudio = context.storage.get(&wav_cid).await?;
    assert_eq!(retrieved_wav.metadata.channels, Some(2));
    
    println!("✓ Audio format tests passed");
    
    Ok(())
}

/// Test video format verification
#[test]
fn test_video_verification() {
    // MP4
    let mp4_data = vec![
        0x00, 0x00, 0x00, 0x20,
        0x66, 0x74, 0x79, 0x70, // ftyp
        0x6D, 0x70, 0x34, 0x32, // mp42
    ];
    assert!(Mp4Video::verify(&mp4_data));
    
    // MOV
    let mov_data = vec![
        0x00, 0x00, 0x00, 0x14,
        0x66, 0x74, 0x79, 0x70, // ftyp
        0x71, 0x74, 0x20, 0x20, // qt  
    ];
    assert!(MovVideo::verify(&mov_data));
    
    // MKV
    let mkv_data = vec![
        0x1A, 0x45, 0xDF, 0xA3, // EBML header
    ];
    assert!(MkvVideo::verify(&mkv_data));
}

/// Test content type detection
///
/// ```mermaid
/// graph TD
///     A[File Data] --> B{Check Signature}
///     B -->|PDF| C[PDF Type]
///     B -->|PNG| D[PNG Type]
///     B -->|MP3| E[MP3 Type]
///     B -->|Unknown| F[None]
/// ```
#[test]
fn test_content_detection() {
    // Test various formats
    let test_cases = vec![
        (b"%PDF-1.4".as_slice(), Some(ContentType::Custom(codec::PDF))),
        (b"\x89PNG\r\n\x1a\n".as_slice(), Some(ContentType::Custom(codec::PNG))),
        (b"\xFF\xD8\xFF\xE0".as_slice(), Some(ContentType::Custom(codec::JPEG))),
        (b"GIF89a".as_slice(), Some(ContentType::Custom(codec::GIF))),
        (b"ID3\x04\x00".as_slice(), Some(ContentType::Custom(codec::MP3))),
        (b"fLaC".as_slice(), Some(ContentType::Custom(codec::FLAC))),
        (b"Unknown".as_slice(), None),
    ];
    
    for (data, expected) in test_cases {
        let detected = detect_content_type(data);
        assert_eq!(detected, expected);
    }
}

/// Test content type names
#[test]
fn test_content_type_names() {
    assert_eq!(content_type_name(ContentType::Custom(codec::PDF)), "PDF Document");
    assert_eq!(content_type_name(ContentType::Custom(codec::JPEG)), "JPEG Image");
    assert_eq!(content_type_name(ContentType::Custom(codec::MP3)), "MP3 Audio");
    assert_eq!(content_type_name(ContentType::Custom(codec::MP4)), "MP4 Video");
    assert_eq!(content_type_name(ContentType::Custom(0x999999)), "Unknown");
}

/// Test cross-type CID uniqueness
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_cid_uniqueness() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Same data in different formats should have different CIDs
    let data = b"Test content data".to_vec();
    
    // As text document
    let text = TextDocument::new(
        String::from_utf8(data.clone())?,
        DocumentMetadata::default()
    )?;
    let text_cid = context.storage.put(&text).await?;
    
    // As markdown document
    let md = MarkdownDocument::new(
        String::from_utf8(data.clone())?,
        DocumentMetadata::default()
    )?;
    let md_cid = context.storage.put(&md).await?;
    
    // CIDs should be different due to different content types
    assert_ne!(text_cid, md_cid);
    
    println!("✓ Cross-type CID uniqueness verified");
    println!("  Text CID: {text_cid}");
    println!("  Markdown CID: {md_cid}");
    
    Ok(())
}

/// Test metadata preservation
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_metadata_preservation() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Create rich metadata
    let metadata = ImageMetadata {
        width: Some(1920),
        height: Some(1080),
        format: Some("PNG".to_string()),
        color_space: Some("sRGB".to_string()),
        compression: Some("lossless".to_string()),
        tags: vec!["landscape".to_string(), "nature".to_string(), "hdr".to_string()],
    };
    
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        // Minimal PNG data
    ];
    
    let png = PngImage::new(png_data, metadata.clone())?;
    let cid = context.storage.put(&png).await?;
    
    // Retrieve and verify all metadata
    let retrieved: PngImage = context.storage.get(&cid).await?;
    
    assert_eq!(retrieved.metadata.width, metadata.width);
    assert_eq!(retrieved.metadata.height, metadata.height);
    assert_eq!(retrieved.metadata.format, metadata.format);
    assert_eq!(retrieved.metadata.color_space, metadata.color_space);
    assert_eq!(retrieved.metadata.compression, metadata.compression);
    assert_eq!(retrieved.metadata.tags.len(), 3);
    assert!(retrieved.metadata.tags.contains(&"landscape".to_string()));
    
    println!("✓ Metadata preservation verified");
    
    Ok(())
}

/// Test WebP format with RIFF container
#[test]
fn test_webp_verification() {
    // Valid WebP
    let mut webp_data = b"RIFF".to_vec();
    webp_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size
    webp_data.extend_from_slice(b"WEBP");
    webp_data.extend_from_slice(b"VP8 "); // VP8 chunk
    
    assert!(WebPImage::verify(&webp_data));
    
    // Invalid - wrong container
    let invalid_riff = b"RIFF----WAVE".to_vec();
    assert!(!WebPImage::verify(&invalid_riff));
}

/// Test error handling for corrupted data
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_corrupted_data_handling() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Store valid PDF
    let pdf_data = b"%PDF-1.4\nValid PDF".to_vec();
    let pdf = PdfDocument::new(pdf_data, DocumentMetadata::default())?;
    let cid = context.storage.put(&pdf).await?;
    
    // Try to retrieve as wrong type
    let result = context.storage.get::<JpegImage>(&cid).await;
    assert!(result.is_err());
    
    // Should still work with correct type
    let correct: PdfDocument = context.storage.get(&cid).await?;
    assert!(PdfDocument::verify(&correct.data));
    
    println!("✓ Error handling for wrong types verified");
    
    Ok(())
} 