use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WindowEvent,
};

/// User fitness summary returned to the frontend via IPC
#[derive(Clone, Serialize)]
struct FitnessSummary {
    calories_logged: u32,
    workouts_today: u32,
    steps: u32,
    water_ml: u32,
    fitall_score: f64,
}

/// System health summary
#[derive(Clone, Serialize)]
struct HealthSummary {
    api_up: bool,
    db_connected: bool,
    version: String,
}

/// IPC command: check API health
#[tauri::command]
async fn check_health() -> Result<HealthSummary, String> {
    let client = reqwest::Client::new();
    match client
        .get("https://fitall.auraalpha.cc/api/health")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let data: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Parse error: {}", e))?;

            Ok(HealthSummary {
                api_up: true,
                db_connected: data
                    .get("db")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "ok")
                    .unwrap_or(false),
                version: data
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            })
        }
        Ok(resp) => Err(format!("API returned status {}", resp.status())),
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}

/// IPC command: get today's fitness summary from dashboard endpoint
#[tauri::command]
async fn get_fitness_summary() -> Result<FitnessSummary, String> {
    // This returns a default summary; the real data comes from the web frontend
    // once it loads from fitall.auraalpha.cc
    Ok(FitnessSummary {
        calories_logged: 0,
        workouts_today: 0,
        steps: 0,
        water_ml: 0,
        fitall_score: 0.0,
    })
}

/// IPC command: send a native desktop notification
#[tauri::command]
async fn send_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| format!("Notification error: {}", e))
}

/// IPC command: navigate main window to a URL
#[tauri::command]
async fn navigate_to(app: tauri::AppHandle, url: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let parsed = url
            .parse::<tauri::Url>()
            .map_err(|e| e.to_string())?;
        window
            .navigate(parsed)
            .map_err(|e| format!("Navigation error: {}", e))
    } else {
        Err("Main window not found".to_string())
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_health,
            get_fitness_summary,
            send_notification,
            navigate_to,
        ])
        .setup(|app| {
            // ── System tray ──────────────────────────────────────
            let show = MenuItem::with_id(app, "show", "Open FitAll", true, None::<&str>)?;
            let dashboard =
                MenuItem::with_id(app, "dashboard", "Dashboard", true, None::<&str>)?;
            let quick_log =
                MenuItem::with_id(app, "quick_log", "Quick Log", true, None::<&str>)?;
            let workouts =
                MenuItem::with_id(app, "workouts", "Workouts", true, None::<&str>)?;
            let nutrition =
                MenuItem::with_id(app, "nutrition", "Nutrition", true, None::<&str>)?;
            let health =
                MenuItem::with_id(app, "health", "Check Health", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &show,
                    &dashboard,
                    &quick_log,
                    &workouts,
                    &nutrition,
                    &health,
                    &quit,
                ],
            )?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("FitAll — Fitness Platform")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "dashboard" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let url: tauri::Url =
                                "https://fitall.auraalpha.cc/dashboard".parse().unwrap();
                            let _ = window.navigate(url);
                        }
                    }
                    "quick_log" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let url: tauri::Url =
                                "https://fitall.auraalpha.cc/log".parse().unwrap();
                            let _ = window.navigate(url);
                        }
                    }
                    "workouts" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let url: tauri::Url =
                                "https://fitall.auraalpha.cc/workouts".parse().unwrap();
                            let _ = window.navigate(url);
                        }
                    }
                    "nutrition" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let url: tauri::Url =
                                "https://fitall.auraalpha.cc/nutrition".parse().unwrap();
                            let _ = window.navigate(url);
                        }
                    }
                    "health" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            match check_health().await {
                                Ok(h) => {
                                    let status = if h.api_up { "Healthy" } else { "Down" };
                                    let db = if h.db_connected { "OK" } else { "Error" };
                                    let _ = send_notification(
                                        app,
                                        "FitAll Health".to_string(),
                                        format!(
                                            "API: {} | DB: {} | v{}",
                                            status, db, h.version
                                        ),
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    let _ = send_notification(
                                        app,
                                        "FitAll Health".to_string(),
                                        format!("Health check failed: {}", e),
                                    )
                                    .await;
                                }
                            }
                        });
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Navigate to fitall.auraalpha.cc on startup ──────────
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Brief delay to let the splash/loading screen render
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                let client = reqwest::Client::new();
                let reachable = client
                    .get("https://fitall.auraalpha.cc/api/health")
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false);

                if reachable {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let url: tauri::Url =
                            "https://fitall.auraalpha.cc".parse().unwrap();
                        let _ = window.navigate(url);
                    }
                }
                // If not reachable, the local index.html stays visible with retry button
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            // Minimize to tray instead of closing
            RunEvent::WindowEvent {
                label,
                event: WindowEvent::CloseRequested { api, .. },
                ..
            } if label == "main" => {
                api.prevent_close();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            RunEvent::ExitRequested { api, .. } => {
                // Keep running in tray
                api.prevent_exit();
            }
            _ => {}
        });
}
