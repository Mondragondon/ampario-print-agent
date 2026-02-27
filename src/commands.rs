use crate::api_client::ApiClient;
use crate::config::{self, AgentConfig};
use crate::printer;

#[tauri::command]
pub fn get_settings() -> AgentConfig {
    config::load_config()
}

#[tauri::command]
pub fn save_settings(settings: AgentConfig) -> Result<String, String> {
    config::save_config(&settings)?;
    Ok("Gespeichert".into())
}

#[tauri::command]
pub fn list_printers() -> Result<Vec<String>, String> {
    let printers = printer::list_local_printers();
    if printers.is_empty() {
        eprintln!("list_printers: keine Drucker gefunden");
    } else {
        eprintln!("list_printers: {} Drucker gefunden", printers.len());
    }
    Ok(printers)
}

#[tauri::command]
pub fn test_connection(server_url: String, api_key: String) -> Result<bool, String> {
    let client = ApiClient::new(&server_url, &api_key);
    client.health_check()
}
