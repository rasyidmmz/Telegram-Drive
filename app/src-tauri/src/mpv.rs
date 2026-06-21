pub(crate) fn validate_stream_url(url: &str) -> Result<tauri::Url, String> {
    let parsed = tauri::Url::parse(url).map_err(|_| "Invalid stream URL".to_string())?;
    let is_loopback = matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    let has_token = parsed
        .query_pairs()
        .any(|(key, value)| key == "token" && !value.is_empty());

    if parsed.scheme() != "http"
        || !is_loopback
        || parsed.port() != Some(crate::STREAM_PORT)
        || !parsed.path().starts_with("/stream/")
        || !has_token
    {
        return Err("Only authenticated local Telegram Drive streams can be played".to_string());
    }

    Ok(parsed)
}

pub(crate) fn build_mpv_args(url: &str, filename: &str) -> Result<Vec<String>, String> {
    validate_stream_url(url)?;

    Ok(vec![
        "--no-config".to_string(),
        "--terminal=no".to_string(),
        "--force-window=yes".to_string(),
        "--keep-open=no".to_string(),
        "--hwdec=auto-safe".to_string(),
        "--osc=yes".to_string(),
        "--input-default-bindings=yes".to_string(),
        "--sid=auto".to_string(),
        format!("--title={filename}"),
        "--".to_string(),
        url.to_string(),
    ])
}

pub(crate) trait ManagedChild {
    fn pid(&self) -> u32;
    fn kill(self) -> Result<(), String>;
}

impl ManagedChild for CommandChild {
    fn pid(&self) -> u32 {
        self.pid()
    }

    fn kill(self) -> Result<(), String> {
        self.kill().map_err(|error| error.to_string())
    }
}

pub(crate) struct ProcessSlot<T> {
    child: Option<T>,
}

impl<T> Default for ProcessSlot<T> {
    fn default() -> Self {
        Self { child: None }
    }
}

impl<T: ManagedChild> ProcessSlot<T> {
    pub(crate) fn replace(&mut self, child: T) -> Result<(), String> {
        if let Some(previous) = self.child.take() {
            previous.kill()?;
        }
        self.child = Some(child);
        Ok(())
    }

    pub(crate) fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(ManagedChild::pid)
    }

    pub(crate) fn clear_if_pid(&mut self, pid: u32) -> bool {
        if self.pid() == Some(pid) {
            self.child.take();
            true
        } else {
            false
        }
    }

    pub(crate) fn stop(&mut self) -> Result<(), String> {
        if let Some(child) = self.child.take() {
            child.kill()?;
        }
        Ok(())
    }
}

pub struct MpvProcessState(Arc<Mutex<ProcessSlot<CommandChild>>>);

impl Default for MpvProcessState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(ProcessSlot::default())))
    }
}

impl MpvProcessState {
    pub fn stop(&self) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|_| "mpv process state is unavailable".to_string())?
            .stop()
    }
}

#[tauri::command]
pub async fn cmd_play_video_in_mpv(
    url: String,
    filename: String,
    app_handle: AppHandle,
    state: State<'_, MpvProcessState>,
) -> Result<(), String> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (url, filename, app_handle, state);
        return Err("mpv playback is only supported on Windows".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let args = build_mpv_args(&url, &filename)?;

        state
            .0
            .lock()
            .map_err(|_| "mpv process state is unavailable".to_string())?
            .stop()?;

        let command = app_handle
            .shell()
            .sidecar("binaries/mpv")
            .map_err(|error| format!("Bundled mpv is unavailable: {error}"))?
            .args(args);
        let (mut events, child) = command
            .spawn()
            .map_err(|error| format!("Failed to start mpv: {error}"))?;
        let pid = child.pid();

        state
            .0
            .lock()
            .map_err(|_| "mpv process state is unavailable".to_string())?
            .replace(child)?;

        let process_slot = state.0.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = events.recv().await {
                if matches!(event, CommandEvent::Terminated(_)) {
                    if let Ok(mut slot) = process_slot.lock() {
                        slot.clear_if_pid(pid);
                    }
                    break;
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{build_mpv_args, ManagedChild, ProcessSlot, validate_stream_url};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    const LOCAL_STREAM: &str =
        "http://127.0.0.1:14201/stream/home/42?token=0123456789abcdef";

    #[test]
    fn accepts_the_original_local_stream_endpoint() {
        let parsed = validate_stream_url(LOCAL_STREAM).expect("local stream URL should be valid");

        assert_eq!(parsed.host_str(), Some("127.0.0.1"));
        assert_eq!(parsed.port(), Some(crate::STREAM_PORT));
        assert!(parsed.path().starts_with("/stream/"));
    }

    #[test]
    fn rejects_non_local_or_non_stream_urls() {
        assert!(validate_stream_url("https://example.com/stream/home/42?token=x").is_err());
        assert!(validate_stream_url("http://127.0.0.1:14201/api/files?token=x").is_err());
        assert!(validate_stream_url("http://127.0.0.1:9999/stream/home/42?token=x").is_err());
    }

    #[test]
    fn builds_deterministic_mpv_arguments_with_embedded_subtitles_enabled() {
        let args = build_mpv_args(LOCAL_STREAM, "Episode 01.mkv")
            .expect("local stream should produce mpv arguments");

        assert!(args.iter().any(|arg| arg == "--hwdec=auto-safe"));
        assert!(args.iter().any(|arg| arg == "--osc=yes"));
        assert!(args.iter().any(|arg| arg == "--sid=auto"));
        assert!(args.iter().any(|arg| arg == "--title=Episode 01.mkv"));
        assert_eq!(args[args.len() - 2], "--");
        assert_eq!(args.last().map(String::as_str), Some(LOCAL_STREAM));
    }

    struct FakeChild {
        pid: u32,
        killed: Arc<AtomicBool>,
    }

    impl ManagedChild for FakeChild {
        fn pid(&self) -> u32 {
            self.pid
        }

        fn kill(self) -> Result<(), String> {
            self.killed.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn replacing_a_process_kills_the_previous_child() {
        let killed = Arc::new(AtomicBool::new(false));
        let mut slot = ProcessSlot::default();
        slot.replace(FakeChild { pid: 10, killed: killed.clone() })
            .expect("first child should be stored");

        slot.replace(FakeChild { pid: 11, killed: Arc::new(AtomicBool::new(false)) })
            .expect("replacement child should be stored");

        assert!(killed.load(Ordering::SeqCst));
        assert_eq!(slot.pid(), Some(11));
    }

    #[test]
    fn stale_termination_does_not_clear_the_current_child() {
        let mut slot = ProcessSlot::default();
        slot.replace(FakeChild { pid: 11, killed: Arc::new(AtomicBool::new(false)) })
            .expect("child should be stored");

        assert!(!slot.clear_if_pid(10));
        assert_eq!(slot.pid(), Some(11));
        assert!(slot.clear_if_pid(11));
        assert_eq!(slot.pid(), None);
    }

    #[test]
    fn stopping_the_slot_kills_and_clears_the_current_child() {
        let killed = Arc::new(AtomicBool::new(false));
        let mut slot = ProcessSlot::default();
        slot.replace(FakeChild { pid: 12, killed: killed.clone() })
            .expect("child should be stored");

        slot.stop().expect("current child should stop");

        assert!(killed.load(Ordering::SeqCst));
        assert_eq!(slot.pid(), None);
    }
}
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, State};
use tauri_plugin_shell::{
    ShellExt,
    process::{CommandChild, CommandEvent},
};
