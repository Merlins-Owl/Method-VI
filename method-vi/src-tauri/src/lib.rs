mod database;
pub mod spine;
pub mod ledger;
pub mod context;
pub mod signals;
pub mod config;
pub mod api;
pub mod agents;
pub mod governance;
pub mod commands;
pub mod artifacts;

use std::sync::Mutex;
use tauri::Manager;
use commands::OrchestratorState;
use config::AppConfig;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize the database
            let app_handle = app.handle().clone();
            if let Err(e) = database::init_database(&app_handle) {
                eprintln!("Failed to initialize database: {}", e);
                return Err(e.into());
            }

            println!("Method-VI database initialized successfully");

            // Load configuration
            let config = AppConfig::load(&app_handle).map_err(|e| {
                eprintln!("Failed to load configuration: {}", e);
                e
            })?;

            println!("Configuration loaded successfully");

            // Initialize orchestrator state
            app.manage(OrchestratorState(Mutex::new(None)));
            app.manage(Mutex::new(config));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::start_step_0,
            commands::execute_step_1,
            commands::execute_step_2,
            commands::execute_step_3,
            commands::execute_step_4,
            commands::execute_step_5,
            commands::execute_step_6,
            commands::execute_step_6_5,
            commands::execute_closure,
            commands::export_markdown,
            commands::export_json,
            commands::approve_gate,
            commands::reject_gate,
            commands::handle_halt_decision,
            commands::submit_clarifications,
            commands::get_all_callouts,
            commands::get_pending_callouts,
            commands::get_callout_summary,
            commands::can_proceed,
            commands::acknowledge_callout,
            commands::acknowledge_all_callouts,
            commands::get_current_mode,
            commands::detect_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
