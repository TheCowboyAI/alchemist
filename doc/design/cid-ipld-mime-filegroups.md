# MIME Types and File Groups

> Part of the [CID/IPLD Architecture](./cid-ipld-architecture.md)

## Overview

CIM provides intelligent MIME type detection and file group management, enabling dynamic content handling, automatic hash function selection, and powerful collection capabilities. File groups can be exported as archives, mounted as filesystems, or deployed as container layers.

## MIME Type Intelligence

### MIME Type Registry

```rust
#[derive(Debug, Clone)]
pub struct MimeTypeRegistry {
    // MIME type to CIM codec mapping
    mime_to_codec: HashMap<String, u64>,
    // MIME type to hash function mapping
    mime_to_hasher: HashMap<String, HashFunction>,
    // MIME type to IPLD schema
    mime_to_schema: HashMap<String, IpldSchema>,
    // Pattern matching for complex MIME types
    mime_patterns: Vec<(Regex, MimeHandler)>,
}

#[derive(Debug, Clone)]
pub enum HashFunction {
    Blake3,              // Default for most content
    Blake2b256,          // For compatibility
    Sha256,              // For Git objects
    Sha3_256,            // For Ethereum compatibility
    Custom(Box<dyn Hasher>),
}

impl MimeTypeRegistry {
    pub fn create_content_addressed_object(
        &self,
        data: &[u8],
        mime_type: &str,
        metadata: Option<Metadata>,
    ) -> Result<ContentAddressedObject> {
        // Verify MIME type matches content
        let verified_mime = verify_mime_type(data, mime_type)?;

        // Select appropriate hash function based on MIME type
        let hasher = self.mime_to_hasher
            .get(&verified_mime)
            .unwrap_or(&HashFunction::Blake3);

        // Create multihash with appropriate algorithm
        let hash = match hasher {
            HashFunction::Blake3 => {
                let hash = blake3::hash(data);
                Multihash::wrap(0x1e, hash.as_bytes())?
            }
            HashFunction::Sha256 => {
                let hash = Sha256::digest(data);
                Multihash::wrap(0x12, &hash)?
            }
            HashFunction::Blake2b256 => {
                let hash = Blake2b256::digest(data);
                Multihash::wrap(0xb220, &hash)?
            }
            // ... other hash functions
        };

        // Map to CIM codec
        let codec = self.mime_to_codec
            .get(&verified_mime)
            .or_else(|| self.find_codec_by_pattern(&verified_mime))
            .unwrap_or(&CIM_GENERIC);

        // Create CID with appropriate codec
        let cid = Cid::new_v1(*codec, hash);

        // Get IPLD schema for validation
        let schema = self.mime_to_schema.get(&verified_mime);

        Ok(ContentAddressedObject {
            cid,
            mime_type: verified_mime,
            codec: *codec,
            schema: schema.cloned(),
            metadata,
        })
    }

    pub fn register_mime_mapping(
        &mut self,
        mime_type: &str,
        codec: u64,
        hasher: HashFunction,
        schema: Option<IpldSchema>,
    ) {
        self.mime_to_codec.insert(mime_type.to_string(), codec);
        self.mime_to_hasher.insert(mime_type.to_string(), hasher);
        if let Some(s) = schema {
            self.mime_to_schema.insert(mime_type.to_string(), s);
        }
    }
}
```

### MIME Type Detection

