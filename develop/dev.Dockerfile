# Arch Linux Development Container for Kapsule
FROM archlinux:latest

# Update and install core development tools
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    gtk3 \
    webkit2gtk-4.1 \
    libappindicator-gtk3 \
    librsvg \
    pkgconf \
    git \
    nodejs \
    npm \
    rustup \
    xterm \
    mesa \
    libx11

# Create a non-root user (important for Tauri and security)
RUN useradd -m developer && \
    echo "developer ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
USER developer

# Initialize Rust as the developer user so the toolchain lands in /home/developer/.rustup
ENV CARGO_HOME=/home/developer/.cargo
ENV RUSTUP_HOME=/home/developer/.rustup
ENV PATH=/home/developer/.cargo/bin:$PATH
RUN rustup default stable

WORKDIR /home/developer/app

# Set environment variables for GUI
ENV DISPLAY=:0
ENV SHELL=/bin/bash

# Default command
CMD ["bash"]
