use tauri::State;
use tauri::Manager;
use grammers_client::Client;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use grammers_mtsender::SenderPool;
use grammers_session::storages::SqliteSession;
use tokio::sync::oneshot;
use tokio::time::Duration;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use grammers_tl_types as tl;

use crate::TelegramState;
use crate::models::{AuthResult};
use crate::commands::utils::map_error;
use grammers_client::SignInError;

/// Ensures the Telegram client is initialized.
/// 
/// IMPORTANT: This function properly manages runner lifecycle to prevent stack overflow.
/// Before spawning a new runner, it signals the old runner to shutdown.
pub async fn ensure_client_initialized(
    app_handle: &tauri::AppHandle,
    state: &State<'_, TelegramState>,
    api_id: i32,
) -> Result<Client, String> {
    let mut client_guard = state.client.lock().await;

    if let Some(client) = client_guard.as_ref() {
        return Ok(client.clone());
    }

    // CRITICAL: Shutdown existing runner before creating a new one
    // This prevents runner task accumulation which causes stack overflow
    let did_shutdown_old_runner = {
        let mut guard = state.runner_shutdown.lock().unwrap();
        if let Some(shutdown_tx) = guard.take() {
            log::info!("Signaling old runner to shutdown...");
            let _ = shutdown_tx.send(());
            true
        } else {
            false
        }
    }; // MutexGuard dropped here — before the await
    if did_shutdown_old_runner {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let runner_num = state.runner_count.fetch_add(1, Ordering::SeqCst) + 1;
    log::info!("Initializing Telegram Client #{} with API ID: {}", runner_num, api_id);
    
    // Resolve session path safely
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }
    
    let session_path = app_data_dir.join("telegram.session");
    let session_path_str = session_path.to_string_lossy().to_string();
    log::info!("Opening session at: {}", session_path_str);
    
    let mut session_open_result = SqliteSession::open(&session_path_str).await;
    
    // Retry opening the session database up to 5 times (every 100ms)
    // in case the database is temporarily locked by the old shutting down runner.
    if session_open_result.is_err() {
        for attempt in 1..=5 {
            log::warn!("Failed to open session on attempt {} (database may be locked). Retrying in 100ms...", attempt);
            tokio::time::sleep(Duration::from_millis(100)).await;
            session_open_result = SqliteSession::open(&session_path_str).await;
            if session_open_result.is_ok() {
                break;
            }
        }
    }

    let session = match session_open_result.map_err(|e| e.to_string()) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Session file could not be opened after retries ({}). Recreating...", e);
            let _ = std::fs::remove_file(&session_path);
            let _ = std::fs::remove_file(format!("{}-wal", session_path_str));
            let _ = std::fs::remove_file(format!("{}-shm", session_path_str));
            
            SqliteSession::open(&session_path_str).await
                .map_err(|err| format!("Failed to open session after recreation: {}", err))?
        }
    };
        
    let connection_params = grammers_mtsender::ConnectionParams::default();

    let session = Arc::new(session);
    let pool = SenderPool::with_configuration(session, api_id, connection_params);
    let client = Client::new(pool.handle);
    
    // Create shutdown channel for this runner
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    *state.runner_shutdown.lock().unwrap() = Some(shutdown_tx);
    
    // Spawn the network runner with shutdown support
    let SenderPool { runner, .. } = pool;
    tauri::async_runtime::spawn(async move {
        tokio::select! {
            // Normal runner operation
            _ = runner.run() => {
                log::info!("Runner #{} exited normally", runner_num);
            }
            // Shutdown requested
            _ = shutdown_rx => {
                log::info!("Runner #{} shutdown requested, exiting", runner_num);
            }
        }
    });
    
    *client_guard = Some(client.clone());
    Ok(client)
}

#[tauri::command]
pub async fn cmd_connect(
    app_handle: tauri::AppHandle,
    state: State<'_, TelegramState>,
    api_id: i32,
) -> Result<bool, String> {
    // Store API ID for auto-reconnect
    *state.api_id.lock().await = Some(api_id);
    ensure_client_initialized(&app_handle, &state, api_id).await?;
    Ok(true)
}

