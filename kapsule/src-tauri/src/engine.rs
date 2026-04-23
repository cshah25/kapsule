/// engine.rs — Dual-engine abstraction layer (Podman / Docker)
///
/// On startup, Kapsule probes for available container engine sockets and
/// reports back to the frontend which engines are available.  The user can
/// then choose a preferred engine; subsequent Bollard commands will target
/// the appropriate socket.

use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
}

// ---------------------------------------------------------------------------
// Socket discovery
// ---------------------------------------------------------------------------

/// Return the most likely Podman socket path for the current user.
fn podman_socket_path() -> PathBuf {
    // Rootless Podman: /run/user/<uid>/podman/podman.sock
    if let Ok(uid) = std::env::var("UID")
        .or_else(|_| {
            // Fallback: read /proc/self/status
            std::fs::read_to_string("/proc/self/status")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("Uid:"))
                        .and_then(|l| l.split_whitespace().nth(1).map(String::from))
                })
                .ok_or_else(|| std::env::VarError::NotPresent)
        })
    {
        PathBuf::from(format!("/run/user/{}/podman/podman.sock", uid))
    } else {
        // Last resort
        PathBuf::from("/run/user/1000/podman/podman.sock")
    }
}

fn docker_socket_path() -> PathBuf {
    PathBuf::from("/var/run/docker.sock")
}

// ---------------------------------------------------------------------------
// Async probe
// ---------------------------------------------------------------------------

async fn probe_socket(socket_path: &PathBuf) -> Option<Docker> {
    if !socket_path.exists() {
        return None;
    }
    let url = format!("unix://{}", socket_path.display());
    match Docker::connect_with_socket(&url, 30, bollard::API_DEFAULT_VERSION) {
        Ok(docker) => {
            // Ping to confirm the daemon is actually responding
            match docker.ping().await {
                Ok(_) => Some(docker),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

// ---------------------------------------------------------------------------
// Public API called from Tauri commands
// ---------------------------------------------------------------------------

/// Detect available engines and return an [`EngineStatus`] to the frontend.
pub async fn detect_engines() -> EngineStatus {
    let podman_path = podman_socket_path();
    let docker_path = docker_socket_path();

    let podman_ok = probe_socket(&podman_path).await.is_some();
    let docker_ok = probe_socket(&docker_path).await.is_some();

    // Prefer Podman (rootless / more secure) when both are present.
    let active_engine = if podman_ok {
        Some(Engine::Podman)
    } else if docker_ok {
        Some(Engine::Docker)
    } else {
        None
    };

    EngineStatus {
        podman_available: podman_ok,
        docker_available: docker_ok,
        podman_socket: if podman_ok {
            Some(podman_path.display().to_string())
        } else {
            None
        },
        docker_socket: if docker_ok {
            Some(docker_path.display().to_string())
        } else {
            None
        },
        active_engine,
    }
}

/// Build a [`Docker`] client pointed at the requested engine socket.
/// Returns `None` if the socket is unavailable.
pub async fn connect(engine: Engine) -> Option<Docker> {
    let path = match engine {
        Engine::Podman => podman_socket_path(),
        Engine::Docker => docker_socket_path(),
    };
    probe_socket(&path).await
}
