# Kapsule Container Engine

Kapsule is a lightweight, high-performance container management engine and graphical interface. Designed with modern web technologies and a secure systems backend, Kapsule provides an intuitive desktop experience for managing containerized applications, images, and system resources.

## Features

- **Intuitive Dashboard**: Monitor running containers, system resource utilization, and recent daemon events in real-time.
- **Library Management**: Pull, inspect, and manage container images with byte-accurate progress tracking and stream buffering.
- **Vessel Provisioning**: Create and configure new containers (Vessels) with advanced port routing, volume mapping, and environment variable injection.
- **Interactive Logs**: Stream live execution logs directly from container standard output with integrated auto-scrolling.
- **Native OS Integration**: Built as a native desktop application with comprehensive system distribution support.

## Architecture

Kapsule leverages a hybrid architecture to deliver web-like UI fluidity with native system performance:

- **Frontend**: SvelteKit, compiled to static assets for rapid rendering.
- **Backend/Engine**: Rust and Tauri, handling secure communication with the host operating system.
- **Daemon Communication**: The Rust `bollard` crate interfaces directly with Docker and Podman Unix sockets for zero-overhead container orchestration.

## Installation

### Linux Distributions (Debian, Fedora, AppImage)

Pre-compiled binaries are generated automatically and available on the [Releases](https://github.com/cshah25/kapsule/releases) page.
- **Debian/Ubuntu**: Download and install the `.deb` package.
- **Fedora/RHEL**: Download and install the `.rpm` package.
- **Universal**: Download the `.AppImage`, mark it as executable (`chmod +x`), and run it directly.

### NixOS
Kapsule is packaged as a Nix Flake (x86_64-linux). The package retrieves the latest release and patches it against Nixpkgs' WebKitGTK and GTK3.

Run Kapsule instantly without installation, built against your system's own nixpkgs:
```bash
nix run github:cshah25/kapsule --override-input nixpkgs nixpkgs
```

Or install it from a flake-based NixOS configuration:
```nix
# flake.nix
inputs.kapsule = {
  url = "github:cshah25/kapsule";
  inputs.nixpkgs.follows = "nixpkgs";
};

# configuration.nix (with `inputs` passed via specialArgs)
environment.systemPackages = [ inputs.kapsule.packages.${pkgs.system}.default ];
```

> **Why use your own nixpkgs?** Graphics drivers always come from your system, while the app's graphics libraries come from whichever nixpkgs it was built with. If that nixpkgs is much older than your system's, Kapsule may fail to start with `EGL_BAD_PARAMETER` or show a blank window. `--override-input nixpkgs nixpkgs` and `inputs.nixpkgs.follows` build it against the same nixpkgs as your drivers. Plain `nix run github:cshah25/kapsule` also works, using the pin refreshed at each release.

## Development

Kapsule provides a fully reproducible build environment using Nix.

### Prerequisites

- Docker or Podman

### Setup

1. Clone this repo
```bash
git clone https://github.com/cshah25/kapsule.git
cd kapsule
```

2. Build the container 
```bash
# Building the container
make build

# Running the container
make dev
```
3. Run the application in the container
```bash
cd kapsule && npm run tauri dev 
```

### Building for Production

To compile native release binaries:
```bash
npm run tauri build
```
Compiled bundles will be located in `src-tauri/target/release/bundle/`.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
