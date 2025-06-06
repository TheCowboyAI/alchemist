# Git Hash and CID Isomorphic Translation

> Part of the [CID/IPLD Architecture](./cid-ipld-architecture.md)

## Overview

CIM provides first-class isomorphic translation between Git object hashes and IPLD CIDs, enabling seamless integration with existing Git repositories while maintaining the benefits of content-addressed storage. This allows Git commits to serve as Content Identifiers in the CIM ecosystem.

## Git Object Model

### Git Hash Structure

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum GitObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

#[derive(Debug, Clone)]
pub struct GitObject {
    pub object_type: GitObjectType,
    pub size: usize,
    pub content: Vec<u8>,
    pub git_hash: GitHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GitHash([u8; 20]); // SHA-1 hash

impl GitHash {
    pub fn from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 20 {
            return Err(Error::InvalidGitHash);
        }
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&bytes);
        Ok(GitHash(hash))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}
```

### Git Object Encoding

```rust
impl GitObject {
    /// Encode object in Git's internal format
    pub fn encode(&self) -> Vec<u8> {
        let type_str = match self.object_type {
            GitObjectType::Blob => "blob",
            GitObjectType::Tree => "tree",
            GitObjectType::Commit => "commit",
            GitObjectType::Tag => "tag",
        };

        let header = format!("{} {}\0", type_str, self.content.len());
        let mut encoded = header.as_bytes().to_vec();
        encoded.extend_from_slice(&self.content);
        encoded
    }

    /// Calculate Git hash for object
    pub fn calculate_hash(&self) -> GitHash {
        let encoded = self.encode();
        let hash = Sha1::digest(&encoded);
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&hash);
        GitHash(bytes)
    }
}
```

## Isomorphic Translation

### Git to CID Translation

```rust
pub struct GitCidTranslator {
    /// Git codec in IPLD multicodec table
    git_codec: u64, // 0x78 for git-raw
}

impl GitCidTranslator {
    pub fn new() -> Self {
        Self {
            git_codec: 0x78, // git-raw codec
        }
    }

    /// Convert Git hash to CID
    pub fn git_to_cid(&self, git_hash: &GitHash) -> Result<Cid> {
        // Git uses SHA-1, which is 0x11 in multihash
        let mh = Multihash::wrap(0x11, &git_hash.0)?;

        // Use git-raw codec for direct Git object storage
        Ok(Cid::new_v1(self.git_codec, mh))
    }

    /// Convert CID back to Git hash
    pub fn cid_to_git(&self, cid: &Cid) -> Result<GitHash> {
        // Verify it's a Git object
        if cid.codec() != self.git_codec {
            return Err(Error::NotGitObject);
        }

        // Extract multihash
        let mh = cid.hash();

        // Verify it's SHA-1
        if mh.code() != 0x11 {
            return Err(Error::NotSha1Hash);
        }

        // Extract digest
        let digest = mh.digest();
        if digest.len() != 20 {
            return Err(Error::InvalidDigestLength);
        }

        let mut hash = [0u8; 20];
        hash.copy_from_slice(digest);
        Ok(GitHash(hash))
    }
}
```

### Bidirectional Verification

```rust
impl GitCidTranslator {
    /// Verify that translation is truly isomorphic
    pub fn verify_isomorphism(&self, git_hash: &GitHash) -> Result<bool> {
        let cid = self.git_to_cid(git_hash)?;
        let recovered_hash = self.cid_to_git(&cid)?;
        Ok(git_hash == &recovered_hash)
    }

    /// Batch translation with verification
    pub fn translate_batch(
        &self,
        git_hashes: &[GitHash],
    ) -> Result<Vec<(GitHash, Cid)>> {
        git_hashes
            .iter()
            .map(|hash| {
                let cid = self.git_to_cid(hash)?;
                // Verify round-trip
                let recovered = self.cid_to_git(&cid)?;
                if hash != &recovered {
                    return Err(Error::IsomorphismViolation);
                }
                Ok((hash.clone(), cid))
            })
            .collect()
    }
}
```

## Git Object Storage in IPLD

### Enhanced Git Object Representation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpldGitObject {
    /// Original Git hash for reference
    pub git_hash: String,

    /// Git object type
    pub object_type: GitObjectType,

    /// Parsed content based on type
    pub content: GitObjectContent,

    /// Additional metadata
    pub metadata: GitObjectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitObjectContent {
    Blob {
        data: Vec<u8>,
        mime_type: Option<String>,
    },
    Tree {
        entries: Vec<TreeEntry>,
    },
    Commit {
        tree: String,
        parents: Vec<String>,
        author: GitSignature,
        committer: GitSignature,
        message: String,
        gpg_signature: Option<String>,
    },
    Tag {
        object: String,
        object_type: GitObjectType,
        tag_name: String,
        tagger: GitSignature,
        message: String,
        gpg_signature: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitObjectMetadata {
    /// When this object was imported into CIM
    pub imported_at: SystemTime,

    /// Repository it came from
    pub repository: RepositoryInfo,

    /// Related CIDs (e.g., parent commits as CIDs)
    pub related_cids: Vec<Cid>,

    /// Semantic tags
    pub tags: Vec<String>,
}
```

