use std::path::Path;
use tauri::{Manager, State};

const STREAM_TOKEN_HEADER: &str = "X-Teledrive-Stream-Token";

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
    let watch_later_dir = app_handle
        .path()
        .app_data_dir()
        .ok()
        .map(|dir| dir.join("mpv-watch-later"));
    if let Some(dir) = &watch_later_dir {
        let _ = std::fs::create_dir_all(dir);
    }
    let args = build_mpv_args(&url, watch_later_dir.as_deref());
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    // Try to launch bundled sidecar mpv
    use tauri_plugin_shell::ShellExt;
    if let Ok(sidecar) = app_handle.shell().sidecar("mpv") {
        if sidecar.args(arg_refs.clone()).spawn().is_ok() {
            return Ok(());
        }
    }

    // Fallback: Try to launch system-installed mpv from PATH
    std::process::Command::new("mpv")
        .args(&args)
        .spawn()
        .map_err(|e| format!("Gagal menjalankan MPV: {}. Pastikan 'mpv' terpasang.", e))?;
    Ok(())
}

fn build_mpv_args(url: &str, watch_later_dir: Option<&Path>) -> Vec<String> {
    let (stable_url, token) = strip_token_query(url);
    let mut args = vec![
        "--save-position-on-quit".to_string(),
        "--write-filename-in-watch-later-config=yes".to_string(),
    ];
    if let Some(dir) = watch_later_dir {
        args.push(format!("--watch-later-dir={}", dir.display()));
    }
    if let Some(token) = token {
        args.push(format!("--http-header-fields={}: {}", STREAM_TOKEN_HEADER, token));
    }
    args.push(stable_url);
    args
}

fn strip_token_query(url: &str) -> (String, Option<String>) {
    let Some(query_start) = url.find('?') else {
        return (url.to_string(), None);
    };
    let base = &url[..query_start];
    let query = &url[query_start + 1..];
    let mut token = None;
    let mut kept = Vec::new();

    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("token=") {
            if !value.is_empty() {
                token = Some(value.to_string());
            }
        } else if !pair.is_empty() {
            kept.push(pair);
        }
    }

    let stable_url = if kept.is_empty() {
        base.to_string()
    } else {
        format!("{}?{}", base, kept.join("&"))
    };
    (stable_url, token)
}

pub(crate) fn stream_token_header_name() -> &'static str {
    STREAM_TOKEN_HEADER
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn strips_token_from_stream_url_for_stable_mpv_watch_later_key() {
        let (url, token) = strip_token_query("http://localhost:14201/stream/home/10?token=abc123");

        assert_eq!(url, "http://localhost:14201/stream/home/10");
        assert_eq!(token.as_deref(), Some("abc123"));
    }

    #[test]
    fn keeps_non_token_query_params_when_stripping_token() {
        let (url, token) = strip_token_query("http://localhost:14201/stream/home/10?quality=raw&token=abc123&x=1");

        assert_eq!(url, "http://localhost:14201/stream/home/10?quality=raw&x=1");
        assert_eq!(token.as_deref(), Some("abc123"));
    }

    #[test]
    fn build_mpv_args_enable_resume_and_header_auth() {
        let dir = PathBuf::from(r"C:\Teledrive\mpv-watch-later");
        let args = build_mpv_args("http://localhost:14201/stream/home/10?token=abc123", Some(&dir));

        assert!(args.contains(&"--save-position-on-quit".to_string()));
        assert!(args.contains(&r"--watch-later-dir=C:\Teledrive\mpv-watch-later".to_string()));
        assert!(args.contains(&"--http-header-fields=X-Teledrive-Stream-Token: abc123".to_string()));
        assert_eq!(args.last().map(String::as_str), Some("http://localhost:14201/stream/home/10"));
    }
}
