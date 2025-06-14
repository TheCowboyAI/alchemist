//! Flake template system for common project types

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Available flake templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlakeTemplate {
    /// Basic Rust project
    Rust,
    /// Python project with poetry
    Python,
    /// Node.js project
    NodeJs,
    /// Go project
    Go,
    /// C/C++ project
    Cpp,
    /// Haskell project
    Haskell,
    /// Multi-language project
    Polyglot,
    /// NixOS system configuration
    NixOSSystem,
    /// Home Manager configuration
    HomeManager,
    /// Development shell only
    DevShell,
    /// Custom template from string
    Custom(String),
}

impl FlakeTemplate {
    /// Generate flake.nix content for the template
    pub fn generate_flake_nix(&self) -> String {
        match self {
            FlakeTemplate::Rust => self.rust_template(),
            FlakeTemplate::Python => self.python_template(),
            FlakeTemplate::NodeJs => self.nodejs_template(),
            FlakeTemplate::Go => self.go_template(),
            FlakeTemplate::Cpp => self.cpp_template(),
            FlakeTemplate::Haskell => self.haskell_template(),
            FlakeTemplate::Polyglot => self.polyglot_template(),
            FlakeTemplate::NixOSSystem => self.nixos_system_template(),
            FlakeTemplate::HomeManager => self.home_manager_template(),
            FlakeTemplate::DevShell => self.devshell_template(),
            FlakeTemplate::Custom(content) => content.clone(),
        }
    }