```rust
pub fn verify_mime_type(data: &[u8], claimed_mime: &str) -> Result<String> {
    // Use magic bytes detection
    let detected = tree_magic_mini::from_u8(data);

    // Special handling for text files
    if detected.starts_with("text/") || claimed_mime.starts_with("text/") {
        // Perform additional text analysis
        if let Ok(text) = std::str::from_utf8(data) {
            return Ok(detect_text_mime_type(text, claimed_mime));
        }
    }

    // Verify claimed MIME matches detected
    if claimed_mime != detected && !is_compatible_mime(&claimed_mime, &detected) {
        warn!("MIME type mismatch: claimed={}, detected={}", claimed_mime, detected);
    }

    Ok(detected)
}

fn detect_text_mime_type(text: &str, hint: &str) -> String {
    // Check for specific text formats
    if text.trim_start().starts_with("<!DOCTYPE html") || text.contains("<html") {
        return "text/html".to_string();
    }

    if text.trim_start().starts_with("<?xml") {
        return "application/xml".to_string();
    }

    if text.trim_start().starts_with("{") || text.trim_start().starts_with("[") {
        if serde_json::from_str::<serde_json::Value>(text).is_ok() {
            return "application/json".to_string();
        }
    }

    // Check for programming languages by extension hint
    if hint.contains("rust") || has_rust_patterns(text) {
        return "text/x-rust".to_string();
    }

    if hint.contains("nix") || has_nix_patterns(text) {
        return "text/x-nix".to_string();
    }

    if hint.contains("python") || has_python_patterns(text) {
        return "text/x-python".to_string();
    }

    // Default to plain text
    "text/plain".to_string()
}

fn has_rust_patterns(text: &str) -> bool {
    text.contains("fn ") || text.contains("impl ") ||
    text.contains("use ") || text.contains("pub ") ||
    text.contains("struct ") || text.contains("enum ")
}

fn has_nix_patterns(text: &str) -> bool {
    text.contains("{ pkgs") || text.contains("mkDerivation") ||
    text.contains("buildInputs") || text.contains("nixpkgs")
}

fn has_python_patterns(text: &str) -> bool {
    text.contains("def ") || text.contains("import ") ||
    text.contains("class ") || text.contains("if __name__")
}
```

### Common MIME Mappings

```rust
pub fn initialize_mime_registry() -> MimeTypeRegistry {
    let mut registry = MimeTypeRegistry::default();

    // Documents
    registry.register_mime_mapping("application/pdf", CIM_DOCUMENT, HashFunction::Blake3, None);
    registry.register_mime_mapping("text/markdown", CIM_DOCUMENT, HashFunction::Blake3, None);
    registry.register_mime_mapping("text/html", CIM_DOCUMENT, HashFunction::Blake3, None);
    registry.register_mime_mapping("application/json", CIM_DOCUMENT, HashFunction::Blake3, None);

    // Media
    registry.register_mime_mapping("video/mp4", CIM_VIDEO, HashFunction::Blake3, None);
    registry.register_mime_mapping("audio/mpeg", CIM_AUDIO, HashFunction::Blake3, None);
    registry.register_mime_mapping("image/png", CIM_IMAGE, HashFunction::Blake3, None);
    registry.register_mime_mapping("image/jpeg", CIM_IMAGE, HashFunction::Blake3, None);

    // Code and configs
    registry.register_mime_mapping("text/x-rust", CIM_SOURCE_CODE, HashFunction::Blake3, None);
    registry.register_mime_mapping("text/x-nix", CIM_NIX_CONFIG, HashFunction::Blake3, None);
    registry.register_mime_mapping("application/x-git-object", CIM_GIT_OBJECT, HashFunction::Sha256, None);

    // Archives
    registry.register_mime_mapping("application/zip", CIM_ARCHIVE, HashFunction::Blake3, None);
    registry.register_mime_mapping("application/x-tar", CIM_ARCHIVE, HashFunction::Blake3, None);

    // Pattern-based mappings
    registry.add_pattern(r"^text/.*", |mime| {
        MimeHandler {
            codec: CIM_DOCUMENT,
            hasher: HashFunction::Blake3,
            preprocessor: Some(Box::new(normalize_line_endings)),
        }
    });

    registry.add_pattern(r"^application/.*\+json$", |mime| {
        MimeHandler {
            codec: CIM_DOCUMENT,
            hasher: HashFunction::Blake3,
            preprocessor: Some(Box::new(validate_json)),
        }
    });

    registry
}
```

## File Groups

