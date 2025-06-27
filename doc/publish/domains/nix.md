# Nix Domain

## Overview

The Nix Domain provides comprehensive NixOS package and configuration management within CIM. It enables declarative system configuration, reproducible builds, development environment management, and flake-based project definitions through visual workflows and intelligent automation.

## Key Concepts

### Flake
- **Definition**: A self-contained Nix project with inputs and outputs
- **Components**: Inputs, outputs, metadata, lock file
- **Properties**: URL, description, version, dependencies
- **Benefits**: Reproducibility, composability, versioning

### Derivation
- **Definition**: A build recipe that produces a package
- **Properties**: Name, version, source, build inputs, outputs
- **Operations**: Build, cache, substitute, garbage collect
- **Purity**: Deterministic builds from inputs

### Configuration
- **Definition**: Declarative system or service configuration
- **Types**: System, home-manager, service, development shell
- **Format**: Nix expression language
- **Management**: Version control, rollback, activation

### Development Shell
- **Definition**: Reproducible development environment
- **Contents**: Tools, dependencies, environment variables
- **Activation**: `nix develop`, direnv integration
- **Sharing**: Flake-based, version controlled

## Domain Events

### Commands
- `cmd.nix.create_flake` - Initialize new flake
- `cmd.nix.update_inputs` - Update flake dependencies
- `cmd.nix.build_derivation` - Build package
- `cmd.nix.apply_configuration` - Apply system config
- `cmd.nix.enter_shell` - Activate dev environment

### Events
- `event.nix.flake_created` - New flake initialized
- `event.nix.inputs_updated` - Dependencies updated
- `event.nix.build_completed` - Package built
- `event.nix.configuration_applied` - System updated
- `event.nix.shell_activated` - Dev env ready

### Queries
- `query.nix.search_packages` - Find packages
- `query.nix.get_dependencies` - List dependencies
- `query.nix.check_updates` - Available updates
- `query.nix.get_configuration` - Current config

## API Reference

### NixAggregate
```rust
pub struct NixAggregate {
    pub id: FlakeId,
    pub url: String,
    pub description: String,
    pub inputs: HashMap<String, FlakeInput>,
    pub outputs: FlakeOutputs,
    pub metadata: FlakeMetadata,
}
```

### Key Methods
- `create_flake()` - Initialize flake
- `add_input()` - Add dependency
- `define_output()` - Add package/config
- `update_lock()` - Update lock file
- `evaluate()` - Evaluate expressions

## Flake Management

### Creating Flakes
```rust
// Create new flake
let flake = CreateFlake {
    name: "my-project".to_string(),
    description: "CIM-based application".to_string(),
    template: FlakeTemplate::RustProject,
    initial_inputs: vec![
        FlakeInput {
            name: "nixpkgs".to_string(),
            url: "github:NixOS/nixpkgs/nixos-unstable".to_string(),
        },
        FlakeInput {
            name: "flake-utils".to_string(),
            url: "github:numtide/flake-utils".to_string(),
        },
    ],
};

// Define outputs
let outputs = DefineOutputs {
    flake_id,
    outputs: vec![
        Output::Package {
            name: "my-app".to_string(),
            derivation: Derivation {
                pname: "my-app".to_string(),
                version: "0.1.0".to_string(),
                src: "./.",
                build_inputs: vec!["rustc", "cargo"],
            },
        },
        Output::DevShell {
            packages: vec!["rust-analyzer", "rustfmt", "clippy"],
            environment: HashMap::from([
                ("RUST_LOG", "debug"),
                ("DATABASE_URL", "postgresql://localhost/myapp"),
            ]),
        },
    ],
};
```

### Package Management
```rust
// Search packages
let search = SearchPackages {
    query: "rust".to_string(),
    channel: NixChannel::Unstable,
    filters: SearchFilters {
        platforms: vec![Platform::X86_64Linux],
        licenses: vec![License::Mit, License::Apache2],
        maintainer: None,
    },
};

// Build derivation
let build = BuildDerivation {
    flake_ref: "github:owner/repo#package".to_string(),
    system: "x86_64-linux".to_string(),
    options: BuildOptions {
        use_cache: true,
        max_jobs: 4,
        timeout: Duration::minutes(30),
    },
};

// Install package
let install = InstallPackage {
    package: "firefox".to_string(),
    profile: InstallProfile::User,
    priority: None,
};
```