#[tauri::command]
pub async fn cmd_check_connection(
    app_handle: tauri::AppHandle,
    state: State<'_, TelegramState>,
) -> Result<bool, String> {
    let client_msg_opt = {
        let guard = state.client.lock().await;
        guard.as_ref().cloned()
    };

    if let Some(client) = client_msg_opt {
        // 1. Check local authorization state first (instant, offline-friendly)
        match client.is_authorized().await {
            Ok(false) => {
                log::info!("Connection check: Session is not authorized. Clearing stale client...");
                *state.client.lock().await = None;
                return Ok(false);
            }
            Err(e) => {
                log::warn!("Connection check: is_authorized error: {}", e);
                *state.client.lock().await = None;
            }
            Ok(true) => {
                // 2. Verified authorized locally, now verify network ping with a 5s timeout
                log::info!("Connection check: Session is authorized locally. Pinging Telegram (5s timeout)...");
                match tokio::time::timeout(Duration::from_secs(5), client.get_me()).await {
                    Ok(Ok(_me)) => {
                        log::info!("Connection check: Telegram ping OK.");
                        return Ok(true);
                    }
                    Ok(Err(e)) => {
                        log::warn!("Connection check: get_me failed ({}). Session may be revoked.", e);
                        *state.client.lock().await = None;
                        return Ok(false);
                    }
                    Err(_) => {
                        log::warn!("Connection check: get_me timed out after 5 seconds.");
                        // Allow user into app if authorized locally even if network ping timed out (supports offline mode)
                        return Ok(true);
                    }
                }
            }
        }
    }

    // 3. Fallback: If no client or is_authorized was inconclusive, try reconnect with saved API ID
    let api_id_opt = *state.api_id.lock().await;
    if let Some(api_id) = api_id_opt {
        log::info!("Connection check: Attempting client re-initialization with saved API ID...");
        *state.client.lock().await = None;
        
        match tokio::time::timeout(Duration::from_secs(8), ensure_client_initialized(&app_handle, &state, api_id)).await {
            Ok(Ok(c)) => {
                if let Ok(true) = c.is_authorized().await {
                    log::info!("Auto-reconnect successful.");
                    return Ok(true);
                }
            }
            Ok(Err(e)) => log::warn!("Auto-reconnect failed: {}", e),
            Err(_) => log::warn!("Auto-reconnect timed out after 8s"),
        }
    }

    *state.client.lock().await = None;
    Ok(false) // Not connected and no credentials to reconnect
}

#[tauri::command]
pub async fn cmd_reconnect_with_network_settings(
    app_handle: tauri::AppHandle,
    state: State<'_, TelegramState>,
) -> Result<bool, String> {
    let api_id = *state.api_id.lock().await;
    let api_id = match api_id {
        Some(id) => id,
        None => return Err("Not authenticated — no API ID saved.".into()),
    };

    log::info!("Reconnecting with updated network settings...");

    // 1. Shutdown existing runner
    {
        let mut shutdown_guard = state.runner_shutdown.lock().unwrap();
        if let Some(shutdown_tx) = shutdown_guard.take() {
            log::info!("Signaling runner shutdown for reconnect...");
            let _ = shutdown_tx.send(());
        }
    }
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 2. Clear old client
    *state.client.lock().await = None;

    // 3. Reinitialize with the fixed direct-transfer policy.
    let client = ensure_client_initialized(&app_handle, &state, api_id).await?;

    // 4. Verify the new connection works with 5s timeout
    match tokio::time::timeout(Duration::from_secs(5), client.get_me()).await {
        Ok(Ok(_me)) => {
            log::info!("Reconnect successful — verified via get_me().");
            Ok(true)
        }
        Ok(Err(e)) => {
            log::error!("Reconnect init succeeded but get_me failed: {}", e);
            Err(format!("Reconnected but ping failed: {}", e))
        }
        Err(_) => {
            log::warn!("Reconnect get_me timed out after 5 seconds.");
            Ok(true)
        }
    }
}

#[tauri::command]
pub async fn cmd_logout(
    app_handle: tauri::AppHandle,
    state: State<'_, TelegramState>,
) -> Result<bool, String> {
    log::info!("Logging out...");
    
    // 1. Shutdown the network runner FIRST to prevent any operations
    {
        let mut shutdown_guard = state.runner_shutdown.lock().unwrap();
        if let Some(shutdown_tx) = shutdown_guard.take() {
            log::info!("Signaling runner shutdown for logout...");
            let _ = shutdown_tx.send(());
        }
    }
    
    // 2. Try to sign out from Telegram (if connected)
    let client_opt = { state.client.lock().await.clone() };
    if let Some(client) = client_opt {
        // We don't strictly care if this fails (e.g. network down), we just want to clear local state.
        let _ = client.sign_out().await; 
    }

    // 3. Clear State
    *state.client.lock().await = None;
    *state.login_token.lock().await = None;
    *state.password_token.lock().await = None;
    *state.api_id.lock().await = None;
    crate::commands::utils::clear_peer_cache(&state.peer_cache).await;
    state.cancelled_transfers.write().await.clear();

    // 4. Remove Session File
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let session_path = app_data_dir.join("telegram.session");
    let _ = std::fs::remove_file(session_path);
    let _ = std::fs::remove_file(app_data_dir.join("telegram.session-wal"));
    let _ = std::fs::remove_file(app_data_dir.join("telegram.session-shm"));

    log::info!("Logout complete. Runner count: {}", state.runner_count.load(Ordering::SeqCst));
    Ok(true)
}

