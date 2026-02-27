use crate::api_client::ApiClient;
use crate::config::load_config;
use crate::printer::print_pdf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Status des Polling-Loops.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Connected,
    Printing(String),
    Disconnected(String),
    Unconfigured,
}

/// Startet den Polling-Loop in einem eigenen Thread.
pub fn start_polling(
    status_flag: Arc<AtomicBool>,  // true = verbunden
    stop_flag: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut backoff = 5u64;

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let cfg = load_config();

            if cfg.api_key.is_empty() || cfg.server_url.is_empty() {
                status_flag.store(false, Ordering::Relaxed);
                thread::sleep(Duration::from_secs(cfg.poll_interval_seconds.max(5)));
                continue;
            }

            let client = ApiClient::new(&cfg.server_url, &cfg.api_key);

            match client.fetch_queue() {
                Ok(jobs) => {
                    status_flag.store(true, Ordering::Relaxed);
                    backoff = cfg.poll_interval_seconds.max(5);

                    for job in jobs {
                        // Job beanspruchen
                        match client.claim_job(&job.queue_id, &cfg.agent_id) {
                            Ok(claim) => {
                                if !claim.ok {
                                    continue;
                                }
                                // PDF herunterladen
                                match client.download_pdf(&job.queue_id) {
                                    Ok(pdf_path) => {
                                        // Label-Format bestimmen (Default: 103x199mm)
                                        let (w, h) = get_label_size(
                                            &claim.label_format.unwrap_or_default(),
                                        );
                                        let printer = if cfg.printer_name.is_empty() {
                                            // Fallback: ersten verfügbaren Drucker nehmen
                                            crate::printer::list_local_printers()
                                                .first()
                                                .cloned()
                                                .unwrap_or_default()
                                        } else {
                                            cfg.printer_name.clone()
                                        };

                                        if printer.is_empty() {
                                            let _ = client.report_done(
                                                &job.queue_id,
                                                false,
                                                Some("Kein Drucker konfiguriert".into()),
                                            );
                                            continue;
                                        }

                                        // Drucken
                                        match print_pdf(&pdf_path, &printer, w, h) {
                                            Ok(()) => {
                                                let _ = client.report_done(
                                                    &job.queue_id,
                                                    true,
                                                    None,
                                                );
                                                println!(
                                                    "  Gedruckt: {} auf {}",
                                                    job.queue_id, printer
                                                );
                                            }
                                            Err(e) => {
                                                let _ = client.report_done(
                                                    &job.queue_id,
                                                    false,
                                                    Some(e.clone()),
                                                );
                                                eprintln!(
                                                    "  Druckfehler {}: {}",
                                                    job.queue_id, e
                                                );
                                            }
                                        }

                                        // Temp-Datei aufräumen
                                        let _ = std::fs::remove_file(&pdf_path);
                                    }
                                    Err(e) => {
                                        let _ = client.report_done(
                                            &job.queue_id,
                                            false,
                                            Some(format!("Download: {}", e)),
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("  Claim-Fehler {}: {}", job.queue_id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    status_flag.store(false, Ordering::Relaxed);
                    eprintln!("  Polling-Fehler: {} (retry in {}s)", e, backoff);
                    // Exponential backoff
                    backoff = (backoff * 2).min(60);
                }
            }

            thread::sleep(Duration::from_secs(backoff));
        }
    })
}

/// Label-Größe aus Format-String bestimmen (Default: 103x199mm).
fn get_label_size(format: &str) -> (u32, u32) {
    // Alle unterstützten Formate verwenden derzeit 103x199mm
    match format {
        _ => (103, 199),
    }
}