### Configuration Management
```rust
// System configuration
let system_config = SystemConfiguration {
    hostname: "my-nixos".to_string(),
    timezone: "America/New_York".to_string(),
    locale: "en_US.UTF-8".to_string(),
    services: vec![
        Service::SSH { 
            enable: true,
            port: 22,
            password_authentication: false,
        },
        Service::Docker {
            enable: true,
            users: vec!["myuser"],
        },
    ],
    packages: vec![
        "git", "vim", "htop", "tmux",
    ],
    users: vec![
        User {
            name: "myuser".to_string(),
            groups: vec!["wheel", "docker"],
            shell: "/run/current-system/sw/bin/zsh".to_string(),
        },
    ],
};

// Apply configuration
let apply = ApplyConfiguration {
    configuration: ConfigurationType::System(system_config),
    build_first: true,
    switch_method: SwitchMethod::Boot, // or Test, Switch
};
```

## Development Environments

### Shell Configuration
```rust
// Define development shell
let dev_shell = DevShell {
    name: "rust-dev".to_string(),
    packages: vec![
        "rustc",
        "cargo",
        "rust-analyzer",
        "pkg-config",
        "openssl",
    ],
    build_inputs: vec![
        "postgresql",
        "redis",
    ],
    shell_hook: r#"
        echo "Welcome to Rust development environment"
        export DATABASE_URL="postgresql://localhost/dev"
        alias test='cargo test'
        alias run='cargo run'
    "#.to_string(),
};

// Language-specific shells
let python_shell = PythonShell {
    python_version: "3.11".to_string(),
    packages: vec!["numpy", "pandas", "jupyter"],
    venv_path: ".venv".to_string(),
    pip_requirements: Some("requirements.txt".to_string()),
};

let node_shell = NodeShell {
    node_version: "20".to_string(),
    package_manager: PackageManager::Pnpm,
    global_packages: vec!["typescript", "prettier"],
};
```

### Direnv Integration
```rust
// Generate .envrc
let envrc = GenerateEnvrc {
    flake_id,
    use_flake: true,
    watch_files: vec![
        "flake.nix",
        "flake.lock",
        "shell.nix",
    ],
    layout: Some(Layout::Node), // or Python, Ruby
    custom_commands: vec![
        "export PROJECT_ROOT=$PWD",
        "source .env.local",
    ],
};

// Auto-activation
let auto_shell = ConfigureAutoShell {
    project_path: "/home/user/projects/myapp".to_string(),
    activation_method: ActivationMethod::Direnv,
    reload_on_change: true,
};
```

## Home Manager Integration

### User Environment
```rust
// Home configuration
let home_config = HomeConfiguration {
    username: "myuser".to_string(),
    home_directory: "/home/myuser".to_string(),
    state_version: "23.11".to_string(),
    
    programs: vec![
        Program::Git {
            user_name: "Jane Developer".to_string(),
            user_email: "jane@example.com".to_string(),
            signing_key: Some("ABC123".to_string()),
            aliases: HashMap::from([
                ("co", "checkout"),
                ("st", "status"),
            ]),
        },
        Program::Zsh {
            enable_autosuggestions: true,
            enable_syntax_highlighting: true,
            oh_my_zsh: Some(OhMyZsh {
                theme: "robbyrussell".to_string(),
                plugins: vec!["git", "docker", "kubectl"],
            }),
        },
    ],
    
    services: vec![
        HomeService::Syncthing {
            enable: true,
            data_dir: "~/Sync".to_string(),
        },
    ],
};
```

## Nix Expression Generation