    /// Generate additional files for the template
    pub fn additional_files(&self) -> HashMap<String, String> {
        let mut files = HashMap::new();

        match self {
            FlakeTemplate::Rust => {
                files.insert(".envrc".to_string(), "use flake\n".to_string());
                files.insert(".gitignore".to_string(), "/target\n/result\n".to_string());
                files.insert("Cargo.toml".to_string(), self.rust_cargo_toml());
                files.insert("src/main.rs".to_string(), "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string());
            }
            FlakeTemplate::Python => {
                files.insert(".envrc".to_string(), "use flake\n".to_string());
                files.insert(".gitignore".to_string(), "/.venv\n/__pycache__\n/result\n".to_string());
                files.insert("pyproject.toml".to_string(), self.python_pyproject_toml());
                files.insert("src/__init__.py".to_string(), "".to_string());
            }
            FlakeTemplate::NodeJs => {
                files.insert(".envrc".to_string(), "use flake\n".to_string());
                files.insert(".gitignore".to_string(), "/node_modules\n/result\n".to_string());
                files.insert("package.json".to_string(), self.nodejs_package_json());
            }
            _ => {}
        }

        files
    }

    fn rust_template(&self) -> String {
        r#"{
  description = "A Rust project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "my-rust-app";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            cargo-watch
            cargo-edit
            cargo-audit
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      });
}"#.to_string()
    }

    fn python_template(&self) -> String {
        r#"{
  description = "A Python project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    poetry2nix = {
      url = "github:nix-community/poetry2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, poetry2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        poetry2nixLib = poetry2nix.lib.mkPoetry2Nix { inherit pkgs; };
      in
      {
        packages.default = poetry2nixLib.mkPoetryApplication {
          projectDir = self;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          packages = with pkgs; [
            poetry
            python311
            python311Packages.pip
            python311Packages.virtualenv
          ];
        };
      });
}"#.to_string()
    }

    fn nodejs_template(&self) -> String {
        r#"{
  description = "A Node.js project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        nodejs = pkgs.nodejs_20;
      in
      {
        packages.default = pkgs.buildNpmPackage {
          pname = "my-node-app";
          version = "0.1.0";
          src = ./.;
          npmDepsHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
          nodejs = nodejs;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            nodejs
            nodePackages.npm
            nodePackages.yarn
            nodePackages.pnpm
            nodePackages.typescript
            nodePackages.typescript-language-server
          ];
        };
      });
}"#.to_string()
    }

    fn go_template(&self) -> String {
        r#"{
  description = "A Go project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.buildGoModule {
          pname = "my-go-app";
          version = "0.1.0";
          src = ./.;
          vendorHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            go
            gopls
            go-tools
            golangci-lint
            delve
          ];
        };
      });
}"#.to_string()
    }

    fn cpp_template(&self) -> String {
        r#"{
  description = "A C++ project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "my-cpp-app";
          version = "0.1.0";
          src = ./.;
          
          nativeBuildInputs = with pkgs; [
            cmake
            ninja
          ];
          
          buildInputs = with pkgs; [
            boost
            fmt
          ];
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          buildInputs = with pkgs; [
            clang-tools
            gdb
            valgrind
            ccache
          ];
        };
      });
}"#.to_string()
    }

    fn haskell_template(&self) -> String {
        r#"{
  description = "A Haskell project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        haskellPackages = pkgs.haskellPackages;
      in
      {
        packages.default = haskellPackages.callCabal2nix "my-haskell-app" ./. { };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            haskellPackages.ghc
            haskellPackages.cabal-install
            haskellPackages.haskell-language-server
            haskellPackages.hlint
            haskellPackages.ormolu
          ];
        };
      });
}"#.to_string()
    }

    fn polyglot_template(&self) -> String {
        r#"{
  description = "A multi-language project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              # Common tools
              git
              gnumake
              direnv
              
              # Language toolchains
              rustc
              cargo
              python3
              nodejs
              go
              
              # Editors and LSPs
              neovim
              emacs
              vscode
            ];
          };
          
          rust = pkgs.mkShell {
            buildInputs = with pkgs; [ rustc cargo rust-analyzer ];
          };
          
          python = pkgs.mkShell {
            buildInputs = with pkgs; [ python3 poetry ];
          };
          
          node = pkgs.mkShell {
            buildInputs = with pkgs; [ nodejs nodePackages.npm ];
          };
        };
      });
}"#.to_string()
    }

    fn nixos_system_template(&self) -> String {
        r#"{
  description = "NixOS system configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, home-manager }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.my-system = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          ./configuration.nix
          home-manager.nixosModules.home-manager
          {
            home-manager.useGlobalPkgs = true;
            home-manager.useUserPackages = true;
            home-manager.users.myuser = import ./home.nix;
          }
        ];
      };
    };
}"#.to_string()
    }

    fn home_manager_template(&self) -> String {
        r#"{
  description = "Home Manager configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, home-manager }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      homeConfigurations.myuser = home-manager.lib.homeManagerConfiguration {
        inherit pkgs;
        modules = [ ./home.nix ];
      };
    };
}"#.to_string()
    }

    fn devshell_template(&self) -> String {
        r#"{
  description = "Development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Add your development tools here
            git
            vim
            tmux
          ];
          
          shellHook = ''
            echo "Welcome to the development shell!"
          '';
        };
      });
}"#.to_string()
    }

    fn rust_cargo_toml(&self) -> String {
        r#"[package]
name = "my-rust-app"
version = "0.1.0"
edition = "2021"

[dependencies]
"#.to_string()
    }

    fn python_pyproject_toml(&self) -> String {
        r#"[tool.poetry]
name = "my-python-app"
version = "0.1.0"
description = ""
authors = ["Your Name <you@example.com>"]

[tool.poetry.dependencies]
python = "^3.11"

[tool.poetry.dev-dependencies]
pytest = "^7.0"
black = "^23.0"
mypy = "^1.0"

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"
"#.to_string()
    }

    fn nodejs_package_json(&self) -> String {
        r#"{
  "name": "my-node-app",
  "version": "0.1.0",
  "description": "",
  "main": "index.js",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1"
  },
  "keywords": [],
  "author": "",
  "license": "ISC"
}
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_template_generation() {
        let template = FlakeTemplate::Rust;
        let flake_content = template.generate_flake_nix();
        assert!(flake_content.contains("rustPlatform"));
        assert!(flake_content.contains("rust-overlay"));
    }

    #[test]
    fn test_additional_files() {
        let template = FlakeTemplate::Rust;
        let files = template.additional_files();
        assert!(files.contains_key(".envrc"));
        assert!(files.contains_key("Cargo.toml"));
        assert!(files.contains_key("src/main.rs"));
    }
} 