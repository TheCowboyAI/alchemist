# Content Types and IPLD Codecs

> Part of the [CID/IPLD Architecture](./cid-ipld-architecture.md)

## Overview

CIM defines domain-specific IPLD codecs to enable type-safe content addressing. Each content type has a unique codec identifier that allows automatic routing, processing, and semantic understanding.

## Codec Range Allocation

CIM uses the IPLD private use range (0x300000-0x3FFFFF) for domain-specific codecs:

```rust
// Base codec for all CIM types
pub const CIM_CODEC_BASE: u64 = 0x300000;

// Event-related codecs (0x300000-0x30000F)
pub const CIM_EVENT: u64 = 0x300000;
pub const CIM_COMMAND: u64 = 0x300001;
pub const CIM_QUERY: u64 = 0x300002;
pub const CIM_AGGREGATE: u64 = 0x300003;
pub const CIM_PROJECTION: u64 = 0x300004;
pub const CIM_SAGA: u64 = 0x300005;
pub const CIM_POLICY: u64 = 0x300006;

// Media codecs (0x300010-0x30001F)
pub const CIM_AUDIO: u64 = 0x300010;
pub const CIM_VIDEO: u64 = 0x300011;
pub const CIM_IMAGE: u64 = 0x300012;
pub const CIM_3D_MODEL: u64 = 0x300013;
pub const CIM_ANIMATION: u64 = 0x300014;

// Document codecs (0x300020-0x30002F)
pub const CIM_DOCUMENT: u64 = 0x300020;
pub const CIM_SPREADSHEET: u64 = 0x300021;
pub const CIM_PRESENTATION: u64 = 0x300022;
pub const CIM_TRANSCRIPT: u64 = 0x300023;
pub const CIM_SUBTITLE: u64 = 0x300024;

// Knowledge codecs (0x300030-0x30003F)
pub const CIM_EMBEDDING: u64 = 0x300030;
pub const CIM_KNOWLEDGE_GRAPH: u64 = 0x300031;
pub const CIM_CONCEPT: u64 = 0x300032;
pub const CIM_RELATIONSHIP: u64 = 0x300033;

// Infrastructure codecs (0x300040-0x30004F)
pub const CIM_SOURCE_CODE: u64 = 0x300040;
pub const CIM_NIX_CONFIG: u64 = 0x300041;
pub const CIM_GIT_OBJECT: u64 = 0x300042;
pub const CIM_CONTAINER_IMAGE: u64 = 0x300043;
pub const CIM_BUILD_ARTIFACT: u64 = 0x300044;

// Graph and visualization codecs (0x300050-0x30005F)
pub const CIM_GRAPH: u64 = 0x300050;
pub const CIM_DAG_NODE: u64 = 0x300051;
pub const CIM_WORKFLOW: u64 = 0x300052;
pub const CIM_VISUALIZATION: u64 = 0x300053;
```

## Content Type Enumeration

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    // Events and Commands
    Event(EventType),
    Command(CommandType),
    Query(QueryType),

    // Media
    Audio(AudioFormat),
    Video(VideoFormat),
    Image(ImageFormat),

    // Documents
    Document(DocumentFormat),
    Transcript(TranscriptFormat),

    // Knowledge
    Embedding(EmbeddingType),
    Graph(GraphType),

    // Infrastructure
    SourceCode(Language),
    NixConfig(NixConfigType),
    GitObject(GitObjectType),

    // Generic
    Raw(Vec<u8>),
    Json(serde_json::Value),
}