#[tauri::command]
pub async fn cmd_auth_request_code(
    app_handle: tauri::AppHandle,
    phone: String,
    api_id: i32,
    api_hash: String,
    state: State<'_, TelegramState>,
) -> Result<String, String> {
    if api_hash.trim().is_empty() {
        return Err("API Hash cannot be empty.".to_string());
    }

    let phone_clean = phone.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect::<String>();
    if phone_clean.is_empty() {
        return Err("Phone number cannot be empty.".to_string());
    }

    // Store API ID
    *state.api_id.lock().await = Some(api_id);

    // Always clear old/stale client and runner before starting a fresh login
    {
        let mut client_guard = state.client.lock().await;
        if let Some(_existing) = client_guard.take() {
            log::info!("Clearing old client instance for fresh login request...");
            let mut shutdown_guard = state.runner_shutdown.lock().unwrap();
            if let Some(tx) = shutdown_guard.take() {
                let _ = tx.send(());
            }
        }
    }
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Reset unauthorized session file if present to guarantee a fresh login state
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let session_path = app_data_dir.join("telegram.session");
    if session_path.exists() {
        if let Ok(sess) = SqliteSession::open(&session_path).await {
            let sess_arc = Arc::new(sess);
            let pool = SenderPool::with_configuration(sess_arc, api_id, grammers_mtsender::ConnectionParams::default());
            let test_client = Client::new(pool.handle);
            if let Ok(false) | Err(_) = test_client.is_authorized().await {
                log::info!("Removing unauthorized old session file for clean login...");
                let _ = std::fs::remove_file(&session_path);
                let _ = std::fs::remove_file(app_data_dir.join("telegram.session-wal"));
                let _ = std::fs::remove_file(app_data_dir.join("telegram.session-shm"));
            }
        }
    }

    let client_handle = ensure_client_initialized(&app_handle, &state, api_id).await?;
    
    log::info!("Requesting code for {}", phone_clean);
    
    let mut last_error = String::new();
    
    // Retry up to 2 times for AUTH_RESTART or 500 with a 15s timeout
    for i in 1..=2 {
        let req_fut = client_handle.request_login_code(&phone_clean, &api_hash);
        match tokio::time::timeout(Duration::from_secs(15), req_fut).await {
            Ok(Ok(token)) => {
                let mut token_guard = state.login_token.lock().await;
                *token_guard = Some(token);
                return Ok("code_sent".to_string());
            },
            Ok(Err(e)) => {
                let err_msg = e.to_string();
                log::warn!("Error requesting code (Attempt {}): {}", i, err_msg);
                
                if err_msg.contains("AUTH_RESTART") || err_msg.contains("500") {
                    log::info!("AUTH_RESTART error detected. Retrying...");
                    last_error = err_msg;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
                
                return Err(map_error(e));
            }
            Err(_) => {
                log::warn!("request_login_code timed out after 15 seconds (Attempt {})", i);
                last_error = "Connection to Telegram timed out. Please check your internet connection and API credentials.".to_string();
            }
        }
    }

    Err(format!("Telegram Error: {}", last_error))
}

#[tauri::command]
pub async fn cmd_auth_sign_in(
    code: String,
    state: State<'_, TelegramState>,
) -> Result<AuthResult, String> {
    log::info!("Signing in with code...");
    
    let client = {
        let guard = state.client.lock().await;
        guard.as_ref().ok_or("Client not initialized. Please go back to phone step.")?.clone()
    };

    let token_guard = state.login_token.lock().await;
    let login_token = token_guard.as_ref().ok_or("No login session found (restart flow)")?;

    let code_clean = code.chars().filter(|c| c.is_ascii_digit()).collect::<String>();

    match tokio::time::timeout(Duration::from_secs(15), client.sign_in(login_token, &code_clean)).await {
        Ok(Ok(_user)) => {
             log::info!("Successfully logged in.");
             Ok(AuthResult {
                success: true,
                next_step: Some("dashboard".to_string()),
                error: None,
            })
        }
        Ok(Err(SignInError::PasswordRequired(token))) => {
            let mut pw_guard = state.password_token.lock().await;
            *pw_guard = Some(token);

            Ok(AuthResult {
                success: false,
                next_step: Some("password".to_string()),
                error: None,
            })
        }
        Ok(Err(e)) => {
           log::error!("Sign in error: {}", e);
           Err(format!("Sign in failed: {}", e))
        }
        Err(_) => {
           log::error!("Sign in timed out after 15s.");
           Err("Sign in timed out. Please check your network connection and try again.".to_string())
        }
    }
}

#[tauri::command]
pub async fn cmd_auth_check_password(
    password: String,
    state: State<'_, TelegramState>,
) -> Result<AuthResult, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.as_ref().ok_or("Client not initialized")?.clone()
    };
    
    let mut pw_guard = state.password_token.lock().await;
    let pw_token = pw_guard.take().ok_or("No password session found")?;

    match tokio::time::timeout(Duration::from_secs(15), client.check_password(pw_token, password.as_str())).await {
        Ok(Ok(_user)) => {
             log::info!("2FA Success.");
             Ok(AuthResult {
                success: true,
                next_step: Some("dashboard".to_string()),
                error: None,
            })
        }
        Ok(Err(e)) => Err(format!("2FA Failed: {}", e)),
        Err(_) => Err("2FA check timed out after 15 seconds.".to_string()),
    }
}

