# CIM-IPLD Content Types

## Overview

CIM-IPLD now supports typed content for common file formats with automatic verification and CID-based storage. This enables type-safe storage and retrieval of documents, images, audio, and video files through NATS JetStream.

## Supported Content Types

### Documents
- **PDF** (0x600001): Verified PDF documents with metadata
- **DOCX** (0x600002): Microsoft Word documents  
- **Markdown** (0x600003): UTF-8 markdown text
- **Text** (0x600004): Plain text documents

### Images
- **JPEG** (0x610001): JPEG images with EXIF support
- **PNG** (0x610002): PNG images with metadata
- **GIF** (0x610003): GIF images (static and animated)
- **WebP** (0x610004): WebP images

### Audio
- **MP3** (0x620001): MP3 audio with ID3 tags
- **WAV** (0x620002): WAV audio files
- **FLAC** (0x620003): FLAC lossless audio
- **AAC** (0x620004): AAC audio files
- **OGG** (0x620005): OGG Vorbis audio

### Video
- **MP4** (0x630001): MP4 video containers
- **MOV** (0x630002): QuickTime movies
- **MKV** (0x630003): Matroska video
- **AVI** (0x630004): AVI video files

## Features

### 1. **Type Verification**
Each content type includes verification to ensure files match their expected format:
```rust
let pdf_data = std::fs::read("document.pdf")?;
let pdf = PdfDocument::new(pdf_data, metadata)?; // Verifies PDF signature
```

### 2. **Rich Metadata**
All content types support metadata appropriate to their format:
- Documents: title, author, created/modified dates, tags, language
- Images: dimensions, format, color space, compression
- Audio: duration, bitrate, sample rate, artist, album, title
- Video: duration, dimensions, frame rate, codecs

### 3. **CID Consistency**
Content type information is included in CID calculation, ensuring different types of the same data have different CIDs:
```rust
// Same data, different CIDs
let text = TextDocument::new(content.clone(), metadata)?;
let markdown = MarkdownDocument::new(content.clone(), metadata)?;
// text_cid != markdown_cid
```

### 4. **Automatic Bucket Routing**
Content is automatically routed to appropriate storage buckets:
- Documents → `cim-documents`
- Images/Audio/Video → `cim-media`

### 5. **Content Detection**
Automatic content type detection from file data:
```rust
if let Some(content_type) = detect_content_type(&data) {
    let name = content_type_name(content_type);
    println!("Detected: {}", name);
}
```

## Usage Examples

### Storing a PDF Document
```rust
use cim_ipld::content_types::{PdfDocument, DocumentMetadata};

let pdf_data = std::fs::read("report.pdf")?;
let metadata = DocumentMetadata {
    title: Some("Annual Report".to_string()),
    author: Some("John Doe".to_string()),
    tags: vec!["finance".to_string(), "2024".to_string()],
    ..Default::default()
};

let pdf = PdfDocument::new(pdf_data, metadata)?;
let cid = storage.put(&pdf).await?;
```

### Storing an Image
```rust
use cim_ipld::content_types::{JpegImage, ImageMetadata};

let image_data = std::fs::read("photo.jpg")?;
let metadata = ImageMetadata {
    width: Some(1920),
    height: Some(1080),
    tags: vec!["landscape".to_string()],
    ..Default::default()
};

let jpeg = JpegImage::new(image_data, metadata)?;
let cid = storage.put(&jpeg).await?;
```

### Retrieving Typed Content
```rust
// Type-safe retrieval
let pdf: PdfDocument = storage.get(&cid).await?;
println!("Title: {:?}", pdf.metadata.title);

// Wrong type returns error
let result = storage.get::<JpegImage>(&cid).await; // Error
```

### Content Detection
```rust
let data = std::fs::read("unknown_file")?;

match detect_content_type(&data) {
    Some(ContentType::Custom(codec::PDF)) => {
        let pdf = PdfDocument::new(data, Default::default())?;
        // Process PDF
    }
    Some(ContentType::Custom(codec::JPEG)) => {
        let jpeg = JpegImage::new(data, Default::default())?;
        // Process JPEG
    }
    _ => {
        println!("Unknown content type");
    }
}
```

## Implementation Details

### Magic Bytes
Content verification uses file format magic bytes:
- PDF: `%PDF-`
- PNG: `\x89PNG\r\n\x1a\n`
- JPEG: `\xFF\xD8\xFF`
- MP3: `ID3` or `\xFF\xFB`

### Codec Ranges
Content types use reserved codec ranges:
- Documents: 0x600000 - 0x60FFFF
- Images: 0x610000 - 0x61FFFF
- Audio: 0x620000 - 0x62FFFF
- Video: 0x630000 - 0x63FFFF

### Storage Buckets
Content is organized into NATS Object Store buckets:
- `cim-documents`: Text-based content
- `cim-media`: Binary media files
- `cim-graphs`, `cim-events`, etc.: System content

## Testing

Comprehensive tests verify:
- Content type verification
- CID consistency
- Metadata preservation
- Cross-type uniqueness
- Error handling

Run tests with:
```bash
cargo test --test content_types_test
```

## Future Enhancements

Potential additions:
- Archive formats (ZIP, TAR, 7Z)
- Office formats (XLSX, PPTX)
- 3D formats (OBJ, GLTF)
- Vector graphics (SVG)
- Ebook formats (EPUB, MOBI)
- Streaming support for large files
- Metadata extraction from files
- Thumbnail generation 