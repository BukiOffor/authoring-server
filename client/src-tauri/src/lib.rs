#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{process, thread};
use tauri::Manager;
use tauri_plugin_log::RotationStrategy;
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};

struct AppState {
    child_process: Arc<Mutex<Option<CommandChild>>>,
}

pub fn run() {
    let child_process: Arc<Mutex<Option<CommandChild>>> = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .manage(AppState { child_process })
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .rotation_strategy(RotationStrategy::KeepAll)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            let sidecar = app
                .shell()
                .sidecar("tool_exec")
                .expect("Failed to create sidecar");
            let result = sidecar.spawn();

            match result {
                Ok((mut rx, child)) => {
                    {
                        let app_state = app.state::<AppState>();
                        let mut child_lock = app_state.child_process.lock().unwrap();
                        *child_lock = Some(child);
                    }
                    tauri::async_runtime::spawn(async move {
                        while let Some(event) = rx.recv().await {
                            match event {
                                CommandEvent::Stdout(line) => {
                                    log::info!(
                                        "[Sidecar stdout] {}",
                                        String::from_utf8_lossy(&line)
                                    );
                                }
                                CommandEvent::Stderr(line) => {
                                    log::error!(
                                        "[Sidecar stderr] {}",
                                        String::from_utf8_lossy(&line)
                                    );
                                }
                                CommandEvent::Error(err) => {
                                    let error_message = format!("[Sidecar error] {}", err);
                                    log::error!("{}", &error_message);
                                }
                                CommandEvent::Terminated(_) => {
                                    log::error!("[Sidecar] Terminated.");
                                    process::exit(1); // Exit on error
                                }
                                _ => {}
                            }
                        }
                    });
                }
                Err(err) => {
                    log::error!("[Sidecar] Failed to spawn: {}", err);
                    app.handle().exit(1);
                }
            }

            Ok(())
        })
        .on_window_event({
            move |window, event| {
                let child_process = window.state::<AppState>().child_process.clone();
                let app = window.app_handle().clone();
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close(); // Prevent default to allow cleanup
                    prompt_user_and_kill(app, child_process);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                
            }
        });
}



pub fn prompt_user_and_kill(
    app: tauri::AppHandle,
    child_process: Arc<Mutex<Option<CommandChild>>>,
) {
    let kill = |app: tauri::AppHandle, child_process: Arc<Mutex<Option<CommandChild>>>| -> () {
        log::info!("Close was accepted");
        thread::sleep(Duration::from_millis(500));
        // Kill sidecar if still running
        let mut child_lock = child_process.lock().unwrap();
        if let Some(child) = child_lock.take() {
            child.kill().unwrap_or_else(|err| {
                log::error!("❌❌❌ Failed to kill sidecar: {}", err);
            });
            log::info!("🛑 Sidecar process killed.");
        }
        app.exit(0);
    };

kill(app, child_process);
}