use tauri::State;

/// Holds the per-session streaming config (token + port)
pub struct StreamConfig {
    pub token: String,
    pub port: u16,
}

/// Returned to the frontend so it can construct stream URLs dynamically
#[derive(serde::Serialize)]
pub struct StreamInfo {
    pub token: String,
    pub base_url: String,
}

/// Returns the streaming server's session token and base URL to the frontend.
/// The frontend must use the returned base_url to construct stream URLs,
/// never hardcoding the port.
#[tauri::command]
pub fn cmd_get_stream_info(config: State<'_, StreamConfig>) -> StreamInfo {
    // Always use "localhost" on all platforms.
    // "localhost" is treated as a secure context by all major browser
    // engines (Chromium/WebView2, WebKit) and is exempt from Mixed Content
    // blocking.  This is critical on Windows where Tauri v2 serves the
    // frontend from https://tauri.localhost — fetching http://127.0.0.1
    // from an HTTPS origin triggers a Mixed Content block in WebView2.
    // The server binds exclusively to 127.0.0.1, so name resolution
    // differences between platforms are not a concern.
    let host = "localhost";

    StreamInfo {
        token: config.token.clone(),
        base_url: format!("http://{}:{}", host, config.port),
    }
}

#[tauri::command]
pub fn cmd_play_in_mpv(url: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    // Try to launch bundled sidecar mpv
    use tauri_plugin_shell::ShellExt;
    if let Ok(sidecar) = app_handle.shell().sidecar("mpv") {
        if sidecar.args(&[&url]).spawn().is_ok() {
            return Ok(());
        }
    }

    // Fallback: Try to launch system-installed mpv from PATH
    std::process::Command::new("mpv")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Gagal menjalankan MPV: {}. Pastikan 'mpv' terpasang.", e))?;
    Ok(())
}

