use serde::{Deserialize, Serialize};
use tauri::{State, ipc::Channel};
use bollard::container::{Config, CreateContainerOptions, ListContainersOptions, StatsOptions, RemoveContainerOptions, LogsOptions};
use bollard::models::{HostConfig, PortBinding};
use bollard::image::CreateImageOptions;
use futures_util::StreamExt;
use std::collections::HashMap;

use crate::AppState;

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
pub struct CreateVesselPayload {
    pub name: String,
    pub image: String,
    pub isolated_home: bool,
    pub volatile: bool,
    pub ports: Vec<PortMapping>,
    pub mounts: Vec<MountMapping>,
}

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

    // 1. Get docker connection based on active engine
    let engine = {
        let lock = state.active_engine.lock().await;
        lock.clone().ok_or("No active engine selected")?
    };

    let docker = crate::engine::connect(engine).await.ok_or("Failed to connect to container engine")?;

    // 2. Directory generation for isolated home
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

    // 3. Pull image
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

    // 4. Create and start container
    let host_config = HostConfig {
        binds: Some(binds),
        port_bindings: Some(port_bindings),
        auto_remove: Some(payload.volatile),
        ..Default::default()
    };

    let config = Config {
        image: Some(payload.image),
        host_config: Some(host_config),
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

#[derive(Serialize)]
pub struct VesselInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub cpu_percent: f64,
    pub mem_used_mb: f64,
    pub mem_limit_mb: f64,
}

#[tauri::command]
pub async fn list_vessels(state: State<'_, AppState>) -> Result<Vec<VesselInfo>, String> {
    let engine = {
        let lock = state.active_engine.lock().await;
        if let Some(e) = lock.clone() { e } else { return Ok(vec![]); }
    };

    let docker = crate::engine::connect(engine).await.ok_or("Failed to connect to engine")?;
    
    let containers = docker.list_containers(Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    })).await.map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for c in containers {
        let name = c.names.unwrap_or_default().first().cloned().unwrap_or_default().trim_start_matches('/').to_string();
        let status = match c.state.as_deref() {
            Some("running") => "running",
            Some("exited") | Some("created") => "stopped",
            _ => "error",
        };

        let mut cpu_percent = 0.0;
        let mut mem_used_mb = 0.0;
        let mut mem_limit_mb = 0.0;

        let id = c.id.clone().unwrap_or_default();

        if status == "running" {
            let mut stream = docker.stats(&id, Some(StatsOptions { stream: false, ..Default::default() }));
            if let Some(Ok(stats)) = stream.next().await {
                let mem_usage = stats.memory_stats.usage.unwrap_or(0) as f64;
                let mem_limit = stats.memory_stats.limit.unwrap_or(0) as f64;
                mem_used_mb = mem_usage / 1024.0 / 1024.0;
                mem_limit_mb = mem_limit / 1024.0 / 1024.0;

                let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64 - stats.precpu_stats.cpu_usage.total_usage as f64;
                let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64 - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
                if system_delta > 0.0 && cpu_delta > 0.0 {
                    let online_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
                    cpu_percent = (cpu_delta / system_delta) * online_cpus * 100.0;
                }
            }
        }
        
        result.push(VesselInfo {
            id,
            name,
            image: c.image.unwrap_or_default(),
            status: status.to_string(),
            cpu_percent,
            mem_used_mb,
            mem_limit_mb,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn start_vessel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let engine = state.active_engine.lock().await.clone().ok_or("No active engine")?;
    let docker = crate::engine::connect(engine).await.ok_or("Failed to connect")?;
    docker.start_container::<String>(&id, None).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_vessel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let engine = state.active_engine.lock().await.clone().ok_or("No active engine")?;
    let docker = crate::engine::connect(engine).await.ok_or("Failed to connect")?;
    docker.stop_container(&id, None).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_vessel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let engine = state.active_engine.lock().await.clone().ok_or("No active engine")?;
    let docker = crate::engine::connect(engine).await.ok_or("Failed to connect")?;
    docker.remove_container(&id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stream_vessel_logs(
    state: State<'_, AppState>,
    id: String,
    on_message: Channel<String>,
) -> Result<(), String> {
    let engine = state.active_engine.lock().await.clone().ok_or("No active engine")?;
    let docker = crate::engine::connect(engine).await.ok_or("Failed to connect")?;

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