### Dual Storage Strategy

```rust
pub struct GitObjectStore {
    /// Raw Git object storage (for exact Git compatibility)
    raw_store: Arc<dyn ObjectStore>,

    /// Enhanced IPLD storage (for rich queries)
    ipld_store: Arc<dyn ObjectStore>,

    /// Translation layer
    translator: GitCidTranslator,

    /// Bidirectional index
    index: GitCidIndex,
}

impl GitObjectStore {
    /// Store Git object with dual representation
    pub async fn store_git_object(
        &mut self,
        git_object: &GitObject,
    ) -> Result<(GitHash, Cid, Cid)> {
        // Calculate Git hash
        let git_hash = git_object.calculate_hash();

        // Store raw Git object
        let raw_cid = self.store_raw_git_object(git_object).await?;

        // Parse and store as IPLD
        let ipld_object = self.parse_to_ipld(git_object)?;
        let ipld_cid = self.store_ipld_object(&ipld_object).await?;

        // Update index
        self.index.add_mapping(git_hash.clone(), raw_cid, ipld_cid)?;

        Ok((git_hash, raw_cid, ipld_cid))
    }

    /// Retrieve by either Git hash or CID
    pub async fn get(&self, id: &ContentId) -> Result<GitObject> {
        match id {
            ContentId::GitHash(hash) => self.get_by_git_hash(hash).await,
            ContentId::Cid(cid) => self.get_by_cid(cid).await,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ContentId {
    GitHash(GitHash),
    Cid(Cid),
}
```

## Git Commit Chain Integration

### Commit Chain as CID Chain

```rust
pub struct GitCommitChain {
    /// Head commit (can be Git hash or CID)
    pub head: ContentId,

    /// Chain of commits
    pub commits: Vec<CommitNode>,

    /// Merkle proof for chain integrity
    pub merkle_root: Cid,
}

#[derive(Debug, Clone)]
pub struct CommitNode {
    /// Git hash of this commit
    pub git_hash: GitHash,

    /// CID of this commit
    pub cid: Cid,

    /// Parent commits (as CIDs)
    pub parents: Vec<Cid>,

    /// Commit metadata
    pub metadata: CommitMetadata,
}

impl GitCommitChain {
    /// Build chain from Git repository
    pub async fn from_git_repo(
        repo: &Repository,
        head_ref: &str,
        depth: Option<usize>,
    ) -> Result<Self> {
        let mut chain = Vec::new();
        let mut current = repo.head()?.peel_to_commit()?;
        let translator = GitCidTranslator::new();

        let mut count = 0;
        loop {
            // Convert Git commit to CID
            let git_hash = GitHash::from_hex(&current.id().to_string())?;
            let cid = translator.git_to_cid(&git_hash)?;

            // Get parent CIDs
            let parent_cids: Vec<Cid> = current
                .parent_ids()
                .map(|id| {
                    let hash = GitHash::from_hex(&id.to_string())?;
                    translator.git_to_cid(&hash)
                })
                .collect::<Result<Vec<_>>>()?;

            chain.push(CommitNode {
                git_hash,
                cid,
                parents: parent_cids,
                metadata: extract_commit_metadata(&current)?,
            });

            // Check depth limit
            count += 1;
            if let Some(max_depth) = depth {
                if count >= max_depth {
                    break;
                }
            }

            // Move to parent
            if current.parent_count() == 0 {
                break;
            }
            current = current.parent(0)?;
        }

        // Calculate Merkle root
        let merkle_root = calculate_chain_merkle_root(&chain)?;

        Ok(GitCommitChain {
            head: ContentId::GitHash(chain[0].git_hash.clone()),
            commits: chain,
            merkle_root,
        })
    }
}
```

## Seamless Git Integration

### Git Remote Helper

```rust
/// Git remote helper for CIM integration
pub struct CimGitRemote {
    store: Arc<GitObjectStore>,
    translator: GitCidTranslator,
}

impl CimGitRemote {
    /// Handle git-upload-pack (for git fetch/clone)
    pub async fn upload_pack(&self, want: &[GitHash]) -> Result<PackFile> {
        let mut objects = Vec::new();

        for hash in want {
            // Translate to CID
            let cid = self.translator.git_to_cid(hash)?;

            // Fetch from store
            let object = self.store.get(&ContentId::Cid(cid)).await?;
            objects.push(object);

            // Recursively fetch dependencies
            self.fetch_dependencies(&mut objects, &object).await?;
        }

        // Build pack file
        Ok(build_pack_file(objects))
    }

    /// Handle git-receive-pack (for git push)
    pub async fn receive_pack(&mut self, pack: PackFile) -> Result<()> {
        let objects = parse_pack_file(pack)?;

        for object in objects {
            // Store with dual representation
            self.store.store_git_object(&object).await?;
        }

        Ok(())
    }
}
```

### Git URL Scheme

