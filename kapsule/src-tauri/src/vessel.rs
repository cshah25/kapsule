use serde::{Deserialize, Serialize};
use tauri::{State, ipc::Channel};
use bollard::container::{Config, CreateContainerOptions, InspectContainerOptions, ListContainersOptions, MemoryStatsStats, StatsOptions, RemoveContainerOptions, LogsOptions};
use bollard::models::{HostConfig, PortBinding};
use bollard::image::{CreateImageOptions, ListImagesOptions};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::AppState;

// ---------------------------------------------------------------------------
// Payload types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct PortMapping {
    pub host: u16,
    pub container: u16,
    /// Bind on all interfaces instead of loopback only.
    #[serde(default)]
    pub expose_lan: bool,
}

#[derive(Deserialize, Debug)]
pub struct MountMapping {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

#[derive(Deserialize, Debug)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateVesselPayload {
    pub name: String,
    pub image: String,
    pub isolated_home: bool,
    pub volatile: bool,
    pub disable_network: bool,
    pub env_vars: Vec<EnvVar>,
    pub ports: Vec<PortMapping>,
    pub mounts: Vec<MountMapping>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct VesselStats {
    pub cpu_percent: f64,
    pub mem_used_mb: f64,
    pub mem_limit_mb: f64,
}

#[derive(Serialize, Clone)]
pub struct VesselInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String, // "running" | "paused" | "restarting" | "stopped" | "error"
    pub cpu_percent: f64,
    pub mem_used_mb: f64,
    pub mem_limit_mb: f64,
}

// ---------------------------------------------------------------------------
// Isolated home helpers
// ---------------------------------------------------------------------------

/// Where the isolated home is mounted inside the container.
const VESSEL_HOME: &str = "/home/kapsule";

/// Root directory holding one `<name>/home` directory per vessel.
fn vessels_root() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("kapsule").join("vessels"))
}

/// Whether the engine runs rootless. Both Docker and Podman report this as a
/// `name=rootless` security option; assume rootful if the query fails, since
/// that only skips keep-id rather than breaking container creation.
async fn is_rootless(docker: &bollard::Docker) -> bool {
    docker
        .info()
        .await
        .ok()
        .and_then(|info| info.security_options)
        .is_some_and(|opts| opts.iter().any(|o| o.contains("name=rootless")))
}

/// Append `:latest` when a reference has neither a tag nor a digest.
/// The tag is after the last `/`, so a registry port (`host:5000/img`) isn't mistaken for one.
fn with_default_tag(image: &str) -> String {
    let last = image.rsplit('/').next().unwrap_or(image);
    if last.contains(':') || image.contains('@') {
        image.to_string()
    } else {
        format!("{}:latest", image)
    }
}

