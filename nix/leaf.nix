# NixOS configuration for Information Alchemist development VM
{ config, pkgs, lib, ... }:

{
  # Basic system configuration
  system.stateVersion = "24.05";

  # Hyper-V guest services
  virtualisation.hypervGuest = {
    enable = true;
  };

  # Hardware configuration
  hardware = {
    graphics = {
      enable = true;
    };
  };

  # Networking
  networking = {
    hostName = "ia-dev-vm";
    networkmanager.enable = true;
    firewall = {
      enable = true;
      allowedTCPPorts = [
        4222 # NATS
        4223 # NATS WebSocket
        8222 # NATS monitoring
      ];
    };
  };

  # Time zone and locale
  time.timeZone = "America/Phoenix";
  i18n.defaultLocale = "en_US.UTF-8";

  # Enable GNOME with Wayland
  services = {
    xserver.enable = true;
    displayManager.gdm = {
      enable = true;
      wayland = true;
    };
    desktopManager.gnome.enable = true;
  };

  # Enable sound
  services.pulseaudio.enable = false;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
  };

  # System packages
  environment.systemPackages = with pkgs; [
    # Development tools
    git
    vim
    neovim
    wget
    curl
    htop
    tmux
    direnv
    nix-direnv

    # Build tools
    gcc
    pkg-config
    openssl
    cmake
    gnumake

    # Rust development (using rust-overlay would be better, but for system-wide we'll use stable)
    rustup
    rust-analyzer
    cargo-watch
    cargo-nextest
    cargo-edit
    cargo-expand
    bacon

    # NATS tools
    nats-server
    natscli
    nsc

    # Graphics/Wayland dependencies
    vulkan-loader
    vulkan-tools
    vulkan-validation-layers
    libxkbcommon
    wayland
    wayland-protocols
    libGL

    # Bevy dependencies
    alsa-lib
    udev
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr

    # Terminal and file management
    gnome-terminal
    firefox

    # System utilities
    unzip
    file
    which
    tree
    ripgrep
    fd
    bat
    eza
    zoxide
    fzf
  ];

  # Enable direnv
  programs.direnv = {
    enable = true;
    nix-direnv.enable = true;
  };

  # User configuration
  users.users.developer = {
    isNormalUser = true;
    description = "IA Developer";
    extraGroups = [ "wheel" "networkmanager" "video" "audio" ];
    shell = pkgs.bash;
    initialPassword = "developer"; # Change on first login!
  };

  # Enable sudo
  security.sudo.wheelNeedsPassword = false; # For development convenience

  # Services
  services = {
    # Enable SSH for remote development
    openssh = {
      enable = true;
      settings = {
        PasswordAuthentication = true;
        PermitRootLogin = "no";
      };
    };

    # NATS Server with JetStream
    nats = {
      enable = true;
      jetstream = true;
      settings = {
        server_name = lib.mkDefault "ia-dev-nats";
        jetstream = {
          store_dir = lib.mkDefault "/var/lib/nats/jetstream";
          max_memory_store = "1GB";
          max_file_store = "10GB";
        };
        # Enable monitoring
        http_port = 8222;
      };
    };
  };

  # System-wide environment variables
  environment.variables = {
    RUST_BACKTRACE = "1";
    WINIT_UNIX_BACKEND = "wayland";
  };

  # Shell configuration
  programs.bash.shellInit = ''
    # Rust setup
    if [ -f "$HOME/.cargo/env" ]; then
      source "$HOME/.cargo/env"
    fi

    # NATS aliases
    alias nats-streams='nats stream ls'
    alias nats-subs='nats consumer ls'

  '';

  # Nix configuration
  nix = {
    settings = {
      experimental-features = [ "nix-command" "flakes" ];
      trusted-users = [ "developer" ];
    };
    gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 7d";
    };
  };

  # Fonts
  fonts.packages = with pkgs; [
    noto-fonts
    noto-fonts-cjk-sans
    noto-fonts-emoji
    liberation_ttf
    fira-code
    fira-code-symbols
    jetbrains-mono
    nerd-fonts.fira-code
  ];

  # Enable automatic updates
  system.autoUpgrade = {
    enable = false; # Set to true if you want automatic updates
    allowReboot = false;
  };
}
