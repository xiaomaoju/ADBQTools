#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod device_manager;
mod embedded;
mod installer;
mod logcat_engine;
mod unity_parser;

use commands::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let resource_dir = app
                .path()
                .resource_dir()
                .expect("failed to get resource dir");
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            std::fs::create_dir_all(&data_dir).ok();

            let resources = embedded::EmbeddedResources::new(resource_dir, data_dir.clone());
            resources.ensure_executable_permissions().ok();
            resources.ensure_jre_extracted().ok();

            let db_path = data_dir.join("logcat.db");
            let db = db::LogDatabase::open(&db_path).expect("failed to open database");

            let device_manager = device_manager::DeviceManager::new(resources.adb_path());

            let app_handle = app.handle().clone();
            let dm_for_poll = device_manager::DeviceManager::new(resources.adb_path());
            tauri::async_runtime::spawn(async move {
                dm_for_poll.start_polling(app_handle).await;
            });

            app.manage(AppState {
                device_manager,
                logcat_sessions: Arc::new(Mutex::new(HashMap::new())),
                db: Arc::new(Mutex::new(db)),
                resources,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_devices,
            commands::connect_wifi,
            commands::pair_device,
            commands::disconnect_device,
            commands::start_logcat,
            commands::stop_logcat,
            commands::pause_logcat,
            commands::resume_logcat,
            commands::clear_logcat,
            commands::query_log_history,
            commands::install_apk,
            commands::install_aab,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