### Template System
```rust
// Generate Nix expressions
let expr_generator = NixExpressionGenerator {
    style: ExpressionStyle::Flake,
    formatting: FormatOptions {
        indent_size: 2,
        max_line_length: 100,
        attribute_set_style: AttributeSetStyle::Multiline,
    },
};

// Package expression
let package_expr = expr_generator.generate_package(
    PackageDefinition {
        pname: "my-tool".to_string(),
        version: "1.0.0".to_string(),
        src: Source::Git {
            url: "https://github.com/user/repo".to_string(),
            rev: "main".to_string(),
        },
        build_phase: Some("make".to_string()),
        install_phase: Some("make install PREFIX=$out".to_string()),
    },
);

// Module expression
let module_expr = expr_generator.generate_module(
    ModuleDefinition {
        options: vec![
            Option {
                name: "services.myapp.enable".to_string(),
                type: OptionType::Bool,
                default: "false".to_string(),
                description: "Enable MyApp service".to_string(),
            },
        ],
        config: "config code here...".to_string(),
    },
);
```

## Integration Features

### CI/CD Integration
```rust
// GitHub Actions with Nix
let github_action = GenerateGitHubAction {
    name: "Nix Build".to_string(),
    triggers: vec![Trigger::Push, Trigger::PullRequest],
    jobs: vec![
        Job {
            name: "build".to_string(),
            runs_on: "ubuntu-latest".to_string(),
            steps: vec![
                Step::InstallNix {
                    extra_config: vec![
                        "experimental-features = nix-command flakes",
                    ],
                },
                Step::BuildFlake {
                    targets: vec!["#packages.x86_64-linux.default"],
                },
                Step::RunTests {
                    command: "nix flake check".to_string(),
                },
            ],
        },
    ],
};
```

### Container Integration
```rust
// Build Docker image with Nix
let docker_image = BuildDockerImage {
    name: "my-app".to_string(),
    tag: "latest".to_string(),
    contents: vec!["./result"],
    config: DockerConfig {
        cmd: vec!["/bin/my-app"],
        expose_ports: vec![8080],
        env: HashMap::from([
            ("APP_ENV", "production"),
        ]),
    },
};

// OCI image
let oci_image = BuildOCIImage {
    name: "my-service".to_string(),
    packages: vec!["my-app", "cacert"],
    entry_point: vec!["/bin/my-app", "serve"],
};
```

## Monitoring and Updates

### Update Management
```rust
// Check for updates
let updates = CheckUpdates {
    flake_id,
    inputs: vec!["nixpkgs", "home-manager"],
};

// Update results
let available_updates = UpdateResults {
    updates: vec![
        Update {
            input: "nixpkgs".to_string(),
            current: "abc123...".to_string(),
            latest: "def456...".to_string(),
            commits_behind: 127,
        },
    ],
};

// Auto-update policy
let auto_update = ConfigureAutoUpdate {
    flake_id,
    schedule: UpdateSchedule::Weekly,
    inputs: vec!["nixpkgs"],
    create_pr: true,
    run_tests: true,
};
```

## Use Cases

### Development Environments
- Reproducible dev shells
- Tool version management
- Dependency isolation
- Team environment sharing

### System Configuration
- Declarative OS config
- Service management
- User environment setup
- Configuration as code

### Package Management
- Custom package builds
- Dependency resolution
- Binary caching
- Overlay management

### CI/CD Pipelines
- Reproducible builds
- Container generation
- Test environments
- Deployment automation

## Performance Characteristics

- **Evaluation Speed**: <1s for typical configs
- **Build Caching**: Automatic deduplication
- **Download Speed**: Limited by network
- **Storage**: Efficient with hard links

## Best Practices

1. **Pin Inputs**: Always use flake.lock
2. **Modular Config**: Split large configurations
3. **Cache Usage**: Configure binary caches
4. **Pure Evaluation**: Avoid impurities
5. **Documentation**: Document custom options

## Related Domains

- **Git Domain**: Flake version control
- **Workflow Domain**: Build automation
- **Policy Domain**: System policies
- **Agent Domain**: Automated updates
