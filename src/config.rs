use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub server_url: String,
    pub api_key: String,
    pub printer_name: String,
    pub poll_interval_seconds: u64,
    pub auto_start: bool,
    pub agent_id: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:5050".into(),
            api_key: String::new(),
            printer_name: String::new(),
            poll_interval_seconds: 5,
            auto_start: true,
            agent_id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
        }
    }
}

fn config_path() -> PathBuf {
    let dir = config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.join("settings.json")
}

/// Plattformabhängiges Config-Verzeichnis:
/// - macOS:   ~/Library/Application Support/ampario-print-agent/
/// - Linux:   ~/.config/ampario-print-agent/
/// - Windows: %APPDATA%/ampario-print-agent/
fn config_dir() -> Option<PathBuf> {
    let base = if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").ok()?;
        PathBuf::from(home).join("Library").join("Application Support")
    } else if cfg!(target_os = "windows") {
        std::env::var("APPDATA").ok().map(PathBuf::from)?
    } else {
        // Linux / andere Unix-Systeme
        let home = std::env::var("HOME").ok()?;
        PathBuf::from(home).join(".config")
    };

    let dir = base.join("ampario-print-agent");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

pub fn load_config() -> AgentConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
    }
    AgentConfig::default()
}

pub fn save_config(cfg: &AgentConfig) -> Result<(), String> {
    let path = config_path();
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}
