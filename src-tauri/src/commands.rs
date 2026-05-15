use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::device_manager::{Device, DeviceManager};
use crate::logcat_engine::LogcatSession;
use crate::db::LogDatabase;
use crate::embedded::EmbeddedResources;

pub struct AppState {
    pub device_manager: DeviceManager,
    pub logcat_sessions: Arc<Mutex<HashMap<String, LogcatSession>>>,
    pub db: Arc<Mutex<LogDatabase>>,
    pub resources: EmbeddedResources,
}

#[tauri::command]
pub async fn get_devices(state: State<'_, AppState>) -> Result<Vec<Device>, String> {
    let devices = state.device_manager.devices();
    let map = devices.lock().await;
    Ok(map.values().cloned().collect())
}

#[tauri::command]
pub async fn connect_wifi(state: State<'_, AppState>, addr: String) -> Result<String, String> {
    state.device_manager.connect_wifi(&addr).await
}

#[tauri::command]
pub async fn pair_device(
    state: State<'_, AppState>,
    addr: String,
    code: String,
) -> Result<String, String> {
    state.device_manager.pair_device(&addr, &code).await
}

#[tauri::command]
pub async fn disconnect_device(
    state: State<'_, AppState>,
    serial: String,
) -> Result<String, String> {
    state.device_manager.disconnect_device(&serial).await
}

#[tauri::command]
pub async fn start_logcat(
    state: State<'_, AppState>,
    serial: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let session = LogcatSession::new(serial.clone(), state.resources.adb_path());
    session.start(app).await;
    let mut sessions = state.logcat_sessions.lock().await;
    sessions.insert(serial, session);
    Ok(())
}

#[tauri::command]
pub async fn stop_logcat(state: State<'_, AppState>, serial: String) -> Result<(), String> {
    let mut sessions = state.logcat_sessions.lock().await;
    if let Some(session) = sessions.remove(&serial) {
        session.stop();
    }
    Ok(())
}

#[tauri::command]
pub async fn pause_logcat(state: State<'_, AppState>, serial: String) -> Result<(), String> {
    let sessions = state.logcat_sessions.lock().await;
    if let Some(session) = sessions.get(&serial) {
        session.pause();
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_logcat(
    state: State<'_, AppState>,
    serial: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let sessions = state.logcat_sessions.lock().await;
    if let Some(session) = sessions.get(&serial) {
        session.flush_buffer(&app).await;
        session.resume();
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_logcat(state: State<'_, AppState>, serial: String) -> Result<(), String> {
    let sessions = state.logcat_sessions.lock().await;
    if let Some(session) = sessions.get(&serial) {
        session.clear_logcat().await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn query_log_history(
    state: State<'_, AppState>,
    device: String,
    from: Option<String>,
    to: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<crate::logcat_engine::LogEntry>, String> {
    let db = state.db.lock().await;
    db.query_entries(
        &device,
        from.as_deref(),
        to.as_deref(),
        limit.unwrap_or(1000),
    )
}

#[tauri::command]
pub async fn install_apk(
    state: State<'_, AppState>,
    serial: String,
    path: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    crate::installer::install_apk(&state.resources.adb_path(), &serial, &path, &app).await
}

#[tauri::command]
pub async fn install_aab(
    state: State<'_, AppState>,
    serial: String,
    path: String,
    keystore: Option<crate::installer::KeystoreConfig>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    crate::installer::install_aab(
        &state.resources,
        &serial,
        &path,
        keystore.as_ref(),
        &app,
    ).await
}
