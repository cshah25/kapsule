/// lib.rs — Kapsule Tauri application entry point
///
/// All Tauri commands exposed to the Svelte frontend are registered here.
/// Heavy async work is dispatched via Tokio; results are serialised to JSON
/// automatically by `serde`.

mod engine;
mod vessel;
mod library;

use engine::{detect_engines, Engine, EngineStatus};
use bollard::Docker;
use tauri::State;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Global app state
// ---------------------------------------------------------------------------

/// Shared mutable state owned by the Tauri runtime.
pub struct AppState {
    /// The engine currently selected by the user (or auto-detected).
    pub active_engine: Mutex<Option<Engine>>,
    /// Cached Docker/Podman client — avoids reconnecting on every command.
    pub docker_client: Mutex<Option<Docker>>,
    /// Background task following the logs shown in the terminal overlay.
    /// Only one overlay is open at a time, so starting a new stream aborts the old one.
    pub log_stream: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

impl AppState {
    /// Get the cached Docker client, or create and cache a new one.
    /// This avoids re-probing sockets and re-pinging on every Tauri command.
    pub async fn get_client(&self) -> Result<Docker, String> {
        // Fast path: return cached client
        {
            let lock = self.docker_client.lock().await;
            if let Some(ref client) = *lock {
                return Ok(client.clone());
            }
        }
        // Slow path: create new connection
        let engine = self.active_engine.lock().await.clone()
            .ok_or("No active engine selected")?;
        let client = engine::connect(engine).await
            .ok_or("Failed to connect to container engine")?;
        // Cache for next time
        {
            let mut lock = self.docker_client.lock().await;
            *lock = Some(client.clone());
        }
        Ok(client)
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Probe for available container engines and return their status.
/// Called at startup and again whenever the user asks to retry detection.
#[tauri::command]
async fn get_engine_status(state: State<'_, AppState>) -> Result<EngineStatus, String> {
    let mut status = detect_engines().await;
    let available = |e: Engine| match e {
        Engine::Podman => status.podman_available,
        Engine::Docker => status.docker_available,
    };

    // Keep the engine already in use if it's still up (a retry shouldn't flip
    // engines), else the saved preference, else the detection default
    let current = *state.active_engine.lock().await;
    let chosen = current
        .filter(|&e| available(e))
        .or_else(|| engine::load_preferred_engine().filter(|&e| available(e)))
        .or(status.active_engine);

    // Report what the backend actually uses, so the UI can't disagree with it
    status.active_engine = chosen;
    *state.active_engine.lock().await = chosen;
    // Drop the cached client so a retry reconnects (e.g. after fixing socket permissions)
    *state.docker_client.lock().await = None;

    if chosen.is_some() {
        // Pre-cache the client so the first list_vessels call is fast
        let _ = state.get_client().await;
    }
    Ok(status)
}

/// Switch the active engine.  Frontend passes `"podman"` or `"docker"`.
/// Verifies the engine is actually reachable before committing the switch.
#[tauri::command]
async fn set_engine(
    state: State<'_, AppState>,
    engine: Engine,
) -> Result<(), String> {
    // Verify and get client in one step
    let client = engine::connect(engine)
        .await
        .ok_or_else(|| format!("{:?} is not reachable — is the daemon running?", engine))?;

    // Update engine
    let mut engine_lock = state.active_engine.lock().await;
    *engine_lock = Some(engine);
    drop(engine_lock);

    // Replace cached client
    let mut client_lock = state.docker_client.lock().await;
    *client_lock = Some(client);

    // Remember the choice for next launch; failing to save shouldn't fail the switch
    if let Err(e) = engine::save_preferred_engine(engine) {
        eprintln!("[kapsule] Could not save engine preference: {}", e);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Application bootstrap
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            active_engine: Mutex::new(None),
            docker_client: Mutex::new(None),
            log_stream: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_engine_status, 
            set_engine, 
            vessel::create_vessel,
            vessel::list_vessels,
            vessel::get_vessels_root,
            vessel::get_all_vessel_stats,
            vessel::start_vessel,
            vessel::resume_vessel,
            vessel::stop_vessel,
            vessel::delete_vessel,
            vessel::stream_vessel_logs,
            vessel::stop_vessel_logs,
            vessel::list_local_images,
            library::search_images,
            library::get_image_tags,
            library::pull_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
