/// engine.rs — Dual-engine abstraction layer (Podman / Docker)
///
/// On startup, Kapsule probes for available container engine sockets and
/// reports back to the frontend which engines are available.  The user can
/// then choose a preferred engine; subsequent Bollard commands will target
/// the appropriate socket.

use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::os::unix::fs::MetadataExt;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Which container engine is being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Podman,
    Docker,
}

/// Availability status of each engine detected at startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub podman_available: bool,
    pub docker_available: bool,
    /// Socket path actually used for the Podman connection (if available).
    pub podman_socket: Option<String>,
    /// Socket path actually used for the Docker connection (if available).
    pub docker_socket: Option<String>,
    /// The engine that Kapsule has selected as default.
    pub active_engine: Option<Engine>,
    /// Diagnostic hints when detection fails (shown to user in the UI).
    pub diagnostics: Vec<String>,
}

// ---------------------------------------------------------------------------
// Socket discovery
// ---------------------------------------------------------------------------

/// Return candidate Podman socket paths for the current user (ordered by priority).
fn podman_socket_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // 1. XDG_RUNTIME_DIR-based (most common on systemd systems)
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        candidates.push(PathBuf::from(format!("{}/podman/podman.sock", xdg)));
    }

    // 2. Rootless Podman: /run/user/<uid>/podman/podman.sock
    if let Ok(uid) = resolve_uid() {
        candidates.push(PathBuf::from(format!("/run/user/{}/podman/podman.sock", uid)));
    }

    // 3. Root podman socket
    candidates.push(PathBuf::from("/run/podman/podman.sock"));

    // 4. Last resort
    candidates.push(PathBuf::from("/run/user/1000/podman/podman.sock"));

    candidates.dedup();
    candidates
}

/// Return candidate Docker socket paths (ordered by priority).
fn docker_socket_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // 1. Standard Docker socket
    candidates.push(PathBuf::from("/var/run/docker.sock"));

    // 2. Docker Desktop (Linux)
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(PathBuf::from(format!("{}/.docker/desktop/docker.sock", home)));
        candidates.push(PathBuf::from(format!("{}/.docker/run/docker.sock", home)));
    }

    // 3. Snap Docker (Ubuntu)
    candidates.push(PathBuf::from("/run/snap.docker/docker.sock"));

    // 4. Rootless Docker
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        candidates.push(PathBuf::from(format!("{}/docker.sock", xdg)));
    }
    if let Ok(uid) = resolve_uid() {
        candidates.push(PathBuf::from(format!("/run/user/{}/docker.sock", uid)));
    }

    candidates.dedup();
    candidates
}

/// Resolve the current user's UID.
fn resolve_uid() -> Result<String, ()> {
    // Try $UID first, then $EUID, then /proc/self/status
    std::env::var("UID")
        .or_else(|_| std::env::var("EUID"))
        .or_else(|_| {
            std::fs::read_to_string("/proc/self/status")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("Uid:"))
                        .and_then(|l| l.split_whitespace().nth(1).map(String::from))
                })
                .ok_or(())
        })
        .map_err(|_| ())
}

/// Get the effective UID of this process.
fn current_uid() -> u32 {
    // Safe: getuid is always valid on POSIX
    unsafe { libc::getuid() }
}

/// Get the effective GID of this process.
fn current_gid() -> u32 {
    unsafe { libc::getgid() }
}

/// Get all supplementary group IDs of this process.
fn current_groups() -> Vec<u32> {
    let ngroups = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if ngroups <= 0 {
        return vec![];
    }
    let mut groups = vec![0u32 as libc::gid_t; ngroups as usize];
    let n = unsafe { libc::getgroups(ngroups, groups.as_mut_ptr()) };
    if n < 0 {
        return vec![];
    }
    groups.truncate(n as usize);
    groups.iter().map(|g| *g as u32).collect()
}

/// Check if the current user can read/write a socket file.
/// Returns (can_access, diagnostic_hint).
fn check_socket_permissions(path: &PathBuf) -> (bool, Option<String>) {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return (false, None), // Can't even stat it
    };

    let file_uid = meta.uid();
    let file_gid = meta.gid();
    let mode = meta.mode();

    let uid = current_uid();
    let gid = current_gid();
    let groups = current_groups();

    // Check: owner match
    if uid == file_uid {
        if mode & 0o600 != 0 {
            return (true, None);
        }
    }

    // Check: group match (primary or supplementary)
    if gid == file_gid || groups.contains(&file_gid) {
        if mode & 0o060 != 0 {
            return (true, None);
        }
    }

    // Check: other
    if mode & 0o006 != 0 {
        return (true, None);
    }

    // No access — build a helpful diagnostic
    let group_name = resolve_group_name(file_gid);
    let hint = format!(
        "Socket {} is owned by {}:{} (mode {:o}). Your user (uid={}) is not in the '{}' group. \
         Run: sudo usermod -aG {} $USER — then log out and back in.",
        path.display(),
        file_uid,
        file_gid,
        mode & 0o7777,
        uid,
        group_name,
        group_name
    );

    (false, Some(hint))
}

/// Try to resolve a GID to a group name by reading /etc/group.
fn resolve_group_name(gid: u32) -> String {
    if let Ok(contents) = std::fs::read_to_string("/etc/group") {
        for line in contents.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(g) = parts[2].parse::<u32>() {
                    if g == gid {
                        return parts[0].to_string();
                    }
                }
            }
        }
    }
    format!("gid-{}", gid)
}

// ---------------------------------------------------------------------------
// Async probe
// ---------------------------------------------------------------------------