```rust
/// Support for cim:// URL scheme in Git
pub struct CimUrlHandler;

impl CimUrlHandler {
    /// Parse CIM URLs for Git
    /// Format: cim://[node]/[namespace]/[repo]
    /// Example: cim://gateway.cim.network/myorg/myrepo
    pub fn parse_url(url: &str) -> Result<CimRepoAddress> {
        if !url.starts_with("cim://") {
            return Err(Error::InvalidCimUrl);
        }

        let parts: Vec<&str> = url[6..].split('/').collect();
        if parts.len() < 3 {
            return Err(Error::InvalidCimUrl);
        }

        Ok(CimRepoAddress {
            node: parts[0].to_string(),
            namespace: parts[1].to_string(),
            repository: parts[2].to_string(),
        })
    }

    /// Convert Git hash to CIM URL
    /// Example: cim://content/git/abc123...
    pub fn content_url(content_id: &ContentId) -> String {
        match content_id {
            ContentId::GitHash(hash) => {
                format!("cim://content/git/{}", hash.to_hex())
            }
            ContentId::Cid(cid) => {
                format!("cim://content/ipld/{}", cid)
            }
        }
    }
}
```

## Performance Optimizations

### Caching Layer

```rust
pub struct GitCidCache {
    /// LRU cache for Git hash to CID mappings
    git_to_cid: LruCache<GitHash, Cid>,

    /// LRU cache for CID to Git hash mappings
    cid_to_git: LruCache<Cid, GitHash>,

    /// Bloom filter for existence checks
    bloom_filter: BloomFilter,
}

impl GitCidCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            git_to_cid: LruCache::new(capacity),
            cid_to_git: LruCache::new(capacity),
            bloom_filter: BloomFilter::new(capacity * 10, 0.01),
        }
    }

    pub fn get_cid(&mut self, git_hash: &GitHash) -> Option<Cid> {
        if !self.bloom_filter.might_contain(&git_hash.0) {
            return None;
        }
        self.git_to_cid.get(git_hash).cloned()
    }

    pub fn insert(&mut self, git_hash: GitHash, cid: Cid) {
        self.bloom_filter.insert(&git_hash.0);
        self.git_to_cid.put(git_hash.clone(), cid);
        self.cid_to_git.put(cid, git_hash);
    }
}
```

### Batch Operations

```rust
impl GitObjectStore {
    /// Efficient batch import from Git repository
    pub async fn import_repository(
        &mut self,
        repo_path: &Path,
        progress: Option<Box<dyn ProgressReporter>>,
    ) -> Result<ImportStats> {
        let repo = Repository::open(repo_path)?;
        let odb = repo.odb()?;

        let mut stats = ImportStats::default();
        let mut batch = Vec::new();

        // Walk all objects
        odb.foreach(|oid| {
            let object = odb.read(oid)?;
            let git_object = parse_git_object(&object)?;

            batch.push(git_object);

            // Process in batches
            if batch.len() >= 1000 {
                self.store_batch(&batch).await?;
                stats.objects_imported += batch.len();

                if let Some(ref progress) = progress {
                    progress.report(stats.clone());
                }

                batch.clear();
            }

            true // Continue iteration
        })?;

        // Process remaining
        if !batch.is_empty() {
            self.store_batch(&batch).await?;
            stats.objects_imported += batch.len();
        }

        Ok(stats)
    }
}
```

## Usage Examples

### Basic Translation

```rust
// Create translator
let translator = GitCidTranslator::new();

// Convert Git hash to CID
let git_hash = GitHash::from_hex("d670460b4b4aece5915caf5c68d12f560a9fe3e4")?;
let cid = translator.git_to_cid(&git_hash)?;
println!("Git {} -> CID {}", git_hash.to_hex(), cid);

// Convert back
let recovered = translator.cid_to_git(&cid)?;
assert_eq!(git_hash, recovered);
```

### Repository Integration

```rust
// Import entire repository
let store = GitObjectStore::new(object_store, translator);
let stats = store.import_repository(Path::new("/path/to/repo"), None).await?;
println!("Imported {} objects", stats.objects_imported);

// Query by either Git hash or CID
let commit = store.get(&ContentId::GitHash(git_hash)).await?;
let same_commit = store.get(&ContentId::Cid(cid)).await?;
```

### Git Remote Usage

```bash
# Add CIM remote to existing repo
git remote add cim cim://gateway.cim.network/myorg/myrepo

# Push to CIM
git push cim main

# Clone from CIM
git clone cim://gateway.cim.network/myorg/myrepo
```

## Related Documents

- [Infrastructure Tracking](./cid-ipld-infrastructure.md) - Git repository tracking
- [Content Types](./cid-ipld-content-types.md) - Git object codecs
- [Event and Object Stores](./cid-ipld-stores.md) - Storage architecture
- [Core Architecture](./cid-ipld-core.md) - CID fundamentals

## Next Steps

1. Implement Git object parser and encoder
2. Create isomorphic translation layer
3. Build Git remote helper for seamless integration
4. Add caching layer for performance
5. Test with real Git repositories