impl ContentType {
    pub fn codec(&self) -> u64 {
        match self {
            ContentType::Event(_) => CIM_EVENT,
            ContentType::Command(_) => CIM_COMMAND,
            ContentType::Query(_) => CIM_QUERY,
            ContentType::Audio(_) => CIM_AUDIO,
            ContentType::Video(_) => CIM_VIDEO,
            ContentType::Image(_) => CIM_IMAGE,
            ContentType::Document(_) => CIM_DOCUMENT,
            ContentType::Transcript(_) => CIM_TRANSCRIPT,
            ContentType::Embedding(_) => CIM_EMBEDDING,
            ContentType::Graph(_) => CIM_GRAPH,
            ContentType::SourceCode(_) => CIM_SOURCE_CODE,
            ContentType::NixConfig(_) => CIM_NIX_CONFIG,
            ContentType::GitObject(_) => CIM_GIT_OBJECT,
            ContentType::Raw(_) => 0x55, // IPLD raw codec
            ContentType::Json(_) => 0x0200, // IPLD JSON codec
        }
    }
}
```

## Format Specifications

### Document Formats

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentFormat {
    Markdown,
    Html,
    Pdf,
    Docx,
    Odt,
    Txt,
    Rtf,
    Latex,
    Epub,
    Xml,
    Json,
    Yaml,
    Toml,
}

impl DocumentFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            DocumentFormat::Markdown => "text/markdown",
            DocumentFormat::Html => "text/html",
            DocumentFormat::Pdf => "application/pdf",
            DocumentFormat::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            DocumentFormat::Json => "application/json",
            // ... etc
        }
    }
}
```

### Media Formats

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VideoFormat {
    Mp4,
    Webm,
    Avi,
    Mkv,
    Mov,
    Flv,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Flac,
    Ogg,
    M4a,
    Opus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Svg,
    Bmp,
    Tiff,
}
```

### Infrastructure Types

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    CSharp,
    Cpp,
    Nix,
    Shell,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NixConfigType {
    Flake,
    Module,
    Package,
    Overlay,
    DevShell,
    NixOSConfig,
    HomeManagerConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
    Branch,
}
```

## Type-Safe Content Creation

```rust
pub fn create_typed_cid<T: TypedContent>(content: &T) -> Result<Cid> {
    let bytes = content.to_bytes()?;
    let hash = blake3::hash(&bytes);
    let mh = Multihash::wrap(0x1e, hash.as_bytes())?;
    Ok(Cid::new_v1(content.codec(), mh))
}

// Example implementations
impl TypedContent for DomainEvent {
    fn content_type(&self) -> ContentType {
        ContentType::Event(self.event_type())
    }

    fn codec(&self) -> u64 {
        CIM_EVENT
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

impl TypedContent for MarkdownDocument {
    fn content_type(&self) -> ContentType {
        ContentType::Document(DocumentFormat::Markdown)
    }

    fn codec(&self) -> u64 {
        CIM_DOCUMENT
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.content.as_bytes().to_vec())
    }
}
```

## Content Type Benefits

1. **Automatic Routing**: Content routes to appropriate handlers based on codec
2. **Type Safety**: Compile-time guarantees about content structure
3. **Semantic Understanding**: Types carry meaning for AI processing
4. **Efficient Storage**: Type-specific compression and optimization
5. **Rich Metadata**: Each type can have specific metadata fields

## Codec Registry

```rust
pub struct CodecRegistry {
    codecs: HashMap<u64, CodecInfo>,
    mime_mappings: HashMap<String, u64>,
}

pub struct CodecInfo {
    pub codec: u64,
    pub name: String,
    pub description: String,
    pub handler: Box<dyn CodecHandler>,
}

impl CodecRegistry {
    pub fn register_cim_codecs(&mut self) {
        self.register(CodecInfo {
            codec: CIM_EVENT,
            name: "CIM Event".to_string(),
            description: "Domain events in event sourcing".to_string(),
            handler: Box::new(EventCodecHandler),
        });

        // Register all other codecs...
    }
}
```

## Related Documents

- [Core CID/IPLD Implementation](./cid-ipld-core.md) - Basic CID creation
- [Content Transformations](./cid-ipld-transformations.md) - Converting between types
- [MIME Type Intelligence](./cid-ipld-mime-filegroups.md) - Automatic type detection

## Next Steps

1. Define custom codecs for your domain
2. Implement TypedContent for your types
3. Register codecs in the registry
4. Set up type-specific handlers