/// Result of probing a socket — tracks why it failed for diagnostics.
enum ProbeResult {
    Ok(Docker),
    PermissionDenied(String),
    ConnectionFailed(String),
    NotFound,
}

async fn probe_socket(socket_path: &PathBuf) -> ProbeResult {
    // First check if the socket exists
    if !socket_path.exists() {
        return ProbeResult::NotFound;
    }

    // Check permissions before attempting connection
    let (has_access, perm_hint) = check_socket_permissions(socket_path);
    if !has_access {
        if let Some(hint) = perm_hint {
            eprintln!(
                "[kapsule] ✗ Permission denied for {}: {}",
                socket_path.display(),
                hint
            );
            return ProbeResult::PermissionDenied(hint);
        }
    }

    // Try to connect
    let url = format!("unix://{}", socket_path.display());
    match Docker::connect_with_socket(&url, 5, bollard::API_DEFAULT_VERSION) {
        Ok(docker) => {
            match docker.ping().await {
                Ok(_) => {
                    eprintln!("[kapsule] ✓ Engine responding at {}", socket_path.display());
                    ProbeResult::Ok(docker)
                }
                Err(e) => {
                    let err_str = e.to_string();
                    eprintln!(
                        "[kapsule] ✗ Socket exists at {} but ping failed: {}",
                        socket_path.display(),
                        err_str
                    );
                    // If we already know it's a permission issue, use that hint
                    if !has_access {
                        if let Some(hint) = check_socket_permissions(socket_path).1 {
                            return ProbeResult::PermissionDenied(hint);
                        }
                    }
                    // Connection succeeded but ping failed — could be permission
                    // at the daemon level or the daemon is starting up
                    if err_str.contains("Permission denied") || err_str.contains("permission") {
                        let group_hint = format!(
                            "Connection to {} was refused (permission denied). \
                             Run: sudo usermod -aG docker $USER — then log out and back in.",
                            socket_path.display()
                        );
                        ProbeResult::PermissionDenied(group_hint)
                    } else {
                        ProbeResult::ConnectionFailed(format!(
                            "Socket at {} exists but the daemon is not responding: {}",
                            socket_path.display(),
                            err_str
                        ))
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "[kapsule] ✗ Could not connect to {}: {}",
                socket_path.display(),
                e
            );
            ProbeResult::ConnectionFailed(format!(
                "Could not connect to {}: {}",
                socket_path.display(),
                e
            ))
        }
    }
}

/// Try each candidate socket in order and return the first one that responds.
/// Collects diagnostics for all failures.
async fn probe_candidates(candidates: &[PathBuf], diagnostics: &mut Vec<String>) -> Option<(Docker, PathBuf)> {
    for path in candidates {
        match probe_socket(path).await {
            ProbeResult::Ok(docker) => return Some((docker, path.clone())),
            ProbeResult::PermissionDenied(hint) => {
                if !diagnostics.contains(&hint) {
                    diagnostics.push(hint);
                }
            }
            ProbeResult::ConnectionFailed(hint) => {
                eprintln!("[kapsule] {}", hint);
            }
            ProbeResult::NotFound => {
                eprintln!("[kapsule] Skipping {} (not found)", path.display());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Public API called from Tauri commands
// ---------------------------------------------------------------------------

/// Detect available engines and return an [`EngineStatus`] to the frontend.
pub async fn detect_engines() -> EngineStatus {
    eprintln!("[kapsule] Starting engine detection...");

    let podman_candidates = podman_socket_candidates();
    let docker_candidates = docker_socket_candidates();

    eprintln!("[kapsule] Podman candidates: {:?}", podman_candidates);
    eprintln!("[kapsule] Docker candidates: {:?}", docker_candidates);

    let mut diagnostics = Vec::new();

    let podman_result = probe_candidates(&podman_candidates, &mut diagnostics).await;
    let docker_result = probe_candidates(&docker_candidates, &mut diagnostics).await;

    let podman_ok = podman_result.is_some();
    let docker_ok = docker_result.is_some();

    let podman_socket_str = podman_result.as_ref().map(|(_, p)| p.display().to_string());
    let docker_socket_str = docker_result.as_ref().map(|(_, p)| p.display().to_string());

    // Prefer Docker per user request, fallback to Podman.
    let active_engine = if docker_ok {
        Some(Engine::Docker)
    } else if podman_ok {
        Some(Engine::Podman)
    } else {
        None
    };

    // If nothing was found AND no permission diagnostics were generated,
    // add a generic "not installed" hint.
    if active_engine.is_none() && diagnostics.is_empty() {
        diagnostics.push(
            "Neither Podman nor Docker appears to be installed. \
             Install Docker: https://docs.docker.com/engine/install/ \
             or Podman: https://podman.io/docs/installation"
                .to_string(),
        );
    }

    eprintln!(
        "[kapsule] Detection result: podman={}, docker={}, active={:?}, diagnostics={}",
        podman_ok, docker_ok, active_engine, diagnostics.len()
    );

    EngineStatus {
        podman_available: podman_ok,
        docker_available: docker_ok,
        podman_socket: podman_socket_str,
        docker_socket: docker_socket_str,
        active_engine,
        diagnostics,
    }
}

/// Build a [`Docker`] client pointed at the requested engine socket.
/// Returns `None` if the socket is unavailable.
pub async fn connect(engine: Engine) -> Option<Docker> {
    let candidates = match engine {
        Engine::Podman => podman_socket_candidates(),
        Engine::Docker => docker_socket_candidates(),
    };
    let mut diag = Vec::new();
    probe_candidates(&candidates, &mut diag).await.map(|(docker, _)| docker)
}
