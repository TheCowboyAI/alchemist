//! Example: Using CIM-IPLD content types for common file formats
//!
//! This example demonstrates:
//! 1. Creating and verifying different content types
//! 2. Storing content with proper type verification
//! 3. Retrieving and identifying content by CID
//! 4. Automatic content type detection

use cim_ipld::{
    content_types::{
        PdfDocument, MarkdownDocument,
        JpegImage, PngImage, Mp3Audio, WavAudio, Mp4Video,
        DocumentMetadata, ImageMetadata, AudioMetadata, VideoMetadata,
        detect_content_type, content_type_name,
    },
    object_store::{NatsObjectStore, ContentBucket},
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== CIM-IPLD Content Types Demo ===\n");

    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = async_nats::jetstream::new(client);
    let storage = NatsObjectStore::new(jetstream, 1024).await?;

    // Demo 1: Document types
    demo_documents(&storage).await?;
    
    // Demo 2: Image types
    demo_images(&storage).await?;
    
    // Demo 3: Audio types
    demo_audio(&storage).await?;
    
    // Demo 4: Video types
    demo_video(&storage).await?;
    
    // Demo 5: Content type detection
    demo_content_detection(&storage).await?;
    
    // Demo 6: Listing by content type
    demo_list_by_type(&storage).await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn demo_documents(storage: &NatsObjectStore) -> Result<(), Box<dyn Error>> {
    println!("1. Document Types Demo\n");
    
    // Create a PDF document
    let pdf_data = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n1 0 obj\n<< /Type /Catalog >>\nendobj";
    let pdf_metadata = DocumentMetadata {
        title: Some("Test PDF Document".to_string()),
        author: Some("CIM System".to_string()),
        created_at: Some(1234567890),
        tags: vec!["test".to_string(), "pdf".to_string()],
        ..Default::default()
    };
    
    let pdf_doc = PdfDocument::new(pdf_data.to_vec(), pdf_metadata)?;
    let pdf_cid = storage.put(&pdf_doc).await?;
    println!("✓ Stored PDF document: {pdf_cid}");
    
    // Create a Markdown document
    let markdown_content = r#"# CIM-IPLD Documentation

This is a test markdown document demonstrating content types.

## Features
- Content verification
- Type-safe storage
- CID-based retrieval
"#;
    
    let md_metadata = DocumentMetadata {
        title: Some("CIM-IPLD Docs".to_string()),
        language: Some("en".to_string()),
        tags: vec!["documentation".to_string(), "markdown".to_string()],
        ..Default::default()
    };
    
    let md_doc = MarkdownDocument::new(markdown_content.to_string(), md_metadata)?;
    let md_cid = storage.put(&md_doc).await?;
    println!("✓ Stored Markdown document: {md_cid}");
    
    // Retrieve and verify
    let retrieved_pdf: PdfDocument = storage.get(&pdf_cid).await?;
    println!("✓ Retrieved PDF, title: {:?}", retrieved_pdf.metadata.title);
    
    let retrieved_md: MarkdownDocument = storage.get(&md_cid).await?;
    println!("✓ Retrieved Markdown, {retrieved_md.content.len(} characters"));
    
    Ok(())
}

async fn demo_images(storage: &NatsObjectStore) -> Result<(), Box<dyn Error>> {
    println!("\n2. Image Types Demo\n");
    
    // Create a minimal PNG image
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        // ... minimal PNG data
    ];
    
    let png_metadata = ImageMetadata {
        width: Some(100),
        height: Some(100),
        format: Some("PNG".to_string()),
        color_space: Some("RGB".to_string()),
        tags: vec!["test".to_string(), "image".to_string()],
        ..Default::default()
    };
    
    let png_image = PngImage::new(png_data, png_metadata)?;
    let png_cid = storage.put(&png_image).await?;
    println!("✓ Stored PNG image: {png_cid}");
    
    // Create a JPEG image (minimal header)
    let jpeg_data = vec![
        0xFF, 0xD8, 0xFF, 0xE0, // JPEG SOI and APP0 marker
        0x00, 0x10, // APP0 length
        0x4A, 0x46, 0x49, 0x46, 0x00, // JFIF\0
        // ... minimal JPEG data
    ];
    
    let jpeg_metadata = ImageMetadata {
        width: Some(200),
        height: Some(150),
        format: Some("JPEG".to_string()),
        compression: Some("baseline".to_string()),
        ..Default::default()
    };
    
    let jpeg_image = JpegImage::new(jpeg_data, jpeg_metadata)?;
    let jpeg_cid = storage.put(&jpeg_image).await?;
    println!("✓ Stored JPEG image: {jpeg_cid}");
    
    // Retrieve and display metadata
    let retrieved_png: PngImage = storage.get(&png_cid).await?;
    println!("✓ Retrieved PNG: {retrieved_png.metadata.width.unwrap_or(0}x{} pixels"),
        retrieved_png.metadata.height.unwrap_or(0)
    );
    
    Ok(())
}

