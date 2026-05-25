use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Usb,
    Wifi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
    Unauthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub serial: String,
    pub model: String,
    pub product: String,
    pub transport: TransportType,
    pub status: DeviceStatus,
}

pub fn parse_device_list(output: &str) -> Vec<Device> {
    let mut devices = Vec::new();
    for line in output.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            continue;
        }
        let serial = parts[0].to_string();
        let rest = parts[1].trim();

        let status = if rest.starts_with("device") {
            DeviceStatus::Online
        } else if rest.starts_with("offline") {
            DeviceStatus::Offline
        } else if rest.starts_with("unauthorized") {
            DeviceStatus::Unauthorized
        } else {
            continue;
        };

        let transport = if serial.contains(':') {
            TransportType::Wifi
        } else {
            TransportType::Usb
        };

        let model = extract_property(rest, "model:").unwrap_or_default();
        let product = extract_property(rest, "product:").unwrap_or_default();

        devices.push(Device {
            serial,
            model,
            product,
            transport,
            status,
        });
    }
    devices
}

fn extract_property(text: &str, key: &str) -> Option<String> {
    text.find(key).map(|start| {
        let value_start = start + key.len();
        let value_end = text[value_start..]
            .find(' ')
            .map(|i| value_start + i)
            .unwrap_or(text.len());
        text[value_start..value_end].to_string()
    })
}

pub type DeviceMap = Arc<Mutex<HashMap<String, Device>>>;

pub struct DeviceManager {
    adb_path: PathBuf,
    devices: DeviceMap,
}

impl DeviceManager {
    pub fn new(adb_path: PathBuf) -> Self {
        Self {
            adb_path,
            devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn devices(&self) -> DeviceMap {
        self.devices.clone()
    }

    pub async fn poll_devices(&self) -> Result<Vec<Device>, String> {
        let output = crate::util::create_command(&self.adb_path)
            .args(["devices", "-l"])
            .output()
            .await
            .map_err(|e| format!("adb devices failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_device_list(&stdout))
    }

    pub async fn start_polling(&self, app: AppHandle) {
        let devices = self.devices.clone();
        let adb_path = self.adb_path.clone();
        tokio::spawn(async move {
            loop {
                let output = crate::util::create_command(&adb_path)
                    .args(["devices", "-l"])
                    .output()
                    .await;

                if let Ok(output) = output {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let new_devices = parse_device_list(&stdout);
                    let new_map: HashMap<String, Device> = new_devices
                        .into_iter()
                        .map(|d| (d.serial.clone(), d))
                        .collect();

                    let mut current = devices.lock().await;
                    if device_map_changed(&current, &new_map) {
                        *current = new_map.clone();
                        let device_list: Vec<Device> = new_map.into_values().collect();
                        let _ = app.emit("devices-changed", &device_list);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    }

}

/// Standalone polling function that uses a shared DeviceMap.
/// Called from main.rs so the polling writes to the same map that AppState reads from.
pub async fn start_polling_shared(adb_path: PathBuf, devices: DeviceMap, app: AppHandle) {
    tokio::spawn(async move {
        loop {
            let output = crate::util::create_command(&adb_path)
                .args(["devices", "-l"])
                .output()
                .await;

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let new_devices = parse_device_list(&stdout);
                let new_map: HashMap<String, Device> = new_devices
                    .into_iter()
                    .map(|d| (d.serial.clone(), d))
                    .collect();

                let mut current = devices.lock().await;
                if device_map_changed(&current, &new_map) {
                    *current = new_map.clone();
                    let device_list: Vec<Device> = new_map.into_values().collect();
                    let _ = app.emit("devices-changed", &device_list);
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });
}

impl DeviceManager {
    pub async fn connect_wifi(&self, addr: &str) -> Result<String, String> {
        let output = run_with_timeout(&self.adb_path, &["connect", addr], 10).await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if stdout.contains("connected") {
            Ok(stdout)
        } else {
            Err(stdout)
        }
    }

    pub async fn pair_device(&self, addr: &str, code: &str) -> Result<String, String> {
        let output = run_with_timeout(&self.adb_path, &["pair", addr, code], 15).await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if stdout.contains("Successfully paired") {
            Ok(stdout)
        } else {
            Err(stdout)
        }
    }

    pub async fn disconnect_device(&self, serial: &str) -> Result<String, String> {
        let output = run_with_timeout(&self.adb_path, &["disconnect", serial], 5).await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn restart_adb(&self) -> Result<String, String> {
        // kill-server terminates all adb processes
        let _ = run_with_timeout(&self.adb_path, &["kill-server"], 5).await;
        // start-server launches a fresh adb daemon
        let output = run_with_timeout(&self.adb_path, &["start-server"], 10).await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok(format!("{}{}", stdout, stderr).trim().to_string())
    }
}

/// Run an adb command with a timeout in seconds. Kills the process if it exceeds the limit.
async fn run_with_timeout(adb_path: &PathBuf, args: &[&str], timeout_secs: u64) -> Result<std::process::Output, String> {
    let child = crate::util::create_command(adb_path)
        .args(args)
        .kill_on_drop(true)
        .output();

    match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), child).await {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(format!("adb command failed: {}", e)),
        Err(_) => {
            // Timeout — child is dropped here, kill_on_drop sends SIGKILL
            Err(format!("Connection timed out after {}s", timeout_secs))
        }
    }
}

fn device_map_changed(old: &HashMap<String, Device>, new: &HashMap<String, Device>) -> bool {
    if old.len() != new.len() {
        return true;
    }
    for (serial, new_dev) in new {
        match old.get(serial) {
            None => return true,
            Some(old_dev) => {
                if old_dev.status != new_dev.status || old_dev.transport != new_dev.transport {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_devices_list_single_usb() {
        let output = "List of devices attached\nR5CT900ABCD          device usb:1-1 product:starqltechn model:SM_G9600 device:starqltechn transport_id:1\n\n";
        let devices = parse_device_list(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "R5CT900ABCD");
        assert_eq!(devices[0].model, "SM_G9600");
        assert_eq!(devices[0].transport, TransportType::Usb);
        assert_eq!(devices[0].status, DeviceStatus::Online);
    }

    #[test]
    fn test_parse_devices_list_wifi() {
        let output = "List of devices attached\n192.168.1.100:5555   device product:starqltechn model:SM_G9600 device:starqltechn transport_id:2\n\n";
        let devices = parse_device_list(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "192.168.1.100:5555");
        assert_eq!(devices[0].transport, TransportType::Wifi);
    }

    #[test]
    fn test_parse_devices_unauthorized() {
        let output = "List of devices attached\nR5CT900ABCD          unauthorized usb:1-1 transport_id:1\n\n";
        let devices = parse_device_list(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].status, DeviceStatus::Unauthorized);
    }

    #[test]
    fn test_parse_devices_empty() {
        let output = "List of devices attached\n\n";
        let devices = parse_device_list(output);
        assert_eq!(devices.len(), 0);
    }

    #[test]
    fn test_parse_devices_multiple() {
        let output = "List of devices attached\nR5CT900ABCD          device usb:1-1 product:p1 model:M1 device:d1 transport_id:1\n192.168.1.50:5555    device product:p2 model:M2 device:d2 transport_id:2\n\n";
        let devices = parse_device_list(output);
        assert_eq!(devices.len(), 2);
    }
}
