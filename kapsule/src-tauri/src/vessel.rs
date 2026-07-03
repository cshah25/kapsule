use serde::{Deserialize, Serialize};
use tauri::{State, ipc::Channel};
use bollard::container::{Config, CreateContainerOptions, ListContainersOptions, StatsOptions, RemoveContainerOptions, LogsOptions};
use bollard::models::{HostConfig, PortBinding};
use bollard::image::{CreateImageOptions, ListImagesOptions};
use futures_util::StreamExt;
use std::collections::HashMap;

use crate::AppState;

// ---------------------------------------------------------------------------
// Payload types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct PortMapping {
    pub host: u16,
    pub container: u16,
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
    pub status: String, // "running" | "stopped" | "error"
    pub cpu_percent: f64,
    pub mem_used_mb: f64,
    pub mem_limit_mb: f64,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn create_vessel(
    state: State<'_, AppState>,
    payload: CreateVesselPayload,
) -> Result<(), String> {
    if payload.name.is_empty() {
        return Err("Vessel name cannot be empty".into());
    }
    
    if payload.image.is_empty() {
        return Err("Image cannot be empty".into());
    }

    let docker = state.get_client().await?;

    let engine = {
        let lock = state.active_engine.lock().await;
        lock.clone().ok_or("No active engine selected")?
    };

    // Directory generation for isolated home
    let mut binds = vec![];
    if payload.isolated_home {
        if let Some(mut data_dir) = dirs::data_local_dir() {
            data_dir.push("kapsule");
            data_dir.push("vessels");
            data_dir.push(&payload.name);
            data_dir.push("home");

            std::fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create isolated home: {}", e))?;

            let mut bind = format!("{}:/home/kapsule", data_dir.display());
            if engine == crate::engine::Engine::Podman {
                bind.push_str(":Z"); // SELinux relabeling
            }
            binds.push(bind);
        }
    }

    for mount in payload.mounts {
        let mut bind = format!("{}:{}", mount.host_path, mount.container_path);
        if mount.read_only {
            bind.push_str(":ro");
        }
        if engine == crate::engine::Engine::Podman {
            bind.push_str(if mount.read_only { ",Z" } else { ":Z" });
        }
        binds.push(bind);
    }

    let mut port_bindings = HashMap::new();
    for port in payload.ports {
        let key = format!("{}/tcp", port.container);
        port_bindings.insert(key, Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(port.host.to_string()),
        }]));
    }

    // Only pull if image doesn't exist locally
    let images = docker.list_images(Some(ListImagesOptions::<String> {
        all: false,
        ..Default::default()
    })).await.map_err(|e| e.to_string())?;

    let image_exists = images.iter().any(|img| {
        img.repo_tags.iter().any(|t| *t == payload.image)
    });

    if !image_exists {
        let mut pull_stream = docker.create_image(
            Some(CreateImageOptions {
                from_image: payload.image.clone(),
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

    let network_mode = if payload.disable_network {
        Some("none".to_string())
    } else {
        None
    };

    let userns_mode = if engine == crate::engine::Engine::Podman {
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
        image: Some(payload.image),
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

    let container = docker.create_container(Some(options), config).await.map_err(|e| format!("Create container failed: {}", e))?;
    docker.start_container::<String>(&container.id, None).await.map_err(|e| format!("Start container failed: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn list_vessels(state: State<'_, AppState>) -> Result<Vec<VesselInfo>, String> {
    let docker = match state.get_client().await {
        Ok(d) => d,
        Err(_) => return Ok(vec![]), // No engine = no vessels
    };
    
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
            Some("exited") | Some("created") => "stopped",
            _ => "error",
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
            let mem_usage = stats.memory_stats.usage.unwrap_or(0) as f64;
            let mem_limit = stats.memory_stats.limit.unwrap_or(0) as f64;
            let mem_used_mb = (mem_usage / 1024.0 / 1024.0 * 10.0).round() / 10.0;
            let mem_limit_mb = (mem_limit / 1024.0 / 1024.0).round();

            let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64 - stats.precpu_stats.cpu_usage.total_usage as f64;
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64 - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
            let mut cpu_percent = 0.0;
            if system_delta > 0.0 && cpu_delta > 0.0 {
                let online_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
                cpu_percent = ((cpu_delta / system_delta) * online_cpus * 100.0 * 10.0).round() / 10.0;
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
pub async fn stop_vessel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let docker = state.get_client().await?;
    docker.stop_container(&id, None).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_vessel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let docker = state.get_client().await?;
    docker.remove_container(&id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stream_vessel_logs(
    state: State<'_, AppState>,
    id: String,
    on_message: Channel<String>,
) -> Result<(), String> {
    let docker = state.get_client().await?;

    tauri::async_runtime::spawn(async move {
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
    });

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
