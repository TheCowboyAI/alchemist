# Git Domain

## Overview

The Git Domain provides deep integration with Git repositories, enabling version control analysis, commit tracking, branch management, and repository insights within CIM. It bridges the gap between code development and business processes by making Git data accessible for workflows, analytics, and visualization.

## Key Concepts

### Repository
- **Definition**: A Git repository tracked and analyzed by CIM
- **Properties**: URL, name, default branch, clone status
- **Operations**: Clone, fetch, analyze, monitor
- **Integration**: Links to other domains via commit metadata

### Commit
- **Definition**: A point-in-time snapshot of repository state
- **Properties**: SHA, author, timestamp, message, diff
- **Analysis**: Code changes, impact, patterns
- **Relationships**: Parent commits, branches, tags

### Branch
- **Definition**: A named reference to a commit chain
- **Properties**: Name, HEAD commit, upstream, protection rules
- **Operations**: Create, merge, delete, compare
- **Tracking**: Active development, merge status

### Repository Analysis
- **Definition**: Insights derived from repository data
- **Metrics**: Velocity, quality, collaboration patterns
- **Visualization**: Commit graphs, contributor networks
- **Reporting**: Development trends, bottlenecks

## Domain Events

### Commands
- `cmd.git.clone_repository` - Clone new repository
- `cmd.git.analyze_commits` - Process commit history
- `cmd.git.track_branch` - Monitor branch changes
- `cmd.git.generate_insights` - Create analytics
- `cmd.git.sync_repository` - Update from remote

### Events
- `event.git.repository_cloned` - Repository added
- `event.git.commits_analyzed` - Analysis complete
- `event.git.branch_updated` - Branch changed
- `event.git.merge_detected` - Merge occurred
- `event.git.conflict_found` - Merge conflict

### Queries
- `query.git.get_commits` - Retrieve commit history
- `query.git.find_authors` - List contributors
- `query.git.compare_branches` - Branch differences
- `query.git.search_code` - Code search

## API Reference

### GitAggregate
```rust
pub struct GitAggregate {
    pub id: RepositoryId,
    pub url: String,
    pub name: String,
    pub branches: HashMap<BranchName, Branch>,
    pub tags: HashMap<TagName, Tag>,
    pub metadata: RepositoryMetadata,
}
```

### Key Methods
- `clone_repository()` - Initial repository setup
- `fetch_updates()` - Sync with remote
- `analyze_history()` - Process commits
- `get_file_at_commit()` - Historical file access
- `calculate_metrics()` - Generate insights

## Repository Management

### Cloning and Setup
```rust
// Clone repository
let clone = CloneRepository {
    url: "https://github.com/org/repo.git".to_string(),
    name: "my-project".to_string(),
    credentials: Some(GitCredentials::Token(token)),
    shallow: false,
    branch: Some("main".to_string()),
};

// Configure analysis
let config = AnalysisConfig {
    analyze_commits: true,
    analyze_files: true,
    analyze_contributors: true,
    ignore_patterns: vec![
        "*.log".to_string(),
        "node_modules/".to_string(),
    ],
    metrics: vec![
        MetricType::CodeChurn,
        MetricType::CommitFrequency,
        MetricType::ContributorActivity,
    ],
};
```

### Commit Analysis
```rust
// Analyze commit range
let analyze = AnalyzeCommits {
    repository_id,
    start_commit: Some(commit_sha),
    end_commit: None, // HEAD
    options: AnalysisOptions {
        include_diffs: true,
        include_stats: true,
        file_patterns: Some(vec!["*.rs", "*.toml"]),
    },
};

// Commit data
let commit_info = CommitInfo {
    sha: "abc123...".to_string(),
    author: Author {
        name: "Jane Developer".to_string(),
        email: "jane@example.com".to_string(),
    },
    timestamp: commit_time,
    message: "feat: Add new graph visualization".to_string(),
    stats: CommitStats {
        files_changed: 5,
        insertions: 127,
        deletions: 23,
    },
    files: vec![
        FileChange {
            path: "src/graph/mod.rs".to_string(),
            change_type: ChangeType::Modified,
            additions: 45,
            deletions: 12,
        },
    ],
};
```

### Branch Operations
```rust
// Track branch
let track = TrackBranch {
    repository_id,
    branch_name: "feature/new-ui".to_string(),
    track_commits: true,
    notify_on_changes: true,
};

// Compare branches
let compare = CompareBranches {
    repository_id,
    base_branch: "main".to_string(),
    compare_branch: "feature/new-ui".to_string(),
    include_commits: true,
    include_files: true,
};

// Merge analysis
let merge_preview = PreviewMerge {
    repository_id,
    source_branch: "feature/new-ui".to_string(),
    target_branch: "main".to_string(),
};

let merge_result = MergePreviewResult {
    can_merge: true,
    conflicts: vec![],
    commits_ahead: 15,
    commits_behind: 3,
    files_changed: 42,
};
```

## Code Analysis

### File History
```rust
// Get file history
let history = GetFileHistory {
    repository_id,
    file_path: "src/main.rs".to_string(),
    max_commits: 50,
    include_content: true,
};

// File evolution
let evolution = FileEvolution {
    path: "src/main.rs".to_string(),
    commits: vec![
        FileVersion {
            commit_sha: "abc123".to_string(),
            timestamp: commit_time,
            author: "Jane Developer".to_string(),
            lines_added: 20,
            lines_removed: 5,
            content: Some(file_content),
        },
    ],
    total_commits: 127,
    total_authors: 8,
    creation_date: file_created,
};
```

