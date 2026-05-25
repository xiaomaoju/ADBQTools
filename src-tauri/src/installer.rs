use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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

/// Copy bundletool.jar to temp dir to avoid any path encoding issues on Windows.
/// Java on Windows can fail to load jars from paths with spaces or non-ASCII characters.
pub fn safe_jar_path(original: &PathBuf) -> Result<PathBuf, String> {
    let tmp = std::env::temp_dir().join("adbqtools_bundletool.jar");
    std::fs::copy(original, &tmp)
        .map_err(|e| format!("Failed to copy bundletool.jar to temp: {}", e))?;
    Ok(tmp)
}

pub const BUNDLETOOL_MAIN: &str = "com.android.tools.build.bundletool.BundleToolMain";

pub async fn list_keystore_aliases(
    resources: &EmbeddedResources,
    keystore_path: &str,
    store_password: &str,
) -> Result<Vec<String>, String> {
    let keytool = resources.keytool_path();
    if !keytool.exists() {
        return Err("keytool not found in embedded JRE".to_string());
    }

    let output = crate::util::create_command(&keytool)
        .args([
            "-list",
            "-keystore", keystore_path,
            "-storepass", store_password,
        ])
        .output()
        .await
        .map_err(|e| format!("keytool failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!("keytool error: {} {}", stdout, stderr));
    }

    // keytool -list output format: each alias line looks like:
    // "alias_name, Oct 10, 2023, PrivateKeyEntry,"
    let aliases: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Alias lines contain a comma-separated date and entry type
            if line.contains("PrivateKeyEntry") || line.contains("SecretKeyEntry") || line.contains("trustedCertEntry") {
                line.split(',').next().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .filter(|alias| !alias.is_empty())
        .collect();

    Ok(aliases)
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

    let output = crate::util::create_command(adb_path)
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
    let apks_path = tmp_dir.join("adbqtools_temp.apks");
    if apks_path.exists() {
        std::fs::remove_file(&apks_path).ok();
    }

    let java_path = resources.jre_java_path();
    let bundletool_path = resources.bundletool_path();
    let adb_path = resources.adb_path();

    // Validate paths exist before invoking
    if !java_path.exists() {
        return Err(format!("java not found at: {:?}", java_path));
    }
    if !bundletool_path.exists() {
        return Err(format!("bundletool.jar not found at: {:?}", bundletool_path));
    }

    // On Windows, Java may fail to load jars from paths with non-ASCII characters.
    // Copy bundletool.jar to a temp dir with a safe ASCII path.
    let effective_jar = safe_jar_path(&bundletool_path)?;

    let mut args = vec![
        "-cp".to_string(),
        effective_jar.to_string_lossy().to_string(),
        BUNDLETOOL_MAIN.to_string(),
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

    let output = crate::util::create_command(&java_path)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("bundletool build-apks failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let error_msg = if stderr.trim().is_empty() { stdout } else { stderr };
        let detail = format!("Build APKs failed:\njava: {}\njar: {}\n{}", java_path.display(), effective_jar.display(), error_msg);
        let _ = app.emit("install-progress", InstallProgress {
            stage: "failed".to_string(),
            message: detail.clone(),
        });
        return Err(detail);
    }

    let _ = app.emit("install-progress", InstallProgress {
        stage: "installing".to_string(),
        message: "Installing APKs to device...".to_string(),
    });

    let install_output = crate::util::create_command(&java_path)
        .args([
            "-cp",
            &effective_jar.to_string_lossy(),
            BUNDLETOOL_MAIN,
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