/// Same rule the engines apply to container names (`[a-zA-Z0-9][a-zA-Z0-9_.-]*`).
/// This also rules out `/`, `.` and `..`, so the name is safe as a path component.
fn validate_vessel_name(name: &str) -> Result<(), String> {
    let mut chars = name.chars();
    let valid = matches!(chars.next(), Some(c) if c.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'));
    if valid {
        Ok(())
    } else {
        Err("Vessel name must start with a letter or digit and contain only letters, digits, '_', '.' or '-'".into())
    }
}

/// Find the Kapsule-managed vessel directory (`<vessels_root>/<name>`) behind a
/// container's isolated home bind. Anything outside the vessels root — e.g. a
/// user folder mounted at the same path — is never returned.
fn isolated_vessel_dir(binds: &[String]) -> Option<PathBuf> {
    let root = vessels_root()?.canonicalize().ok()?;
    binds.iter().find_map(|bind| {
        let mut parts = bind.split(':');
        let home_dir = PathBuf::from(parts.next()?);
        if parts.next()? != VESSEL_HOME || home_dir.file_name()? != "home" {
            return None;
        }
        let vessel_dir = home_dir.parent()?.canonicalize().ok()?;
        (vessel_dir.parent()? == root).then_some(vessel_dir)
    })
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn create_vessel(
    state: State<'_, AppState>,
    payload: CreateVesselPayload,
) -> Result<(), String> {
    // Validated before it is used as a path component for the isolated home
    validate_vessel_name(&payload.name)?;

    if payload.image.is_empty() {
        return Err("Image cannot be empty".into());
    }

    let docker = state.get_client().await?;

    let engine = {
        let lock = state.active_engine.lock().await;
        lock.clone().ok_or("No active engine selected")?
    };

    let mut binds = vec![];
    for mount in payload.mounts {
        // A relative host path would silently become a named volume
        if !mount.host_path.starts_with('/') || !mount.container_path.starts_with('/') {
            return Err(format!(
                "Mount paths must be absolute: '{}' -> '{}'",
                mount.host_path, mount.container_path
            ));
        }
        let mut bind = format!("{}:{}", mount.host_path, mount.container_path);
        if mount.read_only {
            bind.push_str(":ro");
        }
        if engine == crate::engine::Engine::Podman {
            bind.push_str(if mount.read_only { ",Z" } else { ":Z" });
        }
        binds.push(bind);
    }

    let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
    for port in payload.ports {
        if port.host == 0 || port.container == 0 {
            return Err("Ports must be between 1 and 65535".into());
        }
        // Loopback by default so mapped services aren't reachable from the LAN
        let host_ip = if port.expose_lan { "0.0.0.0" } else { "127.0.0.1" };
        // Append rather than overwrite so one container port can map to several host ports
        port_bindings
            .entry(format!("{}/tcp", port.container))
            .or_insert_with(|| Some(vec![]))
            .get_or_insert_with(Vec::new)
            .push(PortBinding {
                host_ip: Some(host_ip.to_string()),
                host_port: Some(port.host.to_string()),
            });
    }

    // An untagged reference would make the engine pull *every* tag, and would
    // never match the `repo:tag` entries we compare against below
    let image = with_default_tag(&payload.image);

    // Only pull if image doesn't exist locally
    let images = docker.list_images(Some(ListImagesOptions::<String> {
        all: false,
        ..Default::default()
    })).await.map_err(|e| e.to_string())?;

    // Podman lists Docker Hub images fully qualified (docker.io/library/ubuntu:latest)
    let candidates = [
        image.clone(),
        format!("docker.io/{}", image),
        format!("docker.io/library/{}", image),
    ];
    let image_exists = images.iter().any(|img| {
        img.repo_tags.iter().any(|t| candidates.contains(t))
    });

    if !image_exists {
        let mut pull_stream = docker.create_image(
            Some(CreateImageOptions {
                from_image: image.clone(),
                ..Default::default()
            }),
            None,
            None,
        );
        while let Some(result) = pull_stream.next().await {
            if let Err(e) = result {
                return Err(format!("Failed to pull image: {}", e));
            }
        }
    }

    // Create and start container
    let mut env = vec![];
    for var in payload.env_vars {
        if !var.key.is_empty() {
            env.push(format!("{}={}", var.key, var.value));
        }
    }

    // Directory generation for isolated home. Done after the pull so a failed
    // pull doesn't leave an empty vessel directory behind.
    let mut created_vessel_dir = None;
    if payload.isolated_home {
        let vessel_dir = vessels_root()
            .ok_or("Could not determine the local data directory for isolated homes")?
            .join(&payload.name);
        let home_dir = vessel_dir.join("home");

        if !vessel_dir.exists() {
            created_vessel_dir = Some(vessel_dir.clone());
        }
        std::fs::create_dir_all(&home_dir).map_err(|e| format!("Failed to create isolated home: {}", e))?;

        let mut bind = format!("{}:{}", home_dir.display(), VESSEL_HOME);
        if engine == crate::engine::Engine::Podman {
            bind.push_str(":Z"); // SELinux relabeling
        }
        binds.push(bind);

        // Without this the image's default $HOME (usually /root) is used and the mount is ignored
        if !env.iter().any(|e| e.starts_with("HOME=")) {
            env.push(format!("HOME={}", VESSEL_HOME));
        }
    }

    let network_mode = if payload.disable_network {
        Some("none".to_string())
    } else {
        None
    };

    // keep-id maps the host user into the container, but rootful Podman rejects it
    let userns_mode = if engine == crate::engine::Engine::Podman && is_rootless(&docker).await {
        Some("keep-id".to_string())
    } else {
        None
    };

    let host_config = HostConfig {
        binds: Some(binds),
        port_bindings: Some(port_bindings),
        auto_remove: Some(payload.volatile),
        network_mode,
        userns_mode,
        ..Default::default()
    };

    let config = Config {
        image: Some(image),
        host_config: Some(host_config),
        env: Some(env),
        tty: Some(true),
        open_stdin: Some(true),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: payload.name,
        platform: None,
    };

    let container = match docker.create_container(Some(options), config).await {
        Ok(c) => c,
        Err(e) => {
            // e.g. name already taken — don't leave an orphaned home behind
            if let Some(dir) = created_vessel_dir {
                let _ = std::fs::remove_dir_all(dir);
            }
            return Err(format!("Create container failed: {}", e));
        }
    };
    docker.start_container::<String>(&container.id, None).await.map_err(|e| format!("Start container failed: {}", e))?;

    Ok(())
}

/// Directory holding isolated vessel homes, for display in Settings.
#[tauri::command]
pub fn get_vessels_root() -> Option<String> {
    vessels_root().map(|p| p.display().to_string())
}

#[tauri::command]
pub async fn list_vessels(state: State<'_, AppState>) -> Result<Vec<VesselInfo>, String> {
    // Propagate connection errors — an unreachable engine must not look like "no vessels"
    let docker = state.get_client().await?;
    
    let containers = docker.list_containers(Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    })).await.map_err(|e| e.to_string())?;

    let vessels: Vec<VesselInfo> = containers.iter().map(|c| {
        let name = c.names.clone().unwrap_or_default()
            .first().cloned().unwrap_or_default()
            .trim_start_matches('/').to_string();
        let status = match c.state.as_deref() {
            Some("running") => "running",
            Some("paused") => "paused",
            Some("restarting") => "restarting",
            Some("exited") | Some("created") => "stopped",
            _ => "error", // "dead", "removing", or unknown
        };
        VesselInfo {
            id: c.id.clone().unwrap_or_default(),
            name,
            image: c.image.clone().unwrap_or_default(),
            status: status.to_string(),
            cpu_percent: 0.0,
            mem_used_mb: 0.0,
            mem_limit_mb: 0.0,
        }
    }).collect();

    Ok(vessels)
}