### Code Search
```rust
// Search code
let search = SearchCode {
    repository_id,
    query: "GraphNode".to_string(),
    search_type: SearchType::Regex,
    file_patterns: vec!["*.rs"],
    branch: Some("main".to_string()),
    case_sensitive: false,
};

// Search results
let results = CodeSearchResults {
    total_matches: 23,
    files: vec![
        FileMatch {
            path: "src/graph/node.rs".to_string(),
            matches: vec![
                Match {
                    line_number: 42,
                    line_content: "pub struct GraphNode {".to_string(),
                    highlight_start: 11,
                    highlight_end: 20,
                },
            ],
        },
    ],
};
```

## Repository Insights

### Development Metrics
```rust
// Calculate metrics
let metrics = CalculateMetrics {
    repository_id,
    time_range: TimeRange::LastDays(30),
    metrics: vec![
        MetricType::CommitFrequency,
        MetricType::CodeChurn,
        MetricType::ContributorActivity,
        MetricType::BranchLifetime,
    ],
};

// Metric results
let results = MetricResults {
    commit_frequency: CommitFrequency {
        daily_average: 12.5,
        peak_day: Weekday::Wednesday,
        peak_hour: 14, // 2 PM
    },
    code_churn: CodeChurn {
        files_changed_frequently: vec![
            ("src/main.rs", 45),
            ("src/graph/mod.rs", 38),
        ],
        churn_rate: 0.23,
    },
    contributor_activity: ContributorActivity {
        active_contributors: 8,
        commit_distribution: HashMap::from([
            ("Jane Developer", 145),
            ("John Coder", 98),
        ]),
    },
};
```

### Visualization Data
```rust
// Generate commit graph
let graph = GenerateCommitGraph {
    repository_id,
    max_commits: 100,
    include_branches: true,
    include_tags: true,
};

// Graph data for visualization
let commit_graph = CommitGraphData {
    nodes: vec![
        CommitNode {
            sha: "abc123".to_string(),
            message: "Initial commit".to_string(),
            author: "Jane Developer".to_string(),
            timestamp: commit_time,
            x: 0.0,
            y: 0.0,
        },
    ],
    edges: vec![
        CommitEdge {
            from: "abc123".to_string(),
            to: "def456".to_string(),
            edge_type: EdgeType::Parent,
        },
    ],
    branches: HashMap::from([
        ("main", "ghi789"),
        ("feature/new-ui", "jkl012"),
    ]),
};
```

## Integration Patterns

### Workflow Integration
```rust
// Trigger workflow on commit
on_event("git.commit.pushed", |event: CommitPushed| {
    if event.branch == "main" {
        start_workflow("deployment", WorkflowContext {
            commit_sha: event.sha,
            author: event.author,
            message: event.message,
        });
    }
});

// Code review workflow
let review_workflow = CodeReviewWorkflow {
    trigger: WorkflowTrigger::PullRequest,
    steps: vec![
        Step::RunTests,
        Step::CodeAnalysis,
        Step::RequestReview,
        Step::ApprovalGate(min_approvals: 2),
        Step::Merge,
    ],
};
```

### Graph Domain Integration
```rust
// Convert commits to graph
impl From<Repository> for Graph {
    fn from(repo: Repository) -> Self {
        let mut graph = Graph::new("Repository Structure");
        
        // Add commits as nodes
        for commit in repo.commits() {
            graph.add_node(Node {
                id: NodeId::from(commit.sha),
                node_type: NodeType::Commit,
                content: commit.message,
                metadata: commit.into_metadata(),
            });
        }
        
        // Add parent relationships
        for commit in repo.commits() {
            for parent in commit.parents {
                graph.add_edge(Edge {
                    source: NodeId::from(commit.sha),
                    target: NodeId::from(parent),
                    edge_type: EdgeType::Parent,
                });
            }
        }
        
        graph
    }
}
```

## Monitoring and Automation

### Repository Monitoring
```rust
// Monitor repository
let monitor = MonitorRepository {
    repository_id,
    monitors: vec![
        Monitor::CommitRate {
            threshold: 5, // commits per day
            window: Duration::days(7),
            action: Action::Alert,
        },
        Monitor::FileSize {
            max_size_mb: 100,
            action: Action::Reject,
        },
        Monitor::BranchAge {
            max_age_days: 30,
            action: Action::Notify,
        },
    ],
};

// Automated actions
on_event("git.monitor.threshold_exceeded", |event| {
    match event.monitor_type {
        MonitorType::CommitRate => {
            notify_team("Low commit activity detected");
        }
        MonitorType::BranchAge => {
            suggest_branch_cleanup(event.branch);
        }
    }
});
```

## Use Cases

### Development Analytics
- Team productivity metrics
- Code quality trends
- Bottleneck identification
- Contributor insights

### CI/CD Integration
- Automated deployments
- Build triggers
- Test automation
- Release management

### Code Review
- Pull request analysis
- Automated checks
- Review assignments
- Merge conflict detection

### Knowledge Management
- Code documentation
- Change history
- Expert identification
- Technical debt tracking

## Performance Characteristics

- **Repository Size**: Up to 10GB repos
- **Commit Processing**: 1000 commits/second
- **Search Speed**: <100ms for code search
- **Analysis Speed**: Full repo analysis in minutes

## Best Practices

1. **Shallow Clones**: Use for large repositories
2. **Incremental Updates**: Fetch only new commits
3. **Selective Analysis**: Filter by file patterns
4. **Caching**: Cache analysis results
5. **Rate Limiting**: Respect Git service limits

## Related Domains

- **Graph Domain**: Visualize commit networks
- **Workflow Domain**: Trigger on Git events
- **Identity Domain**: Map commits to identities
- **Document Domain**: Link docs to commits 