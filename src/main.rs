// Prevents console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, RunEvent,
};
use tauri_plugin_autostart::MacosLauncher;

mod api_client;
mod commands;
mod config;
mod polling;
mod printer;

fn main() {
    let connected = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));

    let connected_clone = connected.clone();
    let stop_clone = stop.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::list_printers,
            commands::test_connection,
        ])
        .setup(move |app| {
            // System Tray aufbauen
            let settings_item = MenuItemBuilder::with_id("settings", "Einstellungen...")
                .build(app)?;
            let quit_item =
                MenuItemBuilder::with_id("quit", "Beenden").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("AMPARIO Print Agent")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "settings" => {
                        eprintln!("Menu: Einstellungen clicked");
                        if let Some(win) = app.get_webview_window("settings") {
                            eprintln!("  Window found, showing...");
                            let _ = win.show();
                            let _ = win.set_focus();
                        } else {
                            eprintln!("  Window NOT found!");
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // macOS: Linksklick öffnet Menü, aber DoubleClick öffnet Fenster
                    if matches!(event, tauri::tray::TrayIconEvent::DoubleClick { .. }) {
                        eprintln!("Tray double-click!");
                        if let Some(win) = tray.app_handle().get_webview_window("settings") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Polling starten
            polling::start_polling(connected_clone, stop_clone);

            // Settings-Fenster beim ersten Start anzeigen
            if let Some(win) = app.get_webview_window("settings") {
                eprintln!("Settings window found, showing...");
                let _ = win.show();
                let _ = win.set_focus();

                // Beim Schließen nur verstecken
                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            } else {
                eprintln!("WARNING: Settings window not found!");
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Fehler beim Erstellen der App")
        .run(|_app, event| {
            // App am Leben halten wenn alle Fenster geschlossen sind
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
