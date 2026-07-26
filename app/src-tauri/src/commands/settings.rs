//! Windows application settings commands.

/// Set the application to run at Windows startup.
#[tauri::command]
pub fn cmd_set_autostart(enabled: bool) -> Result<String, String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe_path.to_string_lossy();
    if enabled {
        log::info!("Enabling Windows Autostart with path: {}", exe_str);
        let status = std::process::Command::new("reg")
            .args(&[
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "TeleStash",
                "/t",
                "REG_SZ",
                "/d",
                &exe_str,
                "/f",
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("Failed to write to Windows registry for autostart".to_string());
        }
    } else {
        log::info!("Disabling Windows Autostart");
        let _ = std::process::Command::new("reg")
            .args(&[
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "TeleStash",
                "/f",
            ])
            .status();
    }
    Ok("Autostart setting updated successfully".into())
}
