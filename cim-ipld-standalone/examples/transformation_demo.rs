//! Demonstrates content transformation capabilities in cim-ipld

use cim_ipld::{
    content_types::{
        transformers::{document, image, audio, video},
        MarkdownDocument, JpegImage, PngImage, Mp3Audio,
        DocumentMetadata, ImageMetadata, AudioMetadata,
    },
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CIM-IPLD Content Transformation Demo ===\n");

    // Demo 1: Markdown to HTML transformation
    demo_markdown_to_html()?;
    
    // Demo 2: Markdown to plain text
    demo_markdown_to_text()?;
    
    // Demo 3: Image format conversion
    demo_image_conversion()?;
    
    // Demo 4: Audio metadata extraction
    demo_audio_metadata()?;
    
    // Demo 5: Video metadata extraction
    demo_video_metadata()?;

    Ok(())
}

fn demo_markdown_to_html() -> Result<()> {
    println!("--- Demo 1: Markdown to HTML ---");
    
    let markdown = MarkdownDocument {
        content: r#"# Welcome to CIM-IPLD

This is a **bold** statement about our *amazing* transformation capabilities.

## Features

- Markdown to HTML conversion
- Image format transformation
- Audio/Video metadata extraction

### Code Example

```rust
let result = transform_content(input)?;
println!("Transformed: {:?}", result);
```

Visit [our website](https://example.com) for more info!"#.to_string(),
        metadata: DocumentMetadata {
            title: Some("CIM-IPLD Demo".to_string()),
            author: Some("CIM Team".to_string()),
            ..Default::default()
        },
    };
    
    let html = document::markdown_to_html(&markdown)?;
    println!("Original Markdown (first 100 chars):");
    println!("{&markdown.content[..100.min(markdown.content.len(}"))]);
    println!("\nConverted HTML (first 200 chars):");
    println!("{&html[..200.min(html.len(}"))]);
    println!();
    
    Ok(())
}

fn demo_markdown_to_text() -> Result<()> {
    println!("--- Demo 2: Markdown to Plain Text ---");
    
    let markdown_content = "# Title\n\nThis is **bold** and *italic* text with a [link](https://example.com).";
    
    let plain_text = document::to_plain_text(markdown_content)?;
    
    println!("Original: {markdown_content}");
    println!("Plain text: {plain_text}");
    println!();
    
    Ok(())
}

fn demo_image_conversion() -> Result<()> {
    println!("--- Demo 3: Image Format Conversion ---");
    
    // Create a simple test image (1x1 red pixel PNG)
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk size
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width = 1
        0x00, 0x00, 0x00, 0x01, // height = 1
        0x08, 0x02, 0x00, 0x00, 0x00, // 8-bit RGB
        0x90, 0x77, 0x53, 0xDE, // CRC
        0x00, 0x00, 0x00, 0x0C, // IDAT chunk size
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00,
        0x18, 0xDD, 0x8D, 0xB4, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk size
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];
    
    println!("Original PNG size: {png_data.len(} bytes"));
    
    // Convert to JPEG
    match image::convert_format(&png_data, "png", "jpeg", Some(90)) {
        Ok(jpeg_data) => {
            println!("Converted to JPEG, size: {jpeg_data.len(} bytes"));
        }
        Err(e) => {
            println!("Note: Image conversion requires valid image data");
            println!("Error: {e}");
        }
    }
    
    println!();
    Ok(())
}

fn demo_audio_metadata() -> Result<()> {
    println!("--- Demo 4: Audio Metadata Extraction ---");
    
    // Create a minimal MP3 header for demonstration
    let mp3_data = vec![
        0x49, 0x44, 0x33, // ID3 tag
        0x03, 0x00, // Version
        0x00, // Flags
        0x00, 0x00, 0x00, 0x0A, // Size
        // Minimal ID3 data
        0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    
    match audio::extract_metadata(&mp3_data, "mp3") {
        Ok(metadata) => {
            println!("Extracted metadata:");
            println!("  Codec: {:?}", metadata.codec);
            println!("  Duration: {:?} ms", metadata.duration_ms);
            println!("  Sample rate: {:?} Hz", metadata.sample_rate);
            println!("  Channels: {:?}", metadata.channels);
        }
        Err(e) => {
            println!("Note: Full metadata extraction requires valid audio files");
            println!("Error: {e}");
        }
    }
    
    println!();
    Ok(())
}

fn demo_video_metadata() -> Result<()> {
    println!("--- Demo 5: Video Metadata Extraction ---");
    
    // Create a minimal MP4 header structure
    let mp4_data = vec![
        0x00, 0x00, 0x00, 0x20, // Box size (32 bytes)
        b'f', b't', b'y', b'p', // Box type: ftyp
        b'i', b's', b'o', b'm', // Major brand
        0x00, 0x00, 0x00, 0x00, // Minor version
        b'i', b's', b'o', b'm', // Compatible brands
        b'i', b's', b'o', b'2',
        b'a', b'v', b'c', b'1',
        b'm', b'p', b'4', b'1',
    ];
    
    let metadata = video::extract_metadata(&mp4_data, "mp4")?;
    
    println!("Extracted metadata:");
    println!("  Video codec: {:?}", metadata.video_codec);
    println!("  Audio codec: {:?}", metadata.audio_codec);
    println!("  Duration: {:?} ms", metadata.duration_ms);
    println!("  Resolution: {metadata.width.unwrap_or(0}x{}"), 
        metadata.height.unwrap_or(0)
    );
    println!("  Tags: {:?}", metadata.tags);
    
    Ok(())
} 