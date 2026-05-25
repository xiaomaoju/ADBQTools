#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod device_manager;
mod embedded;
mod installer;
mod logcat_engine;
mod package_parser;
mod unity_parser;
mod util;

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
            let mut resource_dir = app
                .path()
                .resource_dir()
                .expect("failed to get resource dir");
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            std::fs::create_dir_all(&data_dir).ok();

            if cfg!(debug_assertions) {
                // Dev mode: resource_dir points to target/debug/, fall back to src-tauri/resources/
                let dev_resources = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");
                if dev_resources.exists() {
                    resource_dir = dev_resources;
                }
            } else {
                // Release mode: Tauri bundles resources under Resources/resources/
                let release_resources = resource_dir.join("resources");
                if release_resources.exists() {
                    resource_dir = release_resources;
                }
            }

            let resources = embedded::EmbeddedResources::new(resource_dir.clone(), data_dir.clone());
            println!("[ADBQTools] resource_dir: {:?}", resource_dir);
            println!("[ADBQTools] adb_path: {:?}, exists: {}", resources.adb_path(), resources.adb_path().exists());
            resources.ensure_executable_permissions().ok();
            resources.ensure_jre_extracted().ok();

            let db_path = data_dir.join("logcat.db");
            let db = db::LogDatabase::open(&db_path).expect("failed to open database");

            let device_manager = device_manager::DeviceManager::new(resources.adb_path());

            let app_handle = app.handle().clone();
            let poll_devices = device_manager.devices();
            let poll_adb = resources.adb_path();
            tauri::async_runtime::spawn(async move {
                device_manager::start_polling_shared(poll_adb, poll_devices, app_handle).await;
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
            commands::restart_adb,
            commands::list_keystore_aliases,
            commands::install_apk,
            commands::install_aab,
            commands::parse_package,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
