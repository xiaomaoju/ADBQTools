use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use tauri::{AppHandle, Emitter};
use crate::embedded::EmbeddedResources;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreConfig {
    pub path: String,
    pub alias: String,
    pub store_password: String,
    pub key_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub stage: String,
    pub message: String,
}

pub async fn install_apk(
    adb_path: &PathBuf,
    serial: &str,
    apk_path: &str,
    app: &AppHandle,
) -> Result<String, String> {
    let _ = app.emit("install-progress", InstallProgress {
        stage: "installing".to_string(),
        message: "Installing APK...".to_string(),
    });

    let output = Command::new(adb_path)
        .args(["-s", serial, "install", "-r", "-d", apk_path])
        .output()
        .await
        .map_err(|e| format!("adb install failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if stdout.contains("Success") {
        let _ = app.emit("install-progress", InstallProgress {
            stage: "complete".to_string(),
            message: "APK installed successfully".to_string(),
        });
        Ok(stdout)
    } else {
        let _ = app.emit("install-progress", InstallProgress {
            stage: "failed".to_string(),
            message: format!("Install failed: {} {}", stdout, stderr),
        });
        Err(format!("{} {}", stdout, stderr))
    }
}

pub async fn install_aab(
    resources: &EmbeddedResources,
    serial: &str,
    aab_path: &str,
    keystore: Option<&KeystoreConfig>,
    app: &AppHandle,
) -> Result<String, String> {
    let _ = app.emit("install-progress", InstallProgress {
        stage: "building".to_string(),
        message: "Building APKs from AAB...".to_string(),
    });

    let tmp_dir = std::env::temp_dir();
    let apks_path = tmp_dir.join("androidqtools_temp.apks");
    if apks_path.exists() {
        std::fs::remove_file(&apks_path).ok();
    }

    let java_path = resources.jre_java_path();
    let bundletool_path = resources.bundletool_path();
    let adb_path = resources.adb_path();

    let mut args = vec![
        "-jar".to_string(),
        bundletool_path.to_string_lossy().to_string(),
        "build-apks".to_string(),
        format!("--bundle={}", aab_path),
        format!("--output={}", apks_path.to_string_lossy()),
        "--connected-device".to_string(),
        format!("--adb={}", adb_path.to_string_lossy()),
        format!("--device-id={}", serial),
    ];

    if let Some(ks) = keystore {
        args.push(format!("--ks={}", ks.path));
        args.push(format!("--ks-key-alias={}", ks.alias));
        args.push(format!("--ks-pass=pass:{}", ks.store_password));
        args.push(format!("--key-pass=pass:{}", ks.key_password));
    }

    let output = Command::new(&java_path)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("bundletool build-apks failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let _ = app.emit("install-progress", InstallProgress {
            stage: "failed".to_string(),
            message: format!("Build APKs failed: {}", stderr),
        });
        return Err(stderr);
    }

    let _ = app.emit("install-progress", InstallProgress {
        stage: "installing".to_string(),
        message: "Installing APKs to device...".to_string(),
    });

    let install_output = Command::new(&java_path)
        .args([
            "-jar",
            &bundletool_path.to_string_lossy(),
            "install-apks",
            &format!("--apks={}", apks_path.to_string_lossy()),
            &format!("--adb={}", adb_path.to_string_lossy()),
            &format!("--device-id={}", serial),
        ])
        .output()
        .await
        .map_err(|e| format!("bundletool install-apks failed: {}", e))?;

    std::fs::remove_file(&apks_path).ok();

    if install_output.status.success() {
        let _ = app.emit("install-progress", InstallProgress {
            stage: "complete".to_string(),
            message: "AAB installed successfully".to_string(),
        });
        Ok("AAB installed successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&install_output.stderr).to_string();
        let _ = app.emit("install-progress", InstallProgress {
            stage: "failed".to_string(),
            message: format!("Install failed: {}", stderr),
        });
        Err(stderr)
    }
}