async fn demo_audio(storage: &NatsObjectStore) -> Result<(), Box<dyn Error>> {
    println!("\n3. Audio Types Demo\n");
    
    // Create an MP3 file (ID3 header)
    let mp3_data = vec![
        0x49, 0x44, 0x33, // ID3
        0x04, 0x00, // Version 2.4
        0x00, // Flags
        // ... minimal MP3 data
    ];
    
    let mp3_metadata = AudioMetadata {
        duration_ms: Some(180000), // 3 minutes
        bitrate: Some(320),
        sample_rate: Some(44100),
        artist: Some("CIM Test".to_string()),
        album: Some("Demo Album".to_string()),
        title: Some("Test Track".to_string()),
        ..Default::default()
    };
    
    let mp3_audio = Mp3Audio::new(mp3_data, mp3_metadata)?;
    let mp3_cid = storage.put(&mp3_audio).await?;
    println!("✓ Stored MP3 audio: {mp3_cid}");
    
    // Create a WAV file
    let mut wav_data = vec![
        0x52, 0x49, 0x46, 0x46, // RIFF
        0x24, 0x00, 0x00, 0x00, // File size
        0x57, 0x41, 0x56, 0x45, // WAVE
    ];
    // Add minimal fmt chunk
    wav_data.extend_from_slice(&[
        0x66, 0x6D, 0x74, 0x20, // "fmt "
        0x10, 0x00, 0x00, 0x00, // Chunk size
        0x01, 0x00, // Audio format (PCM)
        0x02, 0x00, // Channels
        0x44, 0xAC, 0x00, 0x00, // Sample rate
        0x10, 0xB1, 0x02, 0x00, // Byte rate
        0x04, 0x00, // Block align
        0x10, 0x00, // Bits per sample
    ]);
    
    let wav_metadata = AudioMetadata {
        duration_ms: Some(5000), // 5 seconds
        bitrate: Some(1411), // CD quality
        sample_rate: Some(44100),
        channels: Some(2),
        codec: Some("PCM".to_string()),
        ..Default::default()
    };
    
    let wav_audio = WavAudio::new(wav_data, wav_metadata)?;
    let wav_cid = storage.put(&wav_audio).await?;
    println!("✓ Stored WAV audio: {wav_cid}");
    
    // Retrieve and display info
    let retrieved_mp3: Mp3Audio = storage.get(&mp3_cid).await?;
    println!("✓ Retrieved MP3: {retrieved_mp3.metadata.artist.as_ref(} - {} ({}ms)").unwrap_or(&"Unknown".to_string()),
        retrieved_mp3.metadata.title.as_ref().unwrap_or(&"Unknown".to_string()),
        retrieved_mp3.metadata.duration_ms.unwrap_or(0)
    );
    
    Ok(())
}

async fn demo_video(storage: &NatsObjectStore) -> Result<(), Box<dyn Error>> {
    println!("\n4. Video Types Demo\n");
    
    // Create an MP4 file
    let mp4_data = vec![
        0x00, 0x00, 0x00, 0x20, // Box size
        0x66, 0x74, 0x79, 0x70, // ftyp
        0x6D, 0x70, 0x34, 0x32, // mp42
        // ... minimal MP4 data
    ];
    
    let mp4_metadata = VideoMetadata {
        duration_ms: Some(120000), // 2 minutes
        width: Some(1920),
        height: Some(1080),
        frame_rate: Some(30.0),
        video_codec: Some("H.264".to_string()),
        audio_codec: Some("AAC".to_string()),
        bitrate: Some(5000),
        ..Default::default()
    };
    
    let mp4_video = Mp4Video::new(mp4_data, mp4_metadata)?;
    let mp4_cid = storage.put(&mp4_video).await?;
    println!("✓ Stored MP4 video: {mp4_cid}");
    
    // Retrieve and display info
    let retrieved_mp4: Mp4Video = storage.get(&mp4_cid).await?;
    println!("✓ Retrieved MP4: {retrieved_mp4.metadata.width.unwrap_or(0}x{} @ {}fps"),
        retrieved_mp4.metadata.height.unwrap_or(0),
        retrieved_mp4.metadata.frame_rate.unwrap_or(0.0)
    );
    
    Ok(())
}

async fn demo_content_detection(_storage: &NatsObjectStore) -> Result<(), Box<dyn Error>> {
    println!("\n5. Content Type Detection Demo\n");
    
    // Test various file signatures
    let test_data = vec![
        (b"%PDF-1.4".to_vec(), "PDF"),
        (b"\x89PNG\r\n\x1a\n".to_vec(), "PNG"),
        (b"\xFF\xD8\xFF\xE0".to_vec(), "JPEG"),
        (b"ID3\x04\x00".to_vec(), "MP3"),
        (b"RIFF----WAVE".to_vec(), "WAV"),
    ];
    
    for (data, expected) in test_data {
        if let Some(content_type) = detect_content_type(&data) {
            let name = content_type_name(content_type);
            println!("✓ Detected {expected} from signature");
            assert!(name.contains(expected));
        }
    }
    
    // Test with unknown data
    let unknown_data = b"Unknown file format";
    if detect_content_type(unknown_data).is_none() {
        println!("✓ Unknown format correctly returns None");
    }
    
    Ok(())
}

async fn demo_list_by_type(storage: &NatsObjectStore) -> Result<(), Box<dyn Error>> {
    println!("\n6. List by Content Type Demo\n");
    
    // List documents
    let docs = storage.list(ContentBucket::Documents).await?;
    println!("Documents bucket: {docs.len(} items"));
    for (i, obj) in docs.iter().take(5).enumerate() {
        println!("  {i + 1}. {obj.cid} ({obj.size} bytes)");
    }
    
    // List media (images, audio, video)
    let media = storage.list(ContentBucket::Media).await?;
    println!("\nMedia bucket: {media.len(} items"));
    for (i, obj) in media.iter().take(5).enumerate() {
        println!("  {i + 1}. {obj.cid} ({obj.size} bytes)");
    }
    
    Ok(())
}

 