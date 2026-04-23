# Project Kapsule: Design Document

**Version:** 1.0 (April 2026)

**Status:** Ready for Implementation

**Lead Developer:** Rayu

---

## 1. Executive Summary

**Kapsule** is a security-first, desktop-native container management tool designed for Linux. It allows developers to run untrusted or version-specific environments (like Node.js 24) without exposing their host `$HOME` directory. Unlike Distrobox, which prioritizes integration, Kapsule prioritizes **isolation and reproducibility** while providing a "point-and-click" GUI for complex Podman/Docker commands.

---

## 2. Technical Stack

Kapsule utilizes a modern, systems-level stack designed for high performance, memory safety, and native Linux integration.

### 2.1 Backend (Systems & Logic)
- **Language:** **Rust** - Chosen for memory safety, zero-cost abstractions, and seamless interaction with Linux namespaces and cgroups.
- **Runtime Manager:** **Bollard** - An asynchronous Rust library for interacting with the Docker and Podman API sockets.
- **Monitoring:** **sysinfo** - Provides real-time CPU, RAM, and process tracking for running containers.
- **Async Runtime:** **Tokio** - Powers the concurrent handling of container logs, status polling, and registry searches.

### 2.2 Frontend (User Interface)
- **Framework:** **Tauri v2** - Leverages the system's native WebKitGTK engine instead of bundling Chromium, significantly reducing memory footprint.
- **UI Architecture:** **Svelte 5** - A high-performance reactive framework that compiles away, ensuring the UI remains snappy even on low-spec hardware.
- **Styling:** **Tailwind CSS** - Configured with a custom theme to match the **GTK/Libadwaita** design language for a consistent desktop feel.
- **Terminal:** **xterm.js** - Provides a high-performance terminal emulator for integrated container shell access and log viewing.

### 2.3 Desktop & Security Integration
- **Desktop Entries:** **freedesktop-desktop-entry** - For programmatically creating and managing `.desktop` files.
- **Credential Storage:** **keyring-rs** - Securely handles registry credentials via the system Secret Service (GNOME Keyring/KWallet).
- **Communication:** **JSON-RPC** - Structured communication between the Rust backend and the Svelte frontend.

---

## 3. Core Feature Architecture

### 3.1 Dual-Engine Abstraction Layer

The backend identifies available sockets and allows the user to toggle globally between engines.

- **Podman Mode:** Rootless by default. Optimized for security using the `:Z` flag for SELinux/permission relabeling.
- **Docker Mode:** Standard root-based execution for legacy home lab compatibility.
- **Auto-Detection:** On startup, Kapsule checks `systemctl --user status podman.socket` and `systemctl status docker.service`.

### 3.2 Registry Explorer (The "App Store")

A visual interface to browse and pull images.

- **Multi-Registry Support:** Search Docker Hub, Quay.io, and GHCR.
- **Tag Management:** Dynamically fetch all available tags (e.g., `alpine`, `bookworm`, `slim`) before pulling.
- **Vulnerability Overlay:** Optional integration with `Trivy` to show a "Security Grade" for images.

### 3.3 Vessel (Container) Configuration

The "Creation Wizard" allows granular control over the container's environment:

- **Isolated Home:** Creates a dedicated directory at `~/.local/share/kapsule/vessels/[name]/home` to act as the container's `$HOME`.
- **Folder Mapping:** Select host directories to mount as `Read/Write` or `Read-Only`.
- **Port Routing:** Map container ports to host ports (e.g., `5173 -> 5173` for Vite).
- **Volatile Mode:** An optional toggle to enable `--rm`, ensuring the container is deleted upon exit.

### 3.4 Linux Desktop Integration

- **Desktop Entry Generator:** One-click creation of `.desktop` files in `~/.local/share/applications`.
- **Icon Management:** Users can assign custom icons or use the default image logo.
- **Shell Integration:** A "Terminal" button that launches the user's host terminal (Zsh/Kitty) into the container environment.

---

## 4. Security Model: The "Fortress" Approach

