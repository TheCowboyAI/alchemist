{
  description = "Information Alchemist Development VM for Hyper-V";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, ... }: {
    packages.x86_64-linux = {
      hyperv = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "hyperv";

        modules = [
          ./hyperv-vm.nix

          # Additional module for Cursor installation script
          ({ pkgs, ... }: {
            # Create a script to download and install Cursor
            environment.systemPackages = with pkgs; [
              (writeScriptBin "install-cursor" ''
                #!${pkgs.bash}/bin/bash
                set -e

                echo "Installing Cursor IDE..."

                # Create temporary directory
                TEMP_DIR=$(mktemp -d)
                cd $TEMP_DIR

                # Download Cursor AppImage
                echo "Downloading Cursor..."
                wget -q --show-progress https://downloader.cursor.sh/linux/appImage/x64 -O cursor.AppImage

                # Make it executable
                chmod +x cursor.AppImage

                # Extract AppImage
                echo "Extracting Cursor..."
                ./cursor.AppImage --appimage-extract

                # Move to user's local applications
                mkdir -p $HOME/.local/share/applications
                mkdir -p $HOME/.local/bin

                # Move extracted files
                mv squashfs-root $HOME/.local/share/cursor

                # Create launcher script
                cat > $HOME/.local/bin/cursor << 'EOF'
                #!/bin/bash
                exec $HOME/.local/share/cursor/cursor "$@"
                EOF

                chmod +x $HOME/.local/bin/cursor

                # Create desktop entry
                cat > $HOME/.local/share/applications/cursor.desktop << 'EOF'
                [Desktop Entry]
                Name=Cursor
                Comment=AI-powered code editor
                Exec=$HOME/.local/bin/cursor %F
                Icon=$HOME/.local/share/cursor/cursor.png
                Type=Application
                Categories=Development;IDE;
                Terminal=false
                StartupNotify=true
                EOF

                # Clean up
                cd /
                rm -rf $TEMP_DIR

                echo "Cursor installed successfully!"
                echo "You can now run 'cursor' from the terminal or find it in your applications menu."
                echo ""
                echo "Make sure $HOME/.local/bin is in your PATH:"
                echo "export PATH=\$HOME/.local/bin:\$PATH"
              '')

              # Script to set up the development environment
              (writeScriptBin "setup-ia-dev" ''
                #!${pkgs.bash}/bin/bash
                set -e

                echo "Setting up Information Alchemist development environment..."

                # Install Rust nightly
                echo "Installing Rust nightly toolchain..."
                rustup default nightly
                rustup component add rust-src rust-analyzer

                # Clone the repository
                echo "Where would you like to clone the Information Alchemist repository?"
                read -p "Directory (default: ~/projects): " PROJECT_DIR
                PROJECT_DIR=''${PROJECT_DIR:-~/projects}

                mkdir -p "$PROJECT_DIR"
                cd "$PROJECT_DIR"

                if [ ! -d "alchemist" ]; then
                  echo "Cloning Information Alchemist repository..."
                  git clone --recursive https://github.com/thecowboyai/alchemist.git
                  cd alchemist
                else
                  echo "Repository already exists at $PROJECT_DIR/alchemist"
                  cd alchemist
                fi

                # Initialize direnv
                echo "Setting up direnv..."
                direnv allow

                # Set up NATS
                echo "Checking NATS server..."
                if systemctl is-active --quiet nats; then
                  echo "NATS server is running"
                else
                  echo "Starting NATS server..."
                  sudo systemctl start nats
                  sudo systemctl enable nats
                fi

                echo ""
                echo "Setup complete! You can now:"
                echo "1. cd $PROJECT_DIR/alchemist"
                echo "2. nix develop"
                echo "3. nix build"
                echo ""
                echo "To install Cursor IDE, run: install-cursor"
              '')
            ];

            # Add PATH configuration for user
            environment.shellInit = ''
              # Add user's local bin to PATH
              export PATH="$HOME/.local/bin:$PATH"
            '';

            # Create initial setup message
            services.getty.helpLine = ''

              Welcome to Information Alchemist Development VM!

              Default user: developer
              Default password: developer (please change it!)

              To get started:
              1. Log in as 'developer'
              2. Run 'setup-ia-dev' to set up the development environment
              3. Run 'install-cursor' to install Cursor IDE

              NATS server is running on localhost:4222
              NATS monitoring available at http://localhost:8222

            '';
          })
        ];
      };
    };
  };
}