/// QR Login -- Step 1: Export a login token and return the `tg://login?token=...` URL.
#[tauri::command]
pub async fn cmd_auth_qr_login(
    app_handle: tauri::AppHandle,
    api_id: i32,
    api_hash: String,
    state: State<'_, TelegramState>,
) -> Result<String, String> {
    if api_hash.trim().is_empty() {
        return Err("API Hash cannot be empty.".to_string());
    }

    *state.api_id.lock().await = Some(api_id);

    // Clear old client for fresh QR login
    {
        let mut client_guard = state.client.lock().await;
        if let Some(_existing) = client_guard.take() {
            log::info!("Clearing old client instance for fresh QR login...");
            let mut shutdown_guard = state.runner_shutdown.lock().unwrap();
            if let Some(tx) = shutdown_guard.take() {
                let _ = tx.send(());
            }
        }
    }
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = ensure_client_initialized(&app_handle, &state, api_id).await?;

    log::info!("Requesting QR login token...");

    let req = tl::functions::auth::ExportLoginToken {
        api_id,
        api_hash: api_hash.clone(),
        except_ids: vec![],
    };

    let result = match tokio::time::timeout(Duration::from_secs(15), client.invoke(&req)).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Err(format!("ExportLoginToken failed: {}", e)),
        Err(_) => return Err("Requesting QR code timed out. Please check your network connection.".to_string()),
    };

    match result {
        tl::enums::auth::LoginToken::Token(t) => {
            let encoded = URL_SAFE_NO_PAD.encode(&t.token);
            let url = format!("tg://login?token={}", encoded);
            log::info!("QR login URL generated, expires at {}", t.expires);
            Ok(url)
        }
        tl::enums::auth::LoginToken::Success(_s) => {
            log::info!("QR login: already authorized");
            Ok("__authorized__".to_string())
        }
        tl::enums::auth::LoginToken::MigrateTo(m) => {
            log::info!("QR login: need to migrate to DC {}", m.dc_id);
            let encoded = URL_SAFE_NO_PAD.encode(&m.token);
            let url = format!("tg://login?token={}", encoded);
            Ok(url)
        }
    }
}

/// QR Login -- Step 2: Poll for scan completion.
#[tauri::command]
pub async fn cmd_auth_qr_poll(
    state: State<'_, TelegramState>,
) -> Result<AuthResult, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.as_ref().ok_or("Client not initialized")?.clone()
    };

    match client.is_authorized().await {
        Ok(true) => {
            log::info!("QR login: session authorized!");
            Ok(AuthResult {
                success: true,
                next_step: Some("dashboard".to_string()),
                error: None,
            })
        }
        Ok(false) => {
            Ok(AuthResult {
                success: false,
                next_step: Some("waiting".to_string()),
                error: None,
            })
        }
        Err(e) => {
            log::warn!("QR poll auth check failed: {}", e);
            Ok(AuthResult {
                success: false,
                next_step: Some("waiting".to_string()),
                error: None,
            })
        }
    }
}

