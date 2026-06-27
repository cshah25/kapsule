/// lib.rs — Kapsule Tauri application entry point
///
/// All Tauri commands exposed to the Svelte frontend are registered here.
/// Heavy async work is dispatched via Tokio; results are serialised to JSON
/// automatically by `serde`.

mod engine;
mod vessel;
mod library;
mod desktop;

use engine::{detect_engines, Engine, EngineStatus};
use tauri::State;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Global app state
// ---------------------------------------------------------------------------

/// Shared mutable state owned by the Tauri runtime.
pub struct AppState {
    /// The engine currently selected by the user (or auto-detected).
    pub active_engine: Mutex<Option<Engine>>,
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Probe for available container engines and return their status.
/// Called once at startup from the Svelte `onMount`.
#[tauri::command]
async fn get_engine_status(state: State<'_, AppState>) -> Result<EngineStatus, String> {
    let status = detect_engines().await;
    if let Some(engine) = status.active_engine {
        let mut lock = state.active_engine.lock().await;
        if lock.is_none() {
            *lock = Some(engine);
        }
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
    // Verify the requested engine is actually reachable
    let _docker = engine::connect(engine)
        .await
        .ok_or_else(|| format!("{:?} is not reachable — is the daemon running?", engine))?;

    let mut lock = state.active_engine.lock().await;
    *lock = Some(engine);
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
        })
        .invoke_handler(tauri::generate_handler![
            get_engine_status, 
            set_engine, 
            vessel::create_vessel,
            vessel::list_vessels,
            vessel::start_vessel,
            vessel::stop_vessel,
            vessel::delete_vessel,
            vessel::stream_vessel_logs,
            vessel::list_local_images,
            library::search_images,
            library::get_image_tags,
            library::pull_image,
            desktop::generate_desktop_entry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