#[tauri::command]
pub async fn get_all_vessel_stats(state: State<'_, AppState>) -> Result<HashMap<String, VesselStats>, String> {
    let docker = match state.get_client().await {
        Ok(d) => d,
        Err(_) => return Ok(HashMap::new()),
    };
    
    let containers = docker.list_containers(Some(ListContainersOptions::<String> {
        all: false,
        ..Default::default()
    })).await.map_err(|e| e.to_string())?;

    let mut result = HashMap::new();
    let stats_futures: Vec<_> = containers.iter().map(|c| {
        let docker = docker.clone();
        let id = c.id.clone().unwrap_or_default();
        async move {
            let mut stream = docker.stats(&id, Some(StatsOptions { stream: false, ..Default::default() }));
            let stats = stream.next().await;
            (id, stats)
        }
    }).collect();

    let stats_results = futures_util::future::join_all(stats_futures).await;

    for (id, stat_res) in stats_results {
        if let Some(Ok(stats)) = stat_res {
            // Exclude reclaimable page cache, as `docker stats` does
            let inactive_file = match stats.memory_stats.stats {
                Some(MemoryStatsStats::V1(s)) => s.total_inactive_file,
                Some(MemoryStatsStats::V2(s)) => s.inactive_file,
                None => 0,
            };
            let mem_usage = stats.memory_stats.usage.unwrap_or(0).saturating_sub(inactive_file) as f64;
            let mem_limit = stats.memory_stats.limit.unwrap_or(0) as f64;
            let mem_used_mb = (mem_usage / 1024.0 / 1024.0 * 10.0).round() / 10.0;
            let mem_limit_mb = (mem_limit / 1024.0 / 1024.0).round();

            let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64 - stats.precpu_stats.cpu_usage.total_usage as f64;
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64 - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
            // Share of the whole machine (0-100), not `docker stats`' per-core
            // figure that exceeds 100% on multi-core hosts. Keeps the card's arc,
            // its label, and the dashboard total consistent.
            let mut cpu_percent = 0.0;
            if system_delta > 0.0 && cpu_delta > 0.0 {
                cpu_percent = ((cpu_delta / system_delta) * 100.0 * 10.0).round() / 10.0;
            }
            result.insert(id, VesselStats { cpu_percent, mem_used_mb, mem_limit_mb });
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn start_vessel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let docker = state.get_client().await?;
    docker.start_container::<String>(&id, None).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_vessel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let docker = state.get_client().await?;
    docker.unpause_container(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_vessel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let docker = state.get_client().await?;
    docker.stop_container(&id, None).await.map_err(|e| e.to_string())
}

/// Force-removes the container and its isolated home directory.
/// Returns a warning if the container was removed but its home could not be.
#[tauri::command]
pub async fn delete_vessel(state: State<'_, AppState>, id: String) -> Result<Option<String>, String> {
    let docker = state.get_client().await?;

    // Look up the home before removal — the binds are gone once the container is
    let vessel_dir = docker
        .inspect_container(&id, None::<InspectContainerOptions>)
        .await
        .ok()
        .and_then(|info| info.host_config?.binds)
        .and_then(|binds| isolated_vessel_dir(&binds));

    docker.remove_container(&id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await.map_err(|e| e.to_string())?;

    if let Some(dir) = vessel_dir {
        if let Err(e) = std::fs::remove_dir_all(&dir) {
            // Typically root-owned files written by a rootful Docker container
            return Ok(Some(format!(
                "Vessel removed, but its home directory {} could not be deleted: {}",
                dir.display(),
                e
            )));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn stream_vessel_logs(
    state: State<'_, AppState>,
    id: String,
    on_message: Channel<String>,
) -> Result<(), String> {
    let docker = state.get_client().await?;

    let handle = tauri::async_runtime::spawn(async move {
        let mut stream = docker.logs(&id, Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            tail: "100".to_string(),
            ..Default::default()
        }));

        while let Some(log_result) = stream.next().await {
            match log_result {
                Ok(log_output) => {
                    let text = log_output.to_string();
                    if on_message.send(text).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        // Reached when the container isn't (or stops) running; aborts skip this
        let _ = on_message.send("\n\x1b[90m— end of logs: container is not running —\x1b[0m\n".to_string());
    });

    // The webview keeps the channel alive, so `send` rarely fails on its own —
    // abort any previous follower explicitly instead of leaking it.
    if let Some(prev) = state.log_stream.lock().await.replace(handle) {
        prev.abort();
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_vessel_logs(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(handle) = state.log_stream.lock().await.take() {
        handle.abort();
    }
    Ok(())
}

#[tauri::command]
pub async fn list_local_images(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let docker = state.get_client().await?;

    let images = docker.list_images(Some(ListImagesOptions::<String> {
        all: false,
        ..Default::default()
    })).await.map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for img in images {
        for tag in img.repo_tags {
            if tag != "<none>:<none>" {
                result.push(tag);
            }
        }
    }
    result.sort();
    result.dedup();
    Ok(result)
}
