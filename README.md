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

### NixOS (Recommended)

Kapsule is packaged as a native Nix Flake. The package automatically retrieves the latest release and patches it against your host's native graphical libraries (WebKitGTK, GTK3) for optimal performance and display server compatibility.

Run Kapsule instantly without installation:
```bash
nix run github:cshah25/kapsule
```

### Linux Distributions (Debian, Fedora, AppImage)

Pre-compiled binaries are generated automatically and available on the [Releases](https://github.com/cshah25/kapsule/releases) page.
- **Debian/Ubuntu**: Download and install the `.deb` package.
- **Fedora/RHEL**: Download and install the `.rpm` package.
- **Universal**: Download the `.AppImage`, mark it as executable (`chmod +x`), and run it directly.

## Development

Kapsule provides a fully reproducible build environment using Nix.

### Prerequisites

- Nix package manager with Flakes enabled.

### Setup

1. Enter the isolated development shell (this provisions Rust, Node.js, and all native GTK libraries):
   ```bash
   nix develop
   ```
2. Navigate to the project root and install Node dependencies:
   ```bash
   cd kapsule
   npm install
   ```
3. Start the development server with Hot-Module Replacement (HMR):
   ```bash
   npm run tauri dev
   ```

### Building for Production

To compile native release binaries:
```bash
npm run tauri build
```
Compiled bundles will be located in `src-tauri/target/release/bundle/`.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
