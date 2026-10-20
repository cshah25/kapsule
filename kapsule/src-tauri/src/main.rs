// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK's DMA-BUF renderer leaves the window blank (or fails to create
    // an EGL display) on some Wayland/GPU setups, notably NixOS. The fallback
    // keeps GPU compositing and only changes how frames reach the window.
    // Set before anything else starts so no other thread can observe the change;
    // a value the user set explicitly is left alone.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    kapsule_lib::run()
}
