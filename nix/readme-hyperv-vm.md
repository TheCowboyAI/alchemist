# Information Alchemist Hyper-V Development VM

This directory contains the NixOS configuration for building a complete development VM for Information Alchemist that runs on Hyper-V.

## Features

The VM includes:

- **Desktop Environment**: GNOME with Wayland support
- **Development Tools**:
  - Rust toolchain (via rustup, configured for nightly)
  - rust-analyzer, cargo-watch, cargo-nextest, bacon
  - Git, Neovim, VS Code
  - Build essentials (gcc, pkg-config, cmake, etc.)
- **NATS Infrastructure**:
  - NATS Server with JetStream enabled
  - natscli (NATS CLI tool)
  - nsc (NATS account management)
  - Pre-configured for development (ports 4222, 4223, 8222)
- **Graphics Support**:
  - Vulkan drivers and tools
  - Wayland protocols
  - All Bevy dependencies (ALSA, udev, X11 compatibility)
- **Convenience Tools**:
  - direnv with nix-direnv
  - Modern CLI tools (ripgrep, fd, bat, eza, zoxide, fzf)
  - tmux, htop
  - Pre-configured shell with helpful aliases

## Building the VM

### Prerequisites

- NixOS or any Linux system with Nix installed
- Flakes enabled in your Nix configuration
- At least 20GB of free disk space
- 8GB+ RAM recommended for the build process

### Build Instructions

1. Navigate to the nix directory:
   ```bash
   cd /git/thecowboyai/alchemist/nix
   ```

2. Run the build script:
   ```bash
   ./build-hyperv-vm.sh
   ```

   Or build manually:
   ```bash
   nix build -L .#hyperv -f hyperv-flake.nix
   ```

3. The build will take 20-30 minutes and produce a `.vhdx` file at `./result/hyperv.vhdx`

## Using the VM

### Importing into Hyper-V

1. Copy the `hyperv.vhdx` file to your Windows machine
2. Open Hyper-V Manager
3. Create a new Virtual Machine:
   - Name: "IA Development"
   - Generation: Generation 2 (UEFI)
   - Memory: 8192 MB (or more)
   - Networking: Default Switch
   - Hard Drive: Use existing virtual hard disk → Select the `.vhdx` file
4. Configure the VM:
   - Processors: 4 or more vCPUs
   - Enable nested virtualization if needed
   - Disable Secure Boot (Settings → Security → Uncheck "Enable Secure Boot")

### First Login

- **Username**: `developer`
- **Password**: `developer` (change this immediately!)

### Initial Setup

1. After logging in, open a terminal and run:
   ```bash
   setup-ia-dev
   ```
   This will:
   - Install Rust nightly toolchain
   - Clone the Information Alchemist repository
   - Set up direnv
   - Ensure NATS server is running

2. Install Cursor IDE:
   ```bash
   install-cursor
   ```
   This downloads and installs Cursor in your user directory.

3. Change your password:
   ```bash
   passwd
   ```

## Development Workflow

### Starting Development

1. Navigate to the project:
   ```bash
   cd ~/projects/alchemist
   ```

2. Enter the development shell:
   ```bash
   nix develop
   ```

3. Build the project:
   ```bash
   nix build
   ```

4. Run the application:
   ```bash
   ./result/bin/ia
   ```

### Using NATS

NATS server starts automatically on boot. Useful commands:

```bash
# Check NATS status
systemctl status nats

# View streams
nats stream ls

# Monitor NATS
# Open browser to http://localhost:8222

# View NATS logs
journalctl -u nats -f
```

### Using Cursor

After installation, you can:
- Launch from terminal: `cursor`
- Launch from GNOME applications menu
- Open project: `cursor ~/projects/alchemist`

## VM Configuration

### System Specifications

- **Base OS**: NixOS 24.05
- **Desktop**: GNOME with Wayland
- **Default User**: developer (sudoer, no password required for sudo)
- **Networking**: NetworkManager, firewall enabled with NATS ports open
- **Time Zone**: America/Los_Angeles (change in `/etc/nixos/configuration.nix`)

### Customization

To modify the VM configuration:

1. Edit `/etc/nixos/configuration.nix`
2. Rebuild: `sudo nixos-rebuild switch`

Common customizations:
- Change timezone: `time.timeZone = "Your/Timezone";`
- Add packages: Add to `environment.systemPackages`
- Change NATS configuration: Modify `services.nats.settings`

### Performance Tuning

For better performance:
- Allocate more RAM (12-16GB recommended)
- Increase vCPUs (6-8 for heavy development)
- Enable RemoteFX if available
- Use SSD storage for the VHDX file

## Troubleshooting

### Graphics Issues

If you experience graphics problems:
```bash
# Check Vulkan
vulkaninfo

# Test Wayland
echo $WAYLAND_DISPLAY

# Force X11 if needed
export WINIT_UNIX_BACKEND=x11
```

### NATS Connection Issues

```bash
# Restart NATS
sudo systemctl restart nats

# Check NATS config
sudo cat /etc/nats.conf

# Test connection
nats --server localhost:4222 pub test "hello"
```

### Bevy/Rust Issues

```bash
# Update Rust
rustup update

# Clear cargo cache
cargo clean

# Rebuild with verbose output
RUST_BACKTRACE=1 cargo build -vv
```

## Files in this Directory

- `hyperv-vm.nix` - Main NixOS configuration
- `hyperv-flake.nix` - Flake definition for nixos-generators
- `build-hyperv-vm.sh` - Build script
- `README-hyperv-vm.md` - This file

## Support

For issues specific to the VM setup, check:
1. Hyper-V event logs
2. VM console during boot
3. `journalctl -b` for boot logs
4. `/var/log/` for application logs

For Information Alchemist issues, see the main project documentation.
