use std::net::TcpStream;
use std::time::Duration;

const DC_ADDRESSES: &[&str] = &[
    "149.154.167.50:443",
    "149.154.167.51:443",
    "149.154.175.50:443",
    "149.154.175.51:443",
];

#[tauri::command]
pub async fn cmd_is_network_available() -> Result<bool, String> {
    tokio::task::spawn_blocking(|| {
        let timeout = Duration::from_secs(5);
        for dc in DC_ADDRESSES {
            if let Ok(address) = dc.parse() {
                if TcpStream::connect_timeout(&address, timeout).is_ok() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    })
    .await
    .map_err(|e| e.to_string())?
}
