use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct QueueResponse {
    pub jobs: Vec<QueueItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueueItem {
    pub queue_id: String,
    pub job_id: String,
    pub label_format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClaimResponse {
    pub ok: bool,
    pub file_url: Option<String>,
    pub label_format: Option<String>,
}

#[derive(Debug, Serialize)]
struct ClaimBody {
    agent_id: String,
}

#[derive(Debug, Serialize)]
struct DoneBody {
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PrintConfig {
    pub label_templates: serde_json::Value,
    pub printer_name: String,
}

pub struct ApiClient {
    client: Client,
    pub server_url: String,
    pub api_key: String,
}

impl ApiClient {
    pub fn new(server_url: &str, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self {
            client,
            server_url: server_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Offene Druckaufträge abrufen.
    pub fn fetch_queue(&self) -> Result<Vec<QueueItem>, String> {
        let resp = self
            .client
            .get(format!("{}/api/print/queue", self.server_url))
            .header("X-Agent-API-Key", &self.api_key)
            .send()
            .map_err(|e| format!("Verbindung fehlgeschlagen: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Server-Fehler: {}", resp.status()));
        }

        let data: QueueResponse = resp.json().map_err(|e| e.to_string())?;
        Ok(data.jobs)
    }

    /// Job beanspruchen.
    pub fn claim_job(&self, queue_id: &str, agent_id: &str) -> Result<ClaimResponse, String> {
        let resp = self
            .client
            .post(format!("{}/api/print/claim/{}", self.server_url, queue_id))
            .header("X-Agent-API-Key", &self.api_key)
            .json(&ClaimBody {
                agent_id: agent_id.to_string(),
            })
            .send()
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Claim fehlgeschlagen: {}", resp.status()));
        }

        resp.json().map_err(|e| e.to_string())
    }

    /// PDF herunterladen und in temp-Datei speichern.
    pub fn download_pdf(&self, queue_id: &str) -> Result<String, String> {
        let resp = self
            .client
            .get(format!("{}/api/print/file/{}", self.server_url, queue_id))
            .header("X-Agent-API-Key", &self.api_key)
            .send()
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Download fehlgeschlagen: {}", resp.status()));
        }

        let bytes = resp.bytes().map_err(|e| e.to_string())?;
        let tmp = tempfile::Builder::new()
            .suffix(".pdf")
            .tempfile()
            .map_err(|e| e.to_string())?;
        let path = tmp.path().to_string_lossy().to_string();
        std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
        // Keep file alive (don't drop tempfile)
        std::mem::forget(tmp);
        Ok(path)
    }

    /// Ergebnis melden.
    pub fn report_done(
        &self,
        queue_id: &str,
        success: bool,
        error: Option<String>,
    ) -> Result<(), String> {
        let resp = self
            .client
            .post(format!("{}/api/print/done/{}", self.server_url, queue_id))
            .header("X-Agent-API-Key", &self.api_key)
            .json(&DoneBody { success, error })
            .send()
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Report fehlgeschlagen: {}", resp.status()));
        }
        Ok(())
    }

    /// Verbindungstest.
    pub fn health_check(&self) -> Result<bool, String> {
        let resp = self
            .client
            .get(format!("{}/api/print/config", self.server_url))
            .header("X-Agent-API-Key", &self.api_key)
            .send()
            .map_err(|e| format!("Verbindung fehlgeschlagen: {}", e))?;
        Ok(resp.status().is_success())
    }

    /// Server-Druckkonfiguration abrufen.
    pub fn fetch_print_config(&self) -> Result<PrintConfig, String> {
        let resp = self
            .client
            .get(format!("{}/api/print/config", self.server_url))
            .header("X-Agent-API-Key", &self.api_key)
            .send()
            .map_err(|e| e.to_string())?;
        resp.json().map_err(|e| e.to_string())
    }
}