### File Group Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileGroup {
    pub group_id: Cid,
    pub name: String,
    pub group_type: FileGroupType,
    pub files: Vec<FileEntry>,
    pub metadata: FileGroupMetadata,
    pub relationships: Vec<GroupRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileGroupType {
    // Filesystem Hierarchy Standard
    FhsDirectory {
        mount_point: PathBuf,
        permissions: UnixPermissions,
    },

    // Archive formats
    ZipArchive {
        compression: CompressionType,
        encryption: Option<EncryptionType>,
    },
    TarArchive {
        compression: Option<CompressionType>,
    },

    // Container filesystems
    ContainerLayer {
        base_image: Option<Cid>,
        dockerfile_cid: Option<Cid>,
    },

    // Nix store paths
    NixStorePath {
        derivation: Cid,
        outputs: Vec<String>,
    },

    // Project structures
    ProjectWorkspace {
        project_type: ProjectType,
        build_system: BuildSystem,
    },

    // Media collections
    MediaBundle {
        primary_media: Cid,
        subtitles: Vec<Cid>,
        thumbnails: Vec<Cid>,
        metadata: MediaMetadata,
    },

    // Document sets
    DocumentCollection {
        collection_type: CollectionType,
        index: Option<Cid>,
        toc: Option<TableOfContents>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub cid: Cid,
    pub mime_type: String,
    pub size: u64,
    pub permissions: Option<UnixPermissions>,
    pub attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRelationship {
    pub relationship_type: RelationshipType,
    pub target_group: Cid,
    pub metadata: HashMap<String, Value>,
}
```

### File Group Operations

```rust
impl FileGroup {
    pub async fn create_from_directory(
        path: &Path,
        group_type: FileGroupType,
        stores: &mut Stores,
        mime_registry: &MimeTypeRegistry,
    ) -> Result<Self> {
        let mut files = Vec::new();

        // Walk directory recursively
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let relative_path = file_path.strip_prefix(path)?;

                // Read file content
                let content = tokio::fs::read(file_path).await?;

                // Detect MIME type
                let mime_type = detect_mime_type(&content, file_path)?;

                // Create content-addressed object
                let cao = mime_registry.create_content_addressed_object(
                    &content,
                    &mime_type,
                    None,
                )?;

                // Store in object store
                stores.object_store.put(cao.cid, content).await?;

                // Create file entry
                files.push(FileEntry {
                    path: relative_path.to_path_buf(),
                    cid: cao.cid,
                    mime_type,
                    size: entry.metadata()?.len(),
                    permissions: extract_permissions(&entry.metadata()?),
                    attributes: HashMap::new(),
                });
            }
        }

        // Create group metadata
        let metadata = FileGroupMetadata {
            created_at: SystemTime::now(),
            total_size: files.iter().map(|f| f.size).sum(),
            file_count: files.len(),
            source_path: Some(path.to_path_buf()),
        };

        // Create group
        let group = FileGroup {
            group_id: Cid::default(), // Will be set when stored
            name: path.file_name()
                .unwrap_or(OsStr::new("unnamed"))
                .to_string_lossy()
                .to_string(),
            group_type,
            files,
            metadata,
            relationships: Vec::new(),
        };

        // Store group and get CID
        let group_bytes = serde_json::to_vec(&group)?;
        let group_cid = create_typed_cid(&TypedContent::FileGroup(group.clone()))?;
        stores.object_store.put(group_cid, group_bytes).await?;

        Ok(FileGroup { group_id: group_cid, ..group })
    }

    pub async fn export_as_zip(&self, stores: &Stores) -> Result<Vec<u8>> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

        for file in &self.files {
            // Retrieve file content
            let content = stores.object_store.get(file.cid).await?;

            // Add to zip
            let options = FileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .unix_permissions(file.permissions.map(|p| p.mode).unwrap_or(0o644));

            zip.start_file(file.path.to_string_lossy(), options)?;
            zip.write_all(&content)?;
        }

        let cursor = zip.finish()?;
        Ok(cursor.into_inner())
    }

    pub async fn mount_as_fhs(
        &self,
        mount_point: &Path,
        stores: &Stores,
    ) -> Result<()> {
        // Create mount point
        tokio::fs::create_dir_all(mount_point).await?;

        for file in &self.files {
            let target_path = mount_point.join(&file.path);

            // Create parent directories
            if let Some(parent) = target_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // Retrieve and write file
            let content = stores.object_store.get(file.cid).await?;
            tokio::fs::write(&target_path, content).await?;

            // Set permissions if available
            if let Some(perms) = &file.permissions {
                set_unix_permissions(&target_path, perms)?;
            }
        }

        Ok(())
    }

    pub fn create_container_layer(&self) -> Result<ContainerLayer> {
        // Convert file group to container layer format
        let layer = ContainerLayer {
            files: self.files.clone(),
            whiteouts: Vec::new(),
            opaque_dirs: Vec::new(),
            metadata: ContainerLayerMetadata {
                created: self.metadata.created_at,
                author: "CIM".to_string(),
                comment: format!("Generated from file group: {}", self.name),
            },
        };

        Ok(layer)
    }
}
```

### Intelligent File Grouping

```rust
pub struct FileGroupBuilder {
    files: Vec<(PathBuf, Cid, Metadata)>,
    relationships: HashMap<Cid, Vec<Cid>>,
    group_type: Option<FileGroupType>,
}

