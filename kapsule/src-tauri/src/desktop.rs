use tauri::State;
use crate::AppState;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DesktopEntryPayload {
    pub vessel_name: String,
    pub icon_path: Option<String>,
}

#[tauri::command]
pub async fn generate_desktop_entry(
    state: State<'_, AppState>,
    payload: DesktopEntryPayload,
) -> Result<(), String> {
    let engine = state.active_engine.lock().await.clone().ok_or("No active engine")?;
    
    let engine_cmd = match engine {
        crate::engine::Engine::Docker => "docker",
        crate::engine::Engine::Podman => "podman",
    };

    let mut applications_dir = dirs::data_local_dir().ok_or("Could not find local data directory")?;
    applications_dir.push("applications");
    fs::create_dir_all(&applications_dir).map_err(|e| e.to_string())?;

    let desktop_file_path = applications_dir.join(format!("kapsule-{}.desktop", payload.vessel_name));
    
    let icon_line = if let Some(icon) = payload.icon_path {
        if !icon.is_empty() {
            format!("Icon={}", icon)
        } else {
            "Icon=utilities-terminal".to_string()
        }
    } else {
        "Icon=utilities-terminal".to_string()
    };

    let desktop_content = format!(
        "[Desktop Entry]\n\
         Version=1.0\n\
         Type=Application\n\
         Name={}\n\
         Comment=Kapsule Vessel\n\
         Exec={} start {}\n\
         Terminal=false\n\
         Categories=Development;Utility;\n\
         {}\n",
        payload.vessel_name, engine_cmd, payload.vessel_name, icon_line
    );

    fs::write(&desktop_file_path, desktop_content).map_err(|e| format!("Failed to write .desktop file: {}", e))?;

    Ok(())
}