Kapsule is designed to defeat supply-chain attacks (like the **March 2026 Axios RAT**).

| Feature                   | Protection Mechanism                                                                                                             |
|---------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| **Home Cloaking**         | The container sees a fake `$HOME`.                                                                                               | \
|                           |                                                                                                                                  | \
|                           | Real `.ssh`, `.aws`, and browser cookies are inaccessible.                                                                       |
| **Environment Scrubbing** | Host environment variables (API keys) are not passed to the container unless specified.                                          |
| **Network Toggling**      | Users can disable the network entirely for local-only data processing.                                                           |
| **User Mapping**          | Uses `--userns keep-id` to ensure files created in the container are owned by the host user, preventing root-owned file clutter. |

---

## 5. UI/UX Design Goals: Modern GNOME (Libadwaita)

Kapsule follows the **GNOME Human Interface Guidelines (HIG)** to provide a first-class experience on Linux desktops.

### 5.1 Design Language
- **Visual Style:** **Libadwaita** (Dark Mode by default). Uses soft-rounded corners (12px), flat surfaces with subtle depth, and high-contrast typography.
- **Palette:**
    - **Window:** `#242424`
    - **Cards/Surfaces:** `#2d2d2d`
    - **Accent (Actions):** `#3584e4` (Blue)
    - **Destructive:** `#c01c28` (Red)
- **Typography:** Uses the system default font (Inter or Cantarell) with semi-bold weights for hierarchy.

### 5.2 Key Layout Patterns
- **The HeaderBar:** 48px height with centered window titles and integrated action buttons (Add Vessel, Global Search).
- **The Dashboard:** A responsive grid of "Vessels." Each card displays real-time resource meters (CPU/RAM) using smooth SVG transitions.
- **Creation Wizard:** A "Centered Dialog" pattern following Libadwaita's modal style, using a multi-step navigation view.
- **Integrated Terminal:** A semi-transparent xterm.js overlay or dedicated view that matches the Libadwaita terminal aesthetic.

---

## 6. Implementation Roadmap (6-Week Cycle)

| Phase      | Milestone              | Focus                                                                                   |
|------------|------------------------|-----------------------------------------------------------------------------------------|
| **Week 1** | **The Foundation**     | Tauri + Svelte scaffold. Rust Engine Switcher: auto-detect Podman/Docker sockets.       |
| **Week 2** | **The Vessel**         | Container creation UI — isolated $HOME, mounts, port routing, Volatile Mode toggle.    |
| **Week 3** | **The Dashboard**      | Real-time CPU/RAM monitoring, container lifecycle (start/stop/delete), log viewer.     |
| **Week 4** | **The Library**        | Registry Explorer — multi-registry search, tag browser, image pull with progress UI.  |
| **Week 5** | **The Fortress**       | Security hardening — env scrubbing, network toggle, userns mapping, Trivy integration. |
| **Week 6** | **The Polish**         | .desktop file generator, icon management, error handling, Arch PKGBUILD.              |

---

## 7. Development Environment: "Kapsule-in-Kapsule"

To maintain the project's philosophy of isolation, the entire development environment is containerized.

### 7.1 The Dev Container
- **Base Image:** Arch Linux (Latest)
- **Tooling:** Rust (Stable), Node.js (v24+), Tauri v2 build dependencies.
- **Passthrough:** X11 socket (for GUI), Podman/Docker socket (for backend testing).

### 7.2 Bootstrapping
1. **Build the Environment:**
   ```bash
   cd develop && docker-compose build
   ```
2. **Launch Development Shell:**
   ```bash
   docker-compose run --rm kapsule-dev
   ```
3. **Run App (Inside Container):**
   ```bash
   npm install && npm run tauri dev
   ```

---

## 8. Future Considerations
- **Quadlet Support:** Exporting Podman configurations as systemd units for persistent home lab services.
- **GPU Passthrough:** One-click toggle for NVIDIA/AMD hardware acceleration (useful for containerized AI tools or Davinci Resolve).