impl FileGroupBuilder {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            relationships: HashMap::new(),
            group_type: None,
        }
    }

    pub fn add_file(&mut self, path: PathBuf, cid: Cid, metadata: Metadata) {
        self.files.push((path, cid, metadata));
    }

    pub fn add_relationship(&mut self, from: Cid, to: Cid) {
        self.relationships.entry(from).or_insert_with(Vec::new).push(to);
    }

    pub fn infer_group_type(&mut self) -> FileGroupType {
        // Check for Dockerfile
        if self.files.iter().any(|(p, _, _)| p.file_name() == Some(OsStr::new("Dockerfile"))) {
            return FileGroupType::ContainerLayer {
                base_image: None,
                dockerfile_cid: None,
            };
        }

        // Check for Nix files
        if self.files.iter().any(|(p, _, _)| p.extension() == Some(OsStr::new("nix"))) {
            return FileGroupType::NixStorePath {
                derivation: Cid::default(),
                outputs: vec!["out".to_string()],
            };
        }

        // Check for project files
        if self.files.iter().any(|(p, _, _)| {
            matches!(p.file_name().and_then(|n| n.to_str()),
                Some("Cargo.toml") | Some("package.json") | Some("pom.xml"))
        }) {
            return FileGroupType::ProjectWorkspace {
                project_type: self.detect_project_type(),
                build_system: self.detect_build_system(),
            };
        }

        // Check for media bundle
        if self.has_media_bundle_structure() {
            return self.create_media_bundle_type();
        }

        // Default to FHS directory
        FileGroupType::FhsDirectory {
            mount_point: PathBuf::from("/"),
            permissions: UnixPermissions::default(),
        }
    }

    fn detect_project_type(&self) -> ProjectType {
        if self.files.iter().any(|(p, _, _)| p.file_name() == Some(OsStr::new("Cargo.toml"))) {
            ProjectType::Rust
        } else if self.files.iter().any(|(p, _, _)| p.file_name() == Some(OsStr::new("package.json"))) {
            ProjectType::JavaScript
        } else if self.files.iter().any(|(p, _, _)| p.file_name() == Some(OsStr::new("pom.xml"))) {
            ProjectType::Java
        } else {
            ProjectType::Unknown
        }
    }

    fn detect_build_system(&self) -> BuildSystem {
        if self.files.iter().any(|(p, _, _)| p.file_name() == Some(OsStr::new("Cargo.toml"))) {
            BuildSystem::Cargo
        } else if self.files.iter().any(|(p, _, _)| p.file_name() == Some(OsStr::new("package.json"))) {
            BuildSystem::Npm
        } else if self.files.iter().any(|(p, _, _)| p.file_name() == Some(OsStr::new("Makefile"))) {
            BuildSystem::Make
        } else {
            BuildSystem::Unknown
        }
    }
}
```

## Archive Support

### Archive Creation

```rust
pub struct ArchiveCreator {
    mime_registry: Arc<MimeTypeRegistry>,
}

impl ArchiveCreator {
    pub async fn create_archive(
        &self,
        file_group: &FileGroup,
        format: ArchiveFormat,
        stores: &Stores,
    ) -> Result<Vec<u8>> {
        match format {
            ArchiveFormat::Zip => {
                file_group.export_as_zip(stores).await
            }
            ArchiveFormat::Tar => {
                self.create_tar_archive(file_group, stores).await
            }
            ArchiveFormat::TarGz => {
                let tar = self.create_tar_archive(file_group, stores).await?;
                Ok(compress_gzip(&tar)?)
            }
            ArchiveFormat::TarBz2 => {
                let tar = self.create_tar_archive(file_group, stores).await?;
                Ok(compress_bzip2(&tar)?)
            }
        }
    }

    async fn create_tar_archive(
        &self,
        file_group: &FileGroup,
        stores: &Stores,
    ) -> Result<Vec<u8>> {
        let mut tar = tar::Builder::new(Vec::new());

        for file in &file_group.files {
            let content = stores.object_store.get(file.cid).await?;

            let mut header = tar::Header::new_gnu();
            header.set_path(&file.path)?;
            header.set_size(content.len() as u64);
            header.set_mode(file.permissions.map(|p| p.mode).unwrap_or(0o644));
            header.set_cksum();

            tar.append(&header, &content[..])?;
        }

        Ok(tar.into_inner()?)
    }
}
```

## Container Integration

### Container Layer Generation

```rust
pub struct ContainerLayerBuilder {
    file_groups: Vec<FileGroup>,
    base_image: Option<Cid>,
}

impl ContainerLayerBuilder {
    pub async fn build_layer(&self, stores: &Stores) -> Result<ContainerLayer> {
        let mut layer_files = Vec::new();
        let mut seen_paths = HashSet::new();

        // Merge file groups
        for group in &self.file_groups {
            for file in &group.files {
                if !seen_paths.contains(&file.path) {
                    layer_files.push(file.clone());
                    seen_paths.insert(file.path.clone());
                }
            }
        }

        // Create layer metadata
        let layer = ContainerLayer {
            files: layer_files,
            whiteouts: Vec::new(),
            opaque_dirs: Vec::new(),
            metadata: ContainerLayerMetadata {
                created: SystemTime::now(),
                author: "CIM".to_string(),
                comment: "Generated from file groups".to_string(),
            },
        };

        Ok(layer)
    }

    pub async fn export_as_oci_layer(&self, stores: &Stores) -> Result<Vec<u8>> {
        let layer = self.build_layer(stores).await?;

        // Create OCI layer tar
        let mut tar = tar::Builder::new(Vec::new());

        for file in &layer.files {
            let content = stores.object_store.get(file.cid).await?;

            let mut header = tar::Header::new_gnu();
            header.set_path(&file.path)?;
            header.set_size(content.len() as u64);
            header.set_mode(file.permissions.map(|p| p.mode).unwrap_or(0o644));
            header.set_cksum();

            tar.append(&header, &content[..])?;
        }

        let tar_data = tar.into_inner()?;
        Ok(compress_gzip(&tar_data)?)
    }
}
```

## Related Documents

- [Content Types and Codecs](./cid-ipld-content-types.md) - Type definitions
- [Content Transformations](./cid-ipld-transformations.md) - Format conversions
- [IPLD Relationships](./cid-ipld-relationships.md) - File relationships
- [Infrastructure Tracking](./cid-ipld-infrastructure.md) - Project detection

## Next Steps

1. Configure MIME type mappings for your content
2. Set up file group creation from directories
3. Implement archive export functionality
4. Enable container layer generation
