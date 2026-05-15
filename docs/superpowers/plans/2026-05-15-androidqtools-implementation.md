# AndroidQTools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform (Windows + macOS) Android developer tool with Logcat viewer, APK/AAB installer, and multi-device management using Tauri 2 + Rust + Svelte.

**Architecture:** Rust backend manages all ADB communication via subprocess calls to embedded adb binary. Frontend communicates via Tauri Commands (request/response) and Tauri Events (real-time logcat streaming). Embedded resources (adb, bundletool, JRE) ship with the app for zero external dependencies.

**Tech Stack:** Tauri 2, Rust + Tokio, Svelte 5 + TypeScript, SQLite (rusqlite), CSS custom properties for theming.

**Spec:** `docs/superpowers/specs/2026-05-15-androidqtools-design.md`

---

## File Structure

### Rust Backend (`src-tauri/src/`)
- `main.rs` — Tauri app entry, plugin registration, command registration
- `lib.rs` — Module declarations
- `embedded.rs` — Embedded resource path resolution (adb, bundletool, JRE)
- `device_manager.rs` — Device polling, WiFi connect/pair, state management
- `logcat_engine.rs` — Logcat subprocess management, line parsing, batched streaming
- `unity_parser.rs` — Unity-specific log parsing (stack frames, crash detection)
- `installer.rs` — APK/AAB installation logic
- `apk_parser.rs` — APK manifest binary XML parsing
- `db.rs` — SQLite log persistence and history queries
- `commands.rs` — All Tauri command handlers

### Rust Tests (`src-tauri/src/` inline + `src-tauri/tests/`)
- Inline `#[cfg(test)]` modules in each source file for unit tests
- `tests/logcat_parsing.rs` — Integration tests for logcat line parsing
- `tests/unity_parsing.rs` — Integration tests for Unity log parsing

### Frontend (`src/`)
- `app.html` — HTML shell
- `App.svelte` — Root component, layout shell
- `lib/components/DevicePanel.svelte` — Left sidebar device list
- `lib/components/WifiDialog.svelte` — WiFi connect/pair dialog
- `lib/components/Toolbar.svelte` — Top toolbar with view switching
- `lib/components/LogcatViewer.svelte` — Logcat virtual scroll list + filters
- `lib/components/LogcatFilterBar.svelte` — Filter controls (level, tag, search, Unity mode)
- `lib/components/LogcatRow.svelte` — Single log row with level coloring
- `lib/components/StackGroup.svelte` — Collapsible stack trace group
- `lib/components/Installer.svelte` — APK/AAB installer view
- `lib/components/AppPreview.svelte` — APK info preview card
- `lib/components/InstallHistory.svelte` — Installation history list
- `lib/components/StatusBar.svelte` — Bottom status bar
- `lib/components/ui/Button.svelte` — Reusable button
- `lib/components/ui/Dialog.svelte` — Modal dialog
- `lib/components/ui/Select.svelte` — Dropdown select
- `lib/components/ui/Tabs.svelte` — Tab container
- `lib/components/ui/ProgressBar.svelte` — Progress bar
- `lib/components/ui/Input.svelte` — Text input
- `lib/stores/devices.ts` — Device state store
- `lib/stores/logcat.ts` — Logcat entries store with filtering
- `lib/stores/installer.ts` — Install state store
- `lib/utils/tauri.ts` — Tauri invoke/listen wrappers
- `lib/utils/format.ts` — Timestamp/size formatting
- `lib/types.ts` — Shared TypeScript types
- `styles/variables.css` — CSS custom properties (theme)
- `styles/global.css` — Global reset and base styles

---

## Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `svelte.config.js`, `vite.config.ts`, `tsconfig.json`, `src/app.html`, `src/App.svelte`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src/styles/variables.css`, `src/styles/global.css`

- [ ] **Step 1: Initialize Tauri 2 + Svelte project**

```bash
cd /Users/children/Documents/Projects/AndroidQTools
npm create tauri-app@latest . -- --template svelte-ts --manager npm --yes
```

If the directory is not empty, move the spec/plan files out first, init, then move back.

- [ ] **Step 2: Verify scaffolding works**

```bash
cd /Users/children/Documents/Projects/AndroidQTools
npm install
```

Expected: dependencies install without errors.

- [ ] **Step 3: Add Rust dependencies to Cargo.toml**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.31", features = ["bundled"] }
regex = "1"
dirs = "5"
zip = "2"
chrono = "0.4"
log = "0.4"
env_logger = "0.11"
```

- [ ] **Step 4: Set up CSS theme variables**

Create `src/styles/variables.css`:

```css
:root {
  --bg-primary: #1e1e1e;
  --bg-secondary: #252526;
  --bg-tertiary: #2d2d2d;
  --bg-hover: #3c3c3c;
  --border-color: #3c3c3c;
  --text-primary: #cccccc;
  --text-secondary: #858585;
  --text-bright: #e0e0e0;
  --accent: #007acc;
  --accent-hover: #1a8ad4;
  --success: #4ec9b0;
  --warning: #cca700;
  --error: #f44747;

  --log-verbose: #6a6a6a;
  --log-debug: #4fc1ff;
  --log-info: #4ec9b0;
  --log-warn: #cca700;
  --log-error: #f44747;
  --log-fatal: #c586c0;

  --font-ui: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', 'Consolas', monospace;

  --sidebar-width: 220px;
  --toolbar-height: 40px;
  --statusbar-height: 28px;

  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
}
```

- [ ] **Step 5: Set up global CSS reset**

Create `src/styles/global.css`:

```css
@import './variables.css';

*, *::before, *::after {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  height: 100%;
  overflow: hidden;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: 13px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
}

::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--bg-hover);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #555;
}

::selection {
  background: var(--accent);
  color: white;
}
```

- [ ] **Step 6: Set up root App.svelte shell**

Replace `src/App.svelte`:

```svelte
<script lang="ts">
  import './styles/global.css';
</script>

<main class="app-shell">
  <div class="sidebar">
    <p style="padding: 12px; color: var(--text-secondary);">Devices</p>
  </div>
  <div class="content">
    <div class="toolbar">
      <p style="padding: 8px;">AndroidQTools</p>
    </div>
    <div class="main-area">
      <p style="padding: 20px; color: var(--text-secondary);">Main content area</p>
    </div>
    <div class="statusbar">
      <span>Ready</span>
    </div>
  </div>
</main>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
  }
  .sidebar {
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
  }
  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .toolbar {
    height: var(--toolbar-height);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
  }
  .main-area {
    flex: 1;
    overflow: hidden;
  }
  .statusbar {
    height: var(--statusbar-height);
    background: var(--accent);
    color: white;
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-size: 12px;
  }
</style>
```

- [ ] **Step 7: Set up Rust entry point**

Replace `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod embedded;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

Create `src-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod embedded;
```

Create `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

Create `src-tauri/src/embedded.rs`:

```rust
// Placeholder — implemented in Task 2
```

- [ ] **Step 8: Verify the app builds and runs**

```bash
cd /Users/children/Documents/Projects/AndroidQTools
npm run tauri dev
```

Expected: Window opens showing the dark-themed shell layout with sidebar, toolbar, main area, and blue status bar.

- [ ] **Step 9: Commit**

```bash
git init
echo "node_modules/\ntarget/\ndist/\n.DS_Store" > .gitignore
git add -A
git commit -m "feat: scaffold Tauri 2 + Svelte + TS project with dark theme shell"
```

---

## Task 2: Embedded Resource Management

**Files:**
- Create: `src-tauri/src/embedded.rs`

- [ ] **Step 1: Write tests for embedded path resolution**

Add to `src-tauri/src/embedded.rs`:

```rust
use std::path::PathBuf;
use std::fs;

pub struct EmbeddedResources {
    base_dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_subdir() {
        let subdir = platform_subdir();
        if cfg!(target_os = "macos") {
            assert_eq!(subdir, "macos");
        } else if cfg!(target_os = "windows") {
            assert_eq!(subdir, "windows");
        }
    }

    #[test]
    fn test_adb_binary_name() {
        let name = adb_binary_name();
        if cfg!(target_os = "windows") {
            assert_eq!(name, "adb.exe");
        } else {
            assert_eq!(name, "adb");
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: FAIL — `platform_subdir` and `adb_binary_name` not found.

- [ ] **Step 3: Implement embedded resource resolution**

Replace `src-tauri/src/embedded.rs`:

```rust
use std::path::PathBuf;
use std::fs;

fn platform_subdir() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        panic!("Unsupported platform")
    }
}

fn adb_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    }
}

pub struct EmbeddedResources {
    resource_dir: PathBuf,
    data_dir: PathBuf,
}

impl EmbeddedResources {
    pub fn new(resource_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self { resource_dir, data_dir }
    }

    pub fn adb_path(&self) -> PathBuf {
        self.resource_dir.join(platform_subdir()).join(adb_binary_name())
    }

    pub fn bundletool_path(&self) -> PathBuf {
        self.resource_dir.join(platform_subdir()).join("bundletool.jar")
    }

    pub fn jre_java_path(&self) -> PathBuf {
        let jre_dir = self.data_dir.join("jre");
        if cfg!(target_os = "windows") {
            jre_dir.join("bin").join("java.exe")
        } else {
            jre_dir.join("Contents").join("Home").join("bin").join("java")
        }
    }

    pub fn jre_source_dir(&self) -> PathBuf {
        self.resource_dir.join(platform_subdir()).join("jre")
    }

    pub fn jre_data_dir(&self) -> PathBuf {
        self.data_dir.join("jre")
    }

    pub fn ensure_jre_extracted(&self) -> Result<(), String> {
        let target = self.jre_data_dir();
        if target.exists() {
            return Ok(());
        }
        let source = self.jre_source_dir();
        if !source.exists() {
            return Err(format!("JRE source not found at {:?}", source));
        }
        copy_dir_recursive(&source, &target)
            .map_err(|e| format!("Failed to extract JRE: {}", e))
    }

    #[cfg(unix)]
    pub fn ensure_executable_permissions(&self) -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;
        let adb = self.adb_path();
        if adb.exists() {
            fs::set_permissions(&adb, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("chmod adb failed: {}", e))?;
        }
        let java = self.jre_java_path();
        if java.exists() {
            fs::set_permissions(&java, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("chmod java failed: {}", e))?;
        }
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn ensure_executable_permissions(&self) -> Result<(), String> {
        Ok(())
    }
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_subdir() {
        let subdir = platform_subdir();
        if cfg!(target_os = "macos") {
            assert_eq!(subdir, "macos");
        } else if cfg!(target_os = "windows") {
            assert_eq!(subdir, "windows");
        }
    }

    #[test]
    fn test_adb_binary_name() {
        let name = adb_binary_name();
        if cfg!(target_os = "windows") {
            assert_eq!(name, "adb.exe");
        } else {
            assert_eq!(name, "adb");
        }
    }

    #[test]
    fn test_adb_path() {
        let res = EmbeddedResources::new(
            PathBuf::from("/app/resources"),
            PathBuf::from("/app/data"),
        );
        let adb = res.adb_path();
        if cfg!(target_os = "macos") {
            assert_eq!(adb, PathBuf::from("/app/resources/macos/adb"));
        }
    }

    #[test]
    fn test_jre_java_path() {
        let res = EmbeddedResources::new(
            PathBuf::from("/app/resources"),
            PathBuf::from("/app/data"),
        );
        let java = res.jre_java_path();
        if cfg!(target_os = "macos") {
            assert_eq!(java, PathBuf::from("/app/data/jre/Contents/Home/bin/java"));
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/embedded.rs
git commit -m "feat: add embedded resource path resolution for adb/bundletool/JRE"
```

---

## Task 3: Device Manager (Rust)

**Files:**
- Create: `src-tauri/src/device_manager.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write tests for adb output parsing**

Create `src-tauri/src/device_manager.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Command;
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
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: FAIL — `parse_device_list` not found.

- [ ] **Step 3: Implement device list parsing**

Add above the `#[cfg(test)]` block in `device_manager.rs`:

```rust
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
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All device parsing tests PASS.

- [ ] **Step 5: Implement DeviceManager with polling**

Add to `device_manager.rs` after the parsing functions:

```rust
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
        let output = Command::new(&self.adb_path)
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
                let manager = DeviceManager { adb_path: adb_path.clone(), devices: devices.clone() };
                if let Ok(new_devices) = manager.poll_devices().await {
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

    pub async fn connect_wifi(&self, addr: &str) -> Result<String, String> {
        let output = Command::new(&self.adb_path)
            .args(["connect", addr])
            .output()
            .await
            .map_err(|e| format!("adb connect failed: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if stdout.contains("connected") {
            Ok(stdout)
        } else {
            Err(stdout)
        }
    }

    pub async fn pair_device(&self, addr: &str, code: &str) -> Result<String, String> {
        let output = Command::new(&self.adb_path)
            .args(["pair", addr, code])
            .output()
            .await
            .map_err(|e| format!("adb pair failed: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if stdout.contains("Successfully paired") {
            Ok(stdout)
        } else {
            Err(stdout)
        }
    }

    pub async fn disconnect_device(&self, serial: &str) -> Result<String, String> {
        let output = Command::new(&self.adb_path)
            .args(["disconnect", serial])
            .output()
            .await
            .map_err(|e| format!("adb disconnect failed: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
```

- [ ] **Step 6: Register module in lib.rs**

Update `src-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod device_manager;
pub mod embedded;
```

- [ ] **Step 7: Run all tests**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All tests PASS.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/device_manager.rs src-tauri/src/lib.rs
git commit -m "feat: add device manager with adb polling, WiFi connect, and pairing"
```

---

## Task 4: Logcat Engine — Core Parsing (Rust)

**Files:**
- Create: `src-tauri/src/logcat_engine.rs`
- Create: `src-tauri/src/unity_parser.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write tests for logcat line parsing**

Create `src-tauri/src/logcat_engine.rs`:

```rust
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    System,
    Unity,
    Il2Cpp,
    Mono,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub module: String,
    pub class_name: String,
    pub method_name: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: String,
    pub pid: u32,
    pub tid: u32,
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
    pub source: LogSource,
    pub stack_frames: Option<Vec<StackFrame>>,
    pub unity_script_info: Option<ScriptInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_logcat_line() {
        let line = "05-15 10:30:45.123  1234  5678 I ActivityManager: Start proc com.example.app";
        let entry = parse_logcat_line(line, 1);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.timestamp, "05-15 10:30:45.123");
        assert_eq!(entry.pid, 1234);
        assert_eq!(entry.tid, 5678);
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.tag, "ActivityManager");
        assert_eq!(entry.message, "Start proc com.example.app");
        assert_eq!(entry.source, LogSource::System);
    }

    #[test]
    fn test_parse_unity_log_line() {
        let line = "05-15 10:30:45.123  1234  5678 I Unity   : Player initialized successfully";
        let entry = parse_logcat_line(line, 2);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.tag, "Unity");
        assert_eq!(entry.source, LogSource::Unity);
    }

    #[test]
    fn test_parse_all_log_levels() {
        let cases = vec![
            ("V", LogLevel::Verbose),
            ("D", LogLevel::Debug),
            ("I", LogLevel::Info),
            ("W", LogLevel::Warn),
            ("E", LogLevel::Error),
            ("F", LogLevel::Fatal),
        ];
        for (letter, expected_level) in cases {
            let line = format!("05-15 10:30:45.123  1234  5678 {} TestTag : test message", letter);
            let entry = parse_logcat_line(&line, 1).unwrap();
            assert_eq!(entry.level, expected_level);
        }
    }

    #[test]
    fn test_parse_invalid_line() {
        let line = "--- beginning of main";
        let entry = parse_logcat_line(line, 1);
        assert!(entry.is_none());
    }

    #[test]
    fn test_detect_unity_source() {
        assert_eq!(detect_source("Unity"), LogSource::Unity);
        assert_eq!(detect_source("Il2Cpp"), LogSource::Il2Cpp);
        assert_eq!(detect_source("Mono"), LogSource::Mono);
        assert_eq!(detect_source("CRASH"), LogSource::Unity);
        assert_eq!(detect_source("ActivityManager"), LogSource::System);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: FAIL — `parse_logcat_line` and `detect_source` not found.

- [ ] **Step 3: Implement logcat line parser**

Add above `#[cfg(test)]` in `logcat_engine.rs`:

```rust
static LOGCAT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d{3})\s+(\d+)\s+(\d+)\s+([VDIWEF])\s+(.+?)\s*:\s+(.*)"
    ).unwrap()
});

pub fn detect_source(tag: &str) -> LogSource {
    match tag.trim() {
        "Unity" | "CRASH" => LogSource::Unity,
        t if t.starts_with("Il2Cpp") => LogSource::Il2Cpp,
        t if t.starts_with("Mono") => LogSource::Mono,
        _ => LogSource::System,
    }
}

fn parse_level(ch: &str) -> LogLevel {
    match ch {
        "V" => LogLevel::Verbose,
        "D" => LogLevel::Debug,
        "I" => LogLevel::Info,
        "W" => LogLevel::Warn,
        "E" => LogLevel::Error,
        "F" => LogLevel::Fatal,
        _ => LogLevel::Verbose,
    }
}

pub fn parse_logcat_line(line: &str, id: u64) -> Option<LogEntry> {
    let caps = LOGCAT_RE.captures(line)?;
    let tag = caps[5].trim().to_string();
    let source = detect_source(&tag);

    Some(LogEntry {
        id,
        timestamp: caps[1].to_string(),
        pid: caps[2].parse().unwrap_or(0),
        tid: caps[3].parse().unwrap_or(0),
        level: parse_level(&caps[4]),
        tag,
        message: caps[6].to_string(),
        source,
        stack_frames: None,
        unity_script_info: None,
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All logcat parsing tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/logcat_engine.rs
git commit -m "feat: add logcat line parser with Unity source detection"
```

---

## Task 5: Unity Log Parser

**Files:**
- Create: `src-tauri/src/unity_parser.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write tests for Unity stack frame parsing**

Create `src-tauri/src/unity_parser.rs`:

```rust
use regex::Regex;
use std::sync::LazyLock;
use crate::logcat_engine::{StackFrame, ScriptInfo};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_il2cpp_stack_frame() {
        let line = "  at GameManager.Initialize () [0x00000] in /Build/Assembly-CSharp/GameManager.cs:42";
        let frame = parse_stack_frame(line);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.class_name, "GameManager");
        assert_eq!(frame.method_name, "Initialize");
        assert_eq!(frame.file, Some("GameManager.cs".to_string()));
        assert_eq!(frame.line, Some(42));
    }

    #[test]
    fn test_parse_mono_stack_frame() {
        let line = "  at UnityEngine.Application.CallLogCallback (System.String condition) [0x00000] in <abc123>:0";
        let frame = parse_stack_frame(line);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.class_name, "Application");
        assert_eq!(frame.method_name, "CallLogCallback");
        assert_eq!(frame.module, "UnityEngine");
    }

    #[test]
    fn test_parse_script_info() {
        let msg = "NullReferenceException: Object reference not set\n  at PlayerController.Update () [0x00000] in /Build/Assembly-CSharp/PlayerController.cs:128";
        let info = extract_script_info(msg);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.file, "PlayerController.cs");
        assert_eq!(info.line, 128);
    }

    #[test]
    fn test_is_stack_frame_line() {
        assert!(is_stack_frame_line("  at GameManager.Start () [0x00000] in file:1"));
        assert!(is_stack_frame_line("  at Foo.Bar ()"));
        assert!(!is_stack_frame_line("NullReferenceException: something"));
        assert!(!is_stack_frame_line("normal log message"));
    }

    #[test]
    fn test_is_native_crash_signal() {
        assert!(is_native_crash("signal 6 (SIGABRT), code -1"));
        assert!(is_native_crash("signal 11 (SIGSEGV), code 1"));
        assert!(!is_native_crash("normal log message"));
    }

    #[test]
    fn test_parse_stack_frames_batch() {
        let lines = vec![
            "  at GameManager.Start () [0x00000] in /Build/Assembly-CSharp/GameManager.cs:10",
            "  at UnityEngine.Object.Instantiate () [0x00000] in <abc>:0",
        ];
        let frames = parse_stack_frames(&lines);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].class_name, "GameManager");
        assert_eq!(frames[1].module, "UnityEngine");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: FAIL — functions not implemented.

- [ ] **Step 3: Implement Unity parser**

Add above `#[cfg(test)]` in `unity_parser.rs`:

```rust
static STACK_FRAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s+at\s+(?:(.+)\.)?(\w+)\.(\w+)\s*\(").unwrap()
});

static FILE_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:in\s+\S*/)?(\w+\.cs):(\d+)").unwrap()
});

static NATIVE_CRASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"signal\s+\d+\s+\(SIG[A-Z]+\)").unwrap()
});

pub fn is_stack_frame_line(line: &str) -> bool {
    line.trim_start().starts_with("at ") && line.contains('(')
}

pub fn is_native_crash(message: &str) -> bool {
    NATIVE_CRASH_RE.is_match(message)
}

pub fn parse_stack_frame(line: &str) -> Option<StackFrame> {
    let caps = STACK_FRAME_RE.captures(line)?;
    let module = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
    let class_name = caps[2].to_string();
    let method_name = caps[3].to_string();

    let (file, file_line) = if let Some(fc) = FILE_LINE_RE.captures(line) {
        (
            Some(fc[1].to_string()),
            fc[2].parse::<u32>().ok(),
        )
    } else {
        (None, None)
    };

    Some(StackFrame {
        module,
        class_name,
        method_name,
        file,
        line: file_line,
    })
}

pub fn parse_stack_frames(lines: &[&str]) -> Vec<StackFrame> {
    lines.iter().filter_map(|line| parse_stack_frame(line)).collect()
}

pub fn extract_script_info(message: &str) -> Option<ScriptInfo> {
    let caps = FILE_LINE_RE.captures(message)?;
    Some(ScriptInfo {
        file: caps[1].to_string(),
        line: caps[2].parse().ok()?,
    })
}
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod commands;
pub mod device_manager;
pub mod embedded;
pub mod logcat_engine;
pub mod unity_parser;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All Unity parser tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/unity_parser.rs src-tauri/src/lib.rs
git commit -m "feat: add Unity log parser for stack frames, crash detection, and script info"
```

---

## Task 6: Logcat Streaming Engine

**Files:**
- Modify: `src-tauri/src/logcat_engine.rs`

- [ ] **Step 1: Add LogcatSession streaming implementation**

Add to the end of `logcat_engine.rs` (before `#[cfg(test)]`):

```rust
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tauri::{AppHandle, Emitter};
use crate::unity_parser;

pub struct LogcatSession {
    serial: String,
    adb_path: PathBuf,
    paused: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    id_counter: Arc<AtomicU64>,
}

impl LogcatSession {
    pub fn new(serial: String, adb_path: PathBuf) -> Self {
        Self {
            serial,
            adb_path,
            paused: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            id_counter: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub async fn start(&self, app: AppHandle) {
        self.running.store(true, Ordering::Relaxed);
        let serial = self.serial.clone();
        let adb_path = self.adb_path.clone();
        let paused = self.paused.clone();
        let running = self.running.clone();
        let buffer = self.buffer.clone();
        let id_counter = self.id_counter.clone();

        tokio::spawn(async move {
            let mut child = match Command::new(&adb_path)
                .args(["-s", &serial, "logcat", "-v", "threadtime"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    log::error!("Failed to start logcat for {}: {}", serial, e);
                    return;
                }
            };

            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout).lines();
            let mut batch: Vec<LogEntry> = Vec::with_capacity(50);
            let mut last_flush = tokio::time::Instant::now();
            let event_name = format!("logcat-{}", serial);

            while running.load(Ordering::Relaxed) {
                let timeout = tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    reader.next_line(),
                );

                match timeout.await {
                    Ok(Ok(Some(line))) => {
                        let id = id_counter.fetch_add(1, Ordering::Relaxed);
                        if let Some(mut entry) = parse_logcat_line(&line, id) {
                            if entry.source != LogSource::System {
                                entry.unity_script_info = unity_parser::extract_script_info(&entry.message);
                            }

                            if paused.load(Ordering::Relaxed) {
                                let mut buf = buffer.lock().await;
                                if buf.len() >= 100_000 {
                                    buf.pop_front();
                                }
                                buf.push_back(entry);
                            } else {
                                batch.push(entry);
                            }
                        }
                    }
                    Ok(Ok(None)) => break,
                    Ok(Err(_)) => break,
                    Err(_) => {} // timeout, check flush
                }

                let should_flush = batch.len() >= 50
                    || (last_flush.elapsed().as_millis() >= 100 && !batch.is_empty());

                if should_flush {
                    let _ = app.emit(&event_name, &batch);
                    batch.clear();
                    last_flush = tokio::time::Instant::now();
                }
            }

            let _ = child.kill().await;
        });
    }

    pub async fn flush_buffer(&self, app: &AppHandle) {
        let mut buf = self.buffer.lock().await;
        if !buf.is_empty() {
            let entries: Vec<LogEntry> = buf.drain(..).collect();
            let event_name = format!("logcat-{}", self.serial);
            let _ = app.emit(&event_name, &entries);
        }
    }

    pub async fn clear_logcat(&self) -> Result<(), String> {
        Command::new(&self.adb_path)
            .args(["-s", &self.serial, "logcat", "-c"])
            .output()
            .await
            .map_err(|e| format!("logcat clear failed: {}", e))?;
        Ok(())
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo check
```

Expected: Compiles without errors.

- [ ] **Step 3: Run all tests**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All tests PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/logcat_engine.rs
git commit -m "feat: add logcat streaming engine with pause/resume and batched emission"
```

---

## Task 7: SQLite Persistence

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write tests for database operations**

Create `src-tauri/src/db.rs`:

```rust
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::logcat_engine::LogEntry;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logcat_engine::{LogLevel, LogSource};

    fn make_entry(id: u64, tag: &str, msg: &str, level: LogLevel) -> LogEntry {
        LogEntry {
            id,
            timestamp: "05-15 10:30:45.123".to_string(),
            pid: 1234,
            tid: 5678,
            level,
            tag: tag.to_string(),
            message: msg.to_string(),
            source: LogSource::System,
            stack_frames: None,
            unity_script_info: None,
        }
    }

    #[test]
    fn test_create_table_and_insert() {
        let db = LogDatabase::open_in_memory().unwrap();
        let entry = make_entry(1, "Test", "hello", LogLevel::Info);
        db.insert_entries("device1", &[entry]).unwrap();
        let results = db.query_entries("device1", None, None, 100).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tag, "Test");
    }

    #[test]
    fn test_query_with_limit() {
        let db = LogDatabase::open_in_memory().unwrap();
        let entries: Vec<LogEntry> = (0..10)
            .map(|i| make_entry(i, "Test", &format!("msg {}", i), LogLevel::Info))
            .collect();
        db.insert_entries("device1", &entries).unwrap();
        let results = db.query_entries("device1", None, None, 5).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_separate_device_tables() {
        let db = LogDatabase::open_in_memory().unwrap();
        let entry1 = make_entry(1, "Tag1", "device1 msg", LogLevel::Info);
        let entry2 = make_entry(2, "Tag2", "device2 msg", LogLevel::Error);
        db.insert_entries("device_A", &[entry1]).unwrap();
        db.insert_entries("device_B", &[entry2]).unwrap();
        let r1 = db.query_entries("device_A", None, None, 100).unwrap();
        let r2 = db.query_entries("device_B", None, None, 100).unwrap();
        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 1);
        assert_eq!(r1[0].tag, "Tag1");
        assert_eq!(r2[0].tag, "Tag2");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: FAIL — `LogDatabase` not found.

- [ ] **Step 3: Implement LogDatabase**

Add above `#[cfg(test)]` in `db.rs`:

```rust
pub struct LogDatabase {
    conn: Connection,
}

impl LogDatabase {
    pub fn open(path: &PathBuf) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("DB open failed: {}", e))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("PRAGMA failed: {}", e))?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| format!("DB open failed: {}", e))?;
        Ok(Self { conn })
    }

    fn ensure_table(&self, device: &str) -> Result<(), String> {
        let table = sanitize_table_name(device);
        self.conn
            .execute_batch(&format!(
                "CREATE TABLE IF NOT EXISTS \"{}\" (
                    id INTEGER PRIMARY KEY,
                    timestamp TEXT NOT NULL,
                    pid INTEGER,
                    tid INTEGER,
                    level TEXT NOT NULL,
                    tag TEXT NOT NULL,
                    message TEXT NOT NULL,
                    source TEXT NOT NULL,
                    created_at TEXT DEFAULT (datetime('now'))
                )",
                table
            ))
            .map_err(|e| format!("Create table failed: {}", e))
    }

    pub fn insert_entries(&self, device: &str, entries: &[LogEntry]) -> Result<(), String> {
        self.ensure_table(device)?;
        let table = sanitize_table_name(device);
        let mut stmt = self.conn
            .prepare(&format!(
                "INSERT INTO \"{}\" (id, timestamp, pid, tid, level, tag, message, source) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                table
            ))
            .map_err(|e| format!("Prepare insert failed: {}", e))?;

        for entry in entries {
            stmt.execute(params![
                entry.id,
                entry.timestamp,
                entry.pid,
                entry.tid,
                format!("{:?}", entry.level),
                entry.tag,
                entry.message,
                format!("{:?}", entry.source),
            ])
            .map_err(|e| format!("Insert failed: {}", e))?;
        }
        Ok(())
    }

    pub fn query_entries(
        &self,
        device: &str,
        from: Option<&str>,
        to: Option<&str>,
        limit: u32,
    ) -> Result<Vec<LogEntry>, String> {
        self.ensure_table(device)?;
        let table = sanitize_table_name(device);
        let mut sql = format!("SELECT id, timestamp, pid, tid, level, tag, message, source FROM \"{}\"", table);
        let mut conditions = Vec::new();

        if let Some(f) = from {
            conditions.push(format!("timestamp >= '{}'", f));
        }
        if let Some(t) = to {
            conditions.push(format!("timestamp <= '{}'", t));
        }
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(&format!(" ORDER BY id DESC LIMIT {}", limit));

        let mut stmt = self.conn.prepare(&sql).map_err(|e| format!("Query failed: {}", e))?;
        let entries = stmt
            .query_map([], |row| {
                let level_str: String = row.get(4)?;
                let source_str: String = row.get(7)?;
                Ok(LogEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    pid: row.get(2)?,
                    tid: row.get(3)?,
                    level: parse_log_level(&level_str),
                    tag: row.get(5)?,
                    message: row.get(6)?,
                    source: parse_log_source(&source_str),
                    stack_frames: None,
                    unity_script_info: None,
                })
            })
            .map_err(|e| format!("Query map failed: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(entries)
    }
}

fn sanitize_table_name(device: &str) -> String {
    device.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
}

fn parse_log_level(s: &str) -> crate::logcat_engine::LogLevel {
    use crate::logcat_engine::LogLevel;
    match s {
        "Verbose" => LogLevel::Verbose,
        "Debug" => LogLevel::Debug,
        "Info" => LogLevel::Info,
        "Warn" => LogLevel::Warn,
        "Error" => LogLevel::Error,
        "Fatal" => LogLevel::Fatal,
        _ => LogLevel::Verbose,
    }
}

fn parse_log_source(s: &str) -> crate::logcat_engine::LogSource {
    use crate::logcat_engine::LogSource;
    match s {
        "Unity" => LogSource::Unity,
        "Il2Cpp" => LogSource::Il2Cpp,
        "Mono" => LogSource::Mono,
        _ => LogSource::System,
    }
}
```

- [ ] **Step 4: Update lib.rs and run tests**

Add `pub mod db;` to `lib.rs`.

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db.rs src-tauri/src/lib.rs
git commit -m "feat: add SQLite log persistence with per-device tables"
```

---

## Task 8: Tauri Commands

**Files:**
- Rewrite: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Implement all Tauri command handlers**

Replace `src-tauri/src/commands.rs`:

```rust
use std::collections::HashMap;
use std::path::PathBuf;
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
```

- [ ] **Step 2: Update main.rs with state management and command registration**

Replace `src-tauri/src/main.rs`:

```rust
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
```

- [ ] **Step 3: Create installer.rs stub** (full implementation in Task 9)

Create `src-tauri/src/installer.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod commands;
pub mod db;
pub mod device_manager;
pub mod embedded;
pub mod installer;
pub mod logcat_engine;
pub mod unity_parser;
```

- [ ] **Step 5: Verify compilation**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo check
```

Expected: Compiles without errors.

- [ ] **Step 6: Run all tests**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All tests PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/installer.rs src-tauri/src/main.rs src-tauri/src/lib.rs
git commit -m "feat: add Tauri commands, installer module, and app state wiring"
```

---

## Task 9: Frontend — TypeScript Types and Stores

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/utils/tauri.ts`
- Create: `src/lib/utils/format.ts`
- Create: `src/lib/stores/devices.ts`
- Create: `src/lib/stores/logcat.ts`
- Create: `src/lib/stores/installer.ts`

- [ ] **Step 1: Create shared types**

Create `src/lib/types.ts`:

```typescript
export type TransportType = 'usb' | 'wifi';
export type DeviceStatus = 'online' | 'offline' | 'unauthorized';
export type LogLevel = 'verbose' | 'debug' | 'info' | 'warn' | 'error' | 'fatal';
export type LogSource = 'system' | 'unity' | 'il2cpp' | 'mono';

export interface Device {
  serial: string;
  model: string;
  product: string;
  transport: TransportType;
  status: DeviceStatus;
}

export interface StackFrame {
  module: string;
  class_name: string;
  method_name: string;
  file: string | null;
  line: number | null;
}

export interface ScriptInfo {
  file: string;
  line: number;
}

export interface LogEntry {
  id: number;
  timestamp: string;
  pid: number;
  tid: number;
  level: LogLevel;
  tag: string;
  message: string;
  source: LogSource;
  stack_frames: StackFrame[] | null;
  unity_script_info: ScriptInfo | null;
}

export interface InstallProgress {
  stage: 'parsing' | 'building' | 'installing' | 'complete' | 'failed';
  message: string;
}

export interface KeystoreConfig {
  path: string;
  alias: string;
  store_password: string;
  key_password: string;
}

export interface InstallRecord {
  id: string;
  filename: string;
  device_serial: string;
  device_model: string;
  timestamp: number;
  result: 'success' | 'failed';
  error?: string;
}

export type ViewMode = 'logcat' | 'installer';

export interface FilterPreset {
  name: string;
  tags: string[];
  levels: LogLevel[];
}

export const UNITY_FILTER_PRESETS: FilterPreset[] = [
  { name: 'Unity All', tags: ['Unity', 'Il2Cpp', 'Mono', 'CRASH'], levels: [] },
  { name: 'Unity Errors', tags: ['Unity', 'Il2Cpp', 'Mono', 'CRASH'], levels: ['error', 'fatal'] },
  { name: 'Unity Scripting', tags: ['Unity'], levels: [] },
  { name: 'Unity Rendering', tags: ['Unity'], levels: [] },
  { name: 'Unity Network', tags: ['Unity'], levels: [] },
];
```

- [ ] **Step 2: Create Tauri IPC wrappers**

Create `src/lib/utils/tauri.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Device, LogEntry, InstallProgress, KeystoreConfig } from '../types';

export async function getDevices(): Promise<Device[]> {
  return invoke('get_devices');
}

export async function connectWifi(addr: string): Promise<string> {
  return invoke('connect_wifi', { addr });
}

export async function pairDevice(addr: string, code: string): Promise<string> {
  return invoke('pair_device', { addr, code });
}

export async function disconnectDevice(serial: string): Promise<string> {
  return invoke('disconnect_device', { serial });
}

export async function startLogcat(serial: string): Promise<void> {
  return invoke('start_logcat', { serial });
}

export async function stopLogcat(serial: string): Promise<void> {
  return invoke('stop_logcat', { serial });
}

export async function pauseLogcat(serial: string): Promise<void> {
  return invoke('pause_logcat', { serial });
}

export async function resumeLogcat(serial: string): Promise<void> {
  return invoke('resume_logcat', { serial });
}

export async function clearLogcat(serial: string): Promise<void> {
  return invoke('clear_logcat', { serial });
}

export async function queryLogHistory(
  device: string,
  from?: string,
  to?: string,
  limit?: number
): Promise<LogEntry[]> {
  return invoke('query_log_history', { device, from, to, limit });
}

export async function installApk(serial: string, path: string): Promise<string> {
  return invoke('install_apk', { serial, path });
}

export async function installAab(
  serial: string,
  path: string,
  keystore?: KeystoreConfig
): Promise<string> {
  return invoke('install_aab', { serial, path, keystore });
}

export function onDevicesChanged(callback: (devices: Device[]) => void): Promise<UnlistenFn> {
  return listen<Device[]>('devices-changed', (event) => callback(event.payload));
}

export function onLogcat(serial: string, callback: (entries: LogEntry[]) => void): Promise<UnlistenFn> {
  return listen<LogEntry[]>(`logcat-${serial}`, (event) => callback(event.payload));
}

export function onInstallProgress(callback: (progress: InstallProgress) => void): Promise<UnlistenFn> {
  return listen<InstallProgress>('install-progress', (event) => callback(event.payload));
}
```

- [ ] **Step 3: Create format utilities**

Create `src/lib/utils/format.ts`:

```typescript
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export function formatTimestamp(ts: string): string {
  return ts;
}

export function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}
```

- [ ] **Step 4: Create device store**

Create `src/lib/stores/devices.ts`:

```typescript
import { writable, derived } from 'svelte/store';
import type { Device } from '../types';

export const devices = writable<Device[]>([]);
export const activeDeviceSerial = writable<string | null>(null);

export const activeDevice = derived(
  [devices, activeDeviceSerial],
  ([$devices, $serial]) => $devices.find(d => d.serial === $serial) ?? null
);

export const onlineDevices = derived(
  devices,
  ($devices) => $devices.filter(d => d.status === 'online')
);
```

- [ ] **Step 5: Create logcat store**

Create `src/lib/stores/logcat.ts`:

```typescript
import { writable, derived, get } from 'svelte/store';
import type { LogEntry, LogLevel, LogSource } from '../types';

const MAX_ENTRIES = 100_000;

export const logEntries = writable<Map<string, LogEntry[]>>(new Map());
export const filterLevel = writable<LogLevel | null>(null);
export const filterTags = writable<string[]>([]);
export const filterSearch = writable<string>('');
export const filterSearchRegex = writable<boolean>(false);
export const filterPid = writable<number | null>(null);
export const unityMode = writable<boolean>(false);
export const autoScroll = writable<boolean>(true);
export const isPaused = writable<boolean>(false);

const UNITY_TAGS = ['Unity', 'Il2Cpp', 'Mono', 'CRASH'];
const LOG_LEVEL_ORDER: LogLevel[] = ['verbose', 'debug', 'info', 'warn', 'error', 'fatal'];

export function addLogEntries(serial: string, entries: LogEntry[]) {
  logEntries.update(map => {
    const existing = map.get(serial) ?? [];
    const combined = existing.concat(entries);
    if (combined.length > MAX_ENTRIES) {
      map.set(serial, combined.slice(combined.length - MAX_ENTRIES));
    } else {
      map.set(serial, combined);
    }
    return new Map(map);
  });
}

export function clearLogEntries(serial: string) {
  logEntries.update(map => {
    map.set(serial, []);
    return new Map(map);
  });
}

export function getFilteredEntries(entries: LogEntry[]): LogEntry[] {
  const level = get(filterLevel);
  const tags = get(filterTags);
  const search = get(filterSearch);
  const isRegex = get(filterSearchRegex);
  const pid = get(filterPid);
  const isUnity = get(unityMode);

  let filtered = entries;

  if (isUnity) {
    filtered = filtered.filter(e => UNITY_TAGS.includes(e.tag));
  }

  if (level) {
    const minIndex = LOG_LEVEL_ORDER.indexOf(level);
    filtered = filtered.filter(e => LOG_LEVEL_ORDER.indexOf(e.level) >= minIndex);
  }

  if (tags.length > 0) {
    filtered = filtered.filter(e => tags.includes(e.tag));
  }

  if (pid !== null) {
    filtered = filtered.filter(e => e.pid === pid);
  }

  if (search) {
    if (isRegex) {
      try {
        const re = new RegExp(search, 'i');
        filtered = filtered.filter(e => re.test(e.message) || re.test(e.tag));
      } catch {
        // invalid regex, skip
      }
    } else {
      const lower = search.toLowerCase();
      filtered = filtered.filter(
        e => e.message.toLowerCase().includes(lower) || e.tag.toLowerCase().includes(lower)
      );
    }
  }

  return filtered;
}
```

- [ ] **Step 6: Create installer store**

Create `src/lib/stores/installer.ts`:

```typescript
import { writable } from 'svelte/store';
import type { InstallProgress, InstallRecord, KeystoreConfig } from '../types';
import { generateId } from '../utils/format';

export const installProgress = writable<InstallProgress | null>(null);
export const installHistory = writable<InstallRecord[]>([]);
export const savedKeystore = writable<KeystoreConfig | null>(null);

export function addInstallRecord(
  filename: string,
  deviceSerial: string,
  deviceModel: string,
  result: 'success' | 'failed',
  error?: string
) {
  installHistory.update(list => [{
    id: generateId(),
    filename,
    device_serial: deviceSerial,
    device_model: deviceModel,
    timestamp: Date.now(),
    result,
    error,
  }, ...list].slice(0, 100));
}
```

- [ ] **Step 7: Commit**

```bash
git add src/lib/
git commit -m "feat: add TypeScript types, Tauri IPC wrappers, and Svelte stores"
```

---

## Task 10: Frontend — UI Components

**Files:**
- Create: `src/lib/components/ui/Button.svelte`
- Create: `src/lib/components/ui/Input.svelte`
- Create: `src/lib/components/ui/Select.svelte`
- Create: `src/lib/components/ui/Dialog.svelte`
- Create: `src/lib/components/ui/Tabs.svelte`
- Create: `src/lib/components/ui/ProgressBar.svelte`

- [ ] **Step 1: Create Button component**

Create `src/lib/components/ui/Button.svelte`:

```svelte
<script lang="ts">
  export let variant: 'primary' | 'secondary' | 'ghost' | 'danger' = 'secondary';
  export let size: 'sm' | 'md' = 'md';
  export let disabled: boolean = false;
  export let active: boolean = false;
</script>

<button
  class="btn btn-{variant} btn-{size}"
  class:active
  {disabled}
  on:click
>
  <slot />
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-sm { padding: 2px 8px; font-size: 12px; }
  .btn-md { padding: 4px 12px; font-size: 13px; }
  .btn-primary {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  .btn-primary:hover:not(:disabled) { background: var(--accent-hover); }
  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
  .btn-secondary:hover:not(:disabled) { background: var(--bg-hover); }
  .btn-ghost {
    background: transparent;
    color: var(--text-primary);
    border-color: transparent;
  }
  .btn-ghost:hover:not(:disabled) { background: var(--bg-hover); }
  .btn-danger {
    background: var(--error);
    color: white;
    border-color: var(--error);
  }
  .btn-danger:hover:not(:disabled) { opacity: 0.9; }
  .active {
    background: var(--accent) !important;
    color: white !important;
    border-color: var(--accent) !important;
  }
</style>
```

- [ ] **Step 2: Create Input component**

Create `src/lib/components/ui/Input.svelte`:

```svelte
<script lang="ts">
  export let value: string = '';
  export let placeholder: string = '';
  export let type: string = 'text';
  export let size: 'sm' | 'md' = 'md';
</script>

<input
  class="input input-{size}"
  {type}
  {placeholder}
  bind:value
  on:input
  on:keydown
/>

<style>
  .input {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    outline: none;
    transition: border-color 0.15s;
    width: 100%;
  }
  .input:focus { border-color: var(--accent); }
  .input::placeholder { color: var(--text-secondary); }
  .input-sm { padding: 2px 8px; font-size: 12px; }
  .input-md { padding: 4px 10px; font-size: 13px; }
</style>
```

- [ ] **Step 3: Create Select component**

Create `src/lib/components/ui/Select.svelte`:

```svelte
<script lang="ts">
  export let value: string = '';
  export let options: { value: string; label: string }[] = [];
  export let placeholder: string = '';
</script>

<select class="select" bind:value on:change>
  {#if placeholder}
    <option value="" disabled>{placeholder}</option>
  {/if}
  {#each options as opt}
    <option value={opt.value}>{opt.label}</option>
  {/each}
</select>

<style>
  .select {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 4px 8px;
    font-family: var(--font-ui);
    font-size: 13px;
    outline: none;
    cursor: pointer;
  }
  .select:focus { border-color: var(--accent); }
</style>
```

- [ ] **Step 4: Create Dialog component**

Create `src/lib/components/ui/Dialog.svelte`:

```svelte
<script lang="ts">
  export let open: boolean = false;
  export let title: string = '';

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) open = false;
  }
</script>

{#if open}
  <div class="backdrop" on:click={handleBackdrop} role="presentation">
    <div class="dialog">
      <div class="dialog-header">
        <span class="dialog-title">{title}</span>
        <button class="close-btn" on:click={() => open = false}>✕</button>
      </div>
      <div class="dialog-body">
        <slot />
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    min-width: 400px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
  }
  .dialog-title { font-weight: 600; color: var(--text-bright); }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 16px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .dialog-body { padding: 16px; }
</style>
```

- [ ] **Step 5: Create Tabs component**

Create `src/lib/components/ui/Tabs.svelte`:

```svelte
<script lang="ts">
  export let tabs: { id: string; label: string }[] = [];
  export let activeTab: string = '';
</script>

<div class="tabs">
  {#each tabs as tab}
    <button
      class="tab"
      class:active={activeTab === tab.id}
      on:click={() => activeTab = tab.id}
    >
      {tab.label}
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border-color);
  }
  .tab {
    padding: 6px 16px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-family: var(--font-ui);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .tab:hover { color: var(--text-primary); }
  .tab.active {
    color: var(--text-bright);
    border-bottom-color: var(--accent);
  }
</style>
```

- [ ] **Step 6: Create ProgressBar component**

Create `src/lib/components/ui/ProgressBar.svelte`:

```svelte
<script lang="ts">
  export let progress: number = 0;
  export let indeterminate: boolean = false;
</script>

<div class="progress-track">
  {#if indeterminate}
    <div class="progress-bar indeterminate" />
  {:else}
    <div class="progress-bar" style="width: {Math.min(100, Math.max(0, progress))}%" />
  {/if}
</div>

<style>
  .progress-track {
    height: 4px;
    background: var(--bg-hover);
    border-radius: 2px;
    overflow: hidden;
  }
  .progress-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s ease;
  }
  .indeterminate {
    width: 40%;
    animation: slide 1.5s ease-in-out infinite;
  }
  @keyframes slide {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(350%); }
  }
</style>
```

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/ui/
git commit -m "feat: add reusable UI components (Button, Input, Select, Dialog, Tabs, ProgressBar)"
```

---

## Task 11: Frontend — Device Panel and Toolbar

**Files:**
- Create: `src/lib/components/DevicePanel.svelte`
- Create: `src/lib/components/WifiDialog.svelte`
- Create: `src/lib/components/Toolbar.svelte`
- Create: `src/lib/components/StatusBar.svelte`

- [ ] **Step 1: Create DevicePanel**

Create `src/lib/components/DevicePanel.svelte`:

```svelte
<script lang="ts">
  import { devices, activeDeviceSerial, onlineDevices } from '../stores/devices';
  import type { Device } from '../types';
  import Button from './ui/Button.svelte';

  export let onWifiClick: () => void = () => {};

  function selectDevice(serial: string) {
    $activeDeviceSerial = serial;
  }

  function transportIcon(d: Device): string {
    return d.transport === 'usb' ? '🔌' : '📶';
  }

  function statusColor(d: Device): string {
    switch (d.status) {
      case 'online': return 'var(--success)';
      case 'offline': return 'var(--text-secondary)';
      case 'unauthorized': return 'var(--warning)';
    }
  }
</script>

<div class="device-panel">
  <div class="panel-header">
    <span class="panel-title">Devices</span>
    <span class="device-count">{$onlineDevices.length}</span>
  </div>

  <div class="device-list">
    {#each $devices as device (device.serial)}
      <button
        class="device-item"
        class:active={$activeDeviceSerial === device.serial}
        on:click={() => selectDevice(device.serial)}
      >
        <span class="status-dot" style="background: {statusColor(device)}" />
        <div class="device-info">
          <span class="device-model">{device.model || device.serial}</span>
          <span class="device-serial">{transportIcon(device)} {device.serial}</span>
        </div>
      </button>
    {:else}
      <div class="empty-state">No devices connected</div>
    {/each}
  </div>

  <div class="panel-footer">
    <Button size="sm" on:click={onWifiClick}>WiFi Connect</Button>
  </div>
</div>

<style>
  .device-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-color);
  }
  .panel-title {
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }
  .device-count {
    background: var(--accent);
    color: white;
    border-radius: 10px;
    padding: 0 6px;
    font-size: 11px;
    min-width: 18px;
    text-align: center;
  }
  .device-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .device-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    font-family: var(--font-ui);
    transition: background 0.1s;
  }
  .device-item:hover { background: var(--bg-hover); }
  .device-item.active { background: var(--bg-tertiary); border-left: 2px solid var(--accent); }
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .device-info { display: flex; flex-direction: column; min-width: 0; }
  .device-model {
    font-size: 13px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .device-serial {
    font-size: 11px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty-state {
    padding: 20px 12px;
    text-align: center;
    color: var(--text-secondary);
    font-size: 12px;
  }
  .panel-footer {
    padding: 8px 12px;
    border-top: 1px solid var(--border-color);
  }
</style>
```

- [ ] **Step 2: Create WifiDialog**

Create `src/lib/components/WifiDialog.svelte`:

```svelte
<script lang="ts">
  import Dialog from './ui/Dialog.svelte';
  import Input from './ui/Input.svelte';
  import Button from './ui/Button.svelte';
  import { connectWifi, pairDevice } from '../utils/tauri';

  export let open: boolean = false;

  let mode: 'connect' | 'pair' = 'connect';
  let addr: string = '';
  let pairCode: string = '';
  let status: string = '';
  let loading: boolean = false;

  async function handleConnect() {
    loading = true;
    status = '';
    try {
      const result = await connectWifi(addr);
      status = result;
      if (result.includes('connected')) {
        setTimeout(() => { open = false; }, 1000);
      }
    } catch (e) {
      status = `Failed: ${e}`;
    }
    loading = false;
  }

  async function handlePair() {
    loading = true;
    status = '';
    try {
      const result = await pairDevice(addr, pairCode);
      status = result;
    } catch (e) {
      status = `Failed: ${e}`;
    }
    loading = false;
  }
</script>

<Dialog bind:open title="WiFi Connection">
  <div class="wifi-form">
    <div class="mode-switch">
      <Button size="sm" active={mode === 'connect'} on:click={() => mode = 'connect'}>Connect</Button>
      <Button size="sm" active={mode === 'pair'} on:click={() => mode = 'pair'}>Pair</Button>
    </div>

    <label class="field">
      <span class="label">{mode === 'pair' ? 'Pair Address' : 'Device Address'}</span>
      <Input bind:value={addr} placeholder="192.168.1.100:5555" />
    </label>

    {#if mode === 'pair'}
      <label class="field">
        <span class="label">Pairing Code</span>
        <Input bind:value={pairCode} placeholder="123456" />
      </label>
    {/if}

    {#if status}
      <div class="status" class:error={status.includes('Failed')}>{status}</div>
    {/if}

    <Button
      variant="primary"
      disabled={loading || !addr}
      on:click={mode === 'pair' ? handlePair : handleConnect}
    >
      {loading ? 'Connecting...' : mode === 'pair' ? 'Pair' : 'Connect'}
    </Button>
  </div>
</Dialog>

<style>
  .wifi-form { display: flex; flex-direction: column; gap: 12px; }
  .mode-switch { display: flex; gap: 4px; }
  .field { display: flex; flex-direction: column; gap: 4px; }
  .label { font-size: 12px; color: var(--text-secondary); }
  .status {
    font-size: 12px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--success);
  }
  .status.error { color: var(--error); }
</style>
```

- [ ] **Step 3: Create Toolbar**

Create `src/lib/components/Toolbar.svelte`:

```svelte
<script lang="ts">
  import Button from './ui/Button.svelte';
  import type { ViewMode } from '../types';

  export let currentView: ViewMode = 'logcat';
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <Button size="sm" active={currentView === 'logcat'} on:click={() => currentView = 'logcat'}>
      Logcat
    </Button>
    <Button size="sm" active={currentView === 'installer'} on:click={() => currentView = 'installer'}>
      Installer
    </Button>
  </div>
  <div class="toolbar-center">
    <span class="app-title">AndroidQTools</span>
  </div>
  <div class="toolbar-right">
    <slot />
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    height: var(--toolbar-height);
    padding: 0 12px;
    gap: 8px;
  }
  .toolbar-left { display: flex; gap: 4px; }
  .toolbar-center { flex: 1; text-align: center; }
  .app-title { font-size: 12px; color: var(--text-secondary); }
  .toolbar-right { display: flex; gap: 8px; align-items: center; }
</style>
```

- [ ] **Step 4: Create StatusBar**

Create `src/lib/components/StatusBar.svelte`:

```svelte
<script lang="ts">
  import { onlineDevices } from '../stores/devices';
</script>

<div class="statusbar">
  <span>{$onlineDevices.length} device{$onlineDevices.length !== 1 ? 's' : ''} connected</span>
  <span class="separator">|</span>
  <span>ADB Ready</span>
</div>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }
  .separator { color: rgba(255, 255, 255, 0.3); }
</style>
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/DevicePanel.svelte src/lib/components/WifiDialog.svelte src/lib/components/Toolbar.svelte src/lib/components/StatusBar.svelte
git commit -m "feat: add DevicePanel, WifiDialog, Toolbar, and StatusBar components"
```

---

## Task 12: Frontend — Logcat Viewer

**Files:**
- Create: `src/lib/components/LogcatViewer.svelte`
- Create: `src/lib/components/LogcatFilterBar.svelte`
- Create: `src/lib/components/LogcatRow.svelte`
- Create: `src/lib/components/StackGroup.svelte`

- [ ] **Step 1: Create LogcatRow**

Create `src/lib/components/LogcatRow.svelte`:

```svelte
<script lang="ts">
  import type { LogEntry } from '../types';

  export let entry: LogEntry;

  const levelColors: Record<string, string> = {
    verbose: 'var(--log-verbose)',
    debug: 'var(--log-debug)',
    info: 'var(--log-info)',
    warn: 'var(--log-warn)',
    error: 'var(--log-error)',
    fatal: 'var(--log-fatal)',
  };

  const levelLetters: Record<string, string> = {
    verbose: 'V', debug: 'D', info: 'I', warn: 'W', error: 'E', fatal: 'F',
  };

  function sourceIcon(source: string): string {
    switch (source) {
      case 'unity': return 'U';
      case 'il2cpp': return 'IL';
      case 'mono': return 'M';
      default: return '';
    }
  }
</script>

<div class="log-row" style="color: {levelColors[entry.level]}">
  <span class="timestamp">{entry.timestamp}</span>
  <span class="pid">{entry.pid}</span>
  <span class="level">{levelLetters[entry.level]}</span>
  {#if entry.source !== 'system'}
    <span class="source-badge">{sourceIcon(entry.source)}</span>
  {/if}
  <span class="tag">{entry.tag}</span>
  <span class="message">{entry.message}</span>
</div>

<style>
  .log-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 1px 12px;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.6;
    white-space: nowrap;
  }
  .log-row:hover { background: var(--bg-tertiary); }
  .timestamp { color: var(--text-secondary); min-width: 140px; }
  .pid { color: var(--text-secondary); min-width: 50px; text-align: right; }
  .level { min-width: 12px; font-weight: 700; }
  .source-badge {
    background: var(--accent);
    color: white;
    border-radius: 2px;
    padding: 0 3px;
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-ui);
  }
  .tag {
    color: var(--accent);
    min-width: 100px;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
```

- [ ] **Step 2: Create StackGroup**

Create `src/lib/components/StackGroup.svelte`:

```svelte
<script lang="ts">
  import type { StackFrame } from '../types';

  export let frames: StackFrame[];
  let collapsed: boolean = true;
</script>

<div class="stack-group">
  <button class="stack-toggle" on:click={() => collapsed = !collapsed}>
    {collapsed ? '▶' : '▼'} Stack Trace ({frames.length} frames)
  </button>
  {#if !collapsed}
    <div class="stack-frames">
      {#each frames as frame}
        <div class="frame">
          <span class="frame-at">at </span>
          {#if frame.module}
            <span class="frame-module">{frame.module}.</span>
          {/if}
          <span class="frame-class">{frame.class_name}</span>
          <span class="frame-dot">.</span>
          <span class="frame-method">{frame.method_name}</span>
          <span class="frame-parens">()</span>
          {#if frame.file}
            <span class="frame-file"> in {frame.file}:{frame.line ?? '?'}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .stack-group { padding: 2px 12px 2px 24px; }
  .stack-toggle {
    background: none;
    border: none;
    color: var(--log-error);
    font-family: var(--font-mono);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 0;
  }
  .stack-toggle:hover { text-decoration: underline; }
  .stack-frames { padding-left: 16px; }
  .frame {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.6;
  }
  .frame-at { color: var(--text-secondary); }
  .frame-module { color: var(--log-verbose); }
  .frame-class { color: var(--log-info); font-weight: 600; }
  .frame-dot { color: var(--text-secondary); }
  .frame-method { color: var(--log-warn); font-weight: 600; }
  .frame-parens { color: var(--text-secondary); }
  .frame-file { color: var(--log-debug); }
</style>
```

- [ ] **Step 3: Create LogcatFilterBar**

Create `src/lib/components/LogcatFilterBar.svelte`:

```svelte
<script lang="ts">
  import Button from './ui/Button.svelte';
  import Input from './ui/Input.svelte';
  import Select from './ui/Select.svelte';
  import { filterLevel, filterSearch, filterSearchRegex, unityMode, autoScroll, isPaused } from '../stores/logcat';
  import { UNITY_FILTER_PRESETS } from '../types';
  import type { LogLevel } from '../types';

  export let onClear: () => void = () => {};
  export let onExport: () => void = () => {};
  export let onPauseToggle: () => void = () => {};

  const levelOptions = [
    { value: '', label: 'All Levels' },
    { value: 'verbose', label: 'Verbose' },
    { value: 'debug', label: 'Debug' },
    { value: 'info', label: 'Info' },
    { value: 'warn', label: 'Warn' },
    { value: 'error', label: 'Error' },
    { value: 'fatal', label: 'Fatal' },
  ];

  let levelValue = '';
  $: $filterLevel = (levelValue || null) as LogLevel | null;
</script>

<div class="filter-bar">
  <div class="filter-group">
    <Select options={levelOptions} bind:value={levelValue} />

    <div class="search-wrapper">
      <Input
        size="sm"
        placeholder="Search / Regex..."
        bind:value={$filterSearch}
      />
      <Button size="sm" variant="ghost" active={$filterSearchRegex} on:click={() => $filterSearchRegex = !$filterSearchRegex}>
        .*
      </Button>
    </div>

    <Button size="sm" active={$unityMode} on:click={() => $unityMode = !$unityMode}>
      Unity
    </Button>
  </div>

  <div class="filter-actions">
    <Button size="sm" variant="ghost" on:click={onPauseToggle}>
      {$isPaused ? '▶ Resume' : '⏸ Pause'}
    </Button>
    <Button size="sm" variant="ghost" active={$autoScroll} on:click={() => $autoScroll = !$autoScroll}>
      Auto-scroll
    </Button>
    <Button size="sm" variant="ghost" on:click={onClear}>Clear</Button>
    <Button size="sm" variant="ghost" on:click={onExport}>Export</Button>
  </div>
</div>

<style>
  .filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    gap: 8px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }
  .filter-group { display: flex; align-items: center; gap: 6px; }
  .search-wrapper { display: flex; align-items: center; gap: 2px; width: 250px; }
  .filter-actions { display: flex; align-items: center; gap: 4px; }
</style>
```

- [ ] **Step 4: Create LogcatViewer with virtual scrolling**

Create `src/lib/components/LogcatViewer.svelte`:

```svelte
<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import LogcatRow from './LogcatRow.svelte';
  import LogcatFilterBar from './LogcatFilterBar.svelte';
  import StackGroup from './StackGroup.svelte';
  import Tabs from './ui/Tabs.svelte';
  import { logEntries, addLogEntries, clearLogEntries, getFilteredEntries, autoScroll, isPaused } from '../stores/logcat';
  import { devices, activeDeviceSerial } from '../stores/devices';
  import { startLogcat, stopLogcat, pauseLogcat, resumeLogcat, clearLogcat, onLogcat } from '../utils/tauri';
  import type { LogEntry } from '../types';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let container: HTMLDivElement;
  let unlisteners: Map<string, UnlistenFn> = new Map();
  const ROW_HEIGHT = 20;
  let scrollTop = 0;
  let containerHeight = 0;

  $: deviceTabs = $devices
    .filter(d => d.status === 'online')
    .map(d => ({ id: d.serial, label: d.model || d.serial }));

  $: currentSerial = $activeDeviceSerial ?? '';
  $: allEntries = $logEntries.get(currentSerial) ?? [];
  $: filtered = getFilteredEntries(allEntries);
  $: totalHeight = filtered.length * ROW_HEIGHT;
  $: startIndex = Math.floor(scrollTop / ROW_HEIGHT);
  $: visibleCount = Math.ceil(containerHeight / ROW_HEIGHT) + 2;
  $: visibleEntries = filtered.slice(startIndex, startIndex + visibleCount);

  async function subscribeLogcat(serial: string) {
    if (unlisteners.has(serial)) return;
    try {
      await startLogcat(serial);
      const unlisten = await onLogcat(serial, (entries) => {
        addLogEntries(serial, entries);
        if ($autoScroll && container) {
          tick().then(() => {
            container.scrollTop = container.scrollHeight;
          });
        }
      });
      unlisteners.set(serial, unlisten);
    } catch (e) {
      console.error(`Failed to start logcat for ${serial}:`, e);
    }
  }

  function handleScroll() {
    if (container) {
      scrollTop = container.scrollTop;
    }
  }

  function handleTabChange(serial: string) {
    $activeDeviceSerial = serial;
    subscribeLogcat(serial);
  }

  async function handlePauseToggle() {
    if (!currentSerial) return;
    if ($isPaused) {
      await resumeLogcat(currentSerial);
      $isPaused = false;
    } else {
      await pauseLogcat(currentSerial);
      $isPaused = true;
    }
  }

  async function handleClear() {
    if (!currentSerial) return;
    await clearLogcat(currentSerial);
    clearLogEntries(currentSerial);
  }

  function handleExport() {
    const lines = filtered.map(e =>
      `${e.timestamp} ${e.pid} ${e.tid} ${e.level.charAt(0).toUpperCase()} ${e.tag}: ${e.message}`
    );
    const blob = new Blob([lines.join('\n')], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `logcat-${currentSerial}-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }

  $: if ($activeDeviceSerial) {
    subscribeLogcat($activeDeviceSerial);
  }

  onMount(() => {
    if (container) {
      containerHeight = container.clientHeight;
      const ro = new ResizeObserver(() => {
        containerHeight = container.clientHeight;
      });
      ro.observe(container);
      return () => ro.disconnect();
    }
  });

  onDestroy(() => {
    unlisteners.forEach((unlisten, serial) => {
      unlisten();
      stopLogcat(serial);
    });
  });
</script>

<div class="logcat-viewer">
  {#if deviceTabs.length > 1}
    <Tabs tabs={deviceTabs} activeTab={currentSerial} on:change={(e) => handleTabChange(e.detail)} />
  {/if}

  <LogcatFilterBar
    onClear={handleClear}
    onExport={handleExport}
    onPauseToggle={handlePauseToggle}
  />

  <div
    class="log-container"
    bind:this={container}
    on:scroll={handleScroll}
  >
    <div class="log-scroll-spacer" style="height: {totalHeight}px">
      <div class="log-visible" style="transform: translateY({startIndex * ROW_HEIGHT}px)">
        {#each visibleEntries as entry (entry.id)}
          <LogcatRow {entry} />
          {#if entry.stack_frames && entry.stack_frames.length > 0}
            <StackGroup frames={entry.stack_frames} />
          {/if}
        {/each}
      </div>
    </div>
  </div>

  {#if filtered.length === 0 && allEntries.length > 0}
    <div class="empty-filter">No entries match current filters</div>
  {:else if allEntries.length === 0}
    <div class="empty-filter">
      {currentSerial ? 'Waiting for logcat data...' : 'Select a device to start'}
    </div>
  {/if}
</div>

<style>
  .logcat-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .log-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--bg-primary);
  }
  .log-scroll-spacer { position: relative; }
  .log-visible { position: absolute; top: 0; left: 0; right: 0; }
  .empty-filter {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: var(--text-secondary);
    font-size: 14px;
  }
</style>
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/LogcatViewer.svelte src/lib/components/LogcatFilterBar.svelte src/lib/components/LogcatRow.svelte src/lib/components/StackGroup.svelte
git commit -m "feat: add LogcatViewer with virtual scrolling, filtering, and Unity mode"
```

---

## Task 13: Frontend — Installer View

**Files:**
- Create: `src/lib/components/Installer.svelte`
- Create: `src/lib/components/AppPreview.svelte`
- Create: `src/lib/components/InstallHistory.svelte`

- [ ] **Step 1: Create AppPreview**

Create `src/lib/components/AppPreview.svelte`:

```svelte
<script lang="ts">
  import { formatFileSize } from '../utils/format';

  export let filename: string = '';
  export let filesize: number = 0;
  export let filetype: 'apk' | 'aab' = 'apk';
</script>

<div class="app-preview">
  <div class="app-icon">{filetype === 'apk' ? '📦' : '📋'}</div>
  <div class="app-info">
    <span class="app-name">{filename}</span>
    <span class="app-meta">{filetype.toUpperCase()} · {formatFileSize(filesize)}</span>
  </div>
</div>

<style>
  .app-preview {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
  }
  .app-icon { font-size: 32px; }
  .app-info { display: flex; flex-direction: column; gap: 2px; }
  .app-name { font-weight: 600; color: var(--text-bright); font-size: 14px; }
  .app-meta { font-size: 12px; color: var(--text-secondary); }
</style>
```

- [ ] **Step 2: Create InstallHistory**

Create `src/lib/components/InstallHistory.svelte`:

```svelte
<script lang="ts">
  import { installHistory } from '../stores/installer';

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleString();
  }
</script>

<div class="history">
  <div class="history-header">Install History</div>
  <div class="history-list">
    {#each $installHistory as record (record.id)}
      <div class="history-item">
        <span class="result-dot" class:success={record.result === 'success'} class:failed={record.result === 'failed'} />
        <div class="history-info">
          <span class="history-name">{record.filename}</span>
          <span class="history-meta">{record.device_model} · {formatTime(record.timestamp)}</span>
          {#if record.error}
            <span class="history-error">{record.error}</span>
          {/if}
        </div>
      </div>
    {:else}
      <div class="empty">No install history</div>
    {/each}
  </div>
</div>

<style>
  .history { display: flex; flex-direction: column; height: 100%; }
  .history-header {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    padding: 8px 0;
  }
  .history-list { flex: 1; overflow-y: auto; }
  .history-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-color);
  }
  .result-dot {
    width: 8px; height: 8px; border-radius: 50%;
    margin-top: 4px; flex-shrink: 0;
  }
  .result-dot.success { background: var(--success); }
  .result-dot.failed { background: var(--error); }
  .history-info { display: flex; flex-direction: column; gap: 2px; }
  .history-name { font-size: 13px; color: var(--text-primary); }
  .history-meta { font-size: 11px; color: var(--text-secondary); }
  .history-error { font-size: 11px; color: var(--error); }
  .empty { padding: 20px 0; text-align: center; color: var(--text-secondary); font-size: 12px; }
</style>
```

- [ ] **Step 3: Create Installer main view**

Create `src/lib/components/Installer.svelte`:

```svelte
<script lang="ts">
  import AppPreview from './AppPreview.svelte';
  import InstallHistory from './InstallHistory.svelte';
  import Button from './ui/Button.svelte';
  import Input from './ui/Input.svelte';
  import ProgressBar from './ui/ProgressBar.svelte';
  import { activeDevice, activeDeviceSerial } from '../stores/devices';
  import { installProgress, addInstallRecord, savedKeystore } from '../stores/installer';
  import { installApk, installAab, onInstallProgress } from '../utils/tauri';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { KeystoreConfig } from '../types';
  import { onMount, onDestroy } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let selectedFile: { name: string; path: string; size: number; type: 'apk' | 'aab' } | null = null;
  let isDragging = false;
  let installing = false;
  let showKeystoreConfig = false;
  let unlisten: UnlistenFn | null = null;

  let keystoreForm: KeystoreConfig = {
    path: '',
    alias: '',
    store_password: '',
    key_password: '',
  };

  onMount(async () => {
    unlisten = await onInstallProgress((progress) => {
      $installProgress = progress;
      if (progress.stage === 'complete' || progress.stage === 'failed') {
        installing = false;
        if (selectedFile && $activeDevice) {
          addInstallRecord(
            selectedFile.name,
            $activeDevice.serial,
            $activeDevice.model,
            progress.stage === 'complete' ? 'success' : 'failed',
            progress.stage === 'failed' ? progress.message : undefined
          );
        }
      }
    });

    if ($savedKeystore) {
      keystoreForm = { ...$savedKeystore };
    }
  });

  onDestroy(() => {
    unlisten?.();
  });

  async function selectFile() {
    const path = await open({
      filters: [{ name: 'Android Package', extensions: ['apk', 'aab'] }],
    });
    if (path) {
      const name = path.split(/[/\\]/).pop() ?? path;
      const type = name.endsWith('.aab') ? 'aab' : 'apk';
      selectedFile = { name, path, size: 0, type };
    }
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    const file = e.dataTransfer?.files[0];
    if (file) {
      const name = file.name;
      const type = name.endsWith('.aab') ? 'aab' : 'apk';
      selectedFile = { name, path: (file as any).path ?? name, size: file.size, type };
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  async function handleInstall() {
    if (!selectedFile || !$activeDeviceSerial) return;
    installing = true;
    $installProgress = { stage: 'installing', message: 'Starting...' };

    try {
      if (selectedFile.type === 'apk') {
        await installApk($activeDeviceSerial, selectedFile.path);
      } else {
        const ks = keystoreForm.path ? keystoreForm : undefined;
        if (ks) $savedKeystore = { ...keystoreForm };
        await installAab($activeDeviceSerial, selectedFile.path, ks);
      }
    } catch (e) {
      $installProgress = { stage: 'failed', message: String(e) };
      installing = false;
    }
  }
</script>

<div class="installer">
  <div class="install-area">
    <div
      class="drop-zone"
      class:dragging={isDragging}
      on:drop={handleDrop}
      on:dragover={handleDragOver}
      on:dragleave={() => isDragging = false}
      role="button"
      tabindex="0"
    >
      {#if selectedFile}
        <AppPreview
          filename={selectedFile.name}
          filesize={selectedFile.size}
          filetype={selectedFile.type}
        />
      {:else}
        <div class="drop-hint">
          <span class="drop-icon">📱</span>
          <span>Drag APK/AAB here or click to browse</span>
        </div>
      {/if}
    </div>

    <div class="install-controls">
      <Button variant="secondary" on:click={selectFile}>Browse Files</Button>

      {#if selectedFile?.type === 'aab'}
        <Button size="sm" variant="ghost" on:click={() => showKeystoreConfig = !showKeystoreConfig}>
          Keystore Config {showKeystoreConfig ? '▲' : '▼'}
        </Button>
      {/if}

      <Button
        variant="primary"
        disabled={!selectedFile || !$activeDeviceSerial || installing}
        on:click={handleInstall}
      >
        {installing ? 'Installing...' : 'Install'}
      </Button>
    </div>

    {#if showKeystoreConfig && selectedFile?.type === 'aab'}
      <div class="keystore-config">
        <Input bind:value={keystoreForm.path} placeholder="Keystore path" size="sm" />
        <Input bind:value={keystoreForm.alias} placeholder="Key alias" size="sm" />
        <Input bind:value={keystoreForm.store_password} placeholder="Store password" type="password" size="sm" />
        <Input bind:value={keystoreForm.key_password} placeholder="Key password" type="password" size="sm" />
      </div>
    {/if}

    {#if $installProgress}
      <div class="progress-section">
        <ProgressBar indeterminate={installing} />
        <span class="progress-text" class:error={$installProgress.stage === 'failed'} class:success={$installProgress.stage === 'complete'}>
          {$installProgress.message}
        </span>
      </div>
    {/if}
  </div>

  <div class="history-section">
    <InstallHistory />
  </div>
</div>

<style>
  .installer {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 20px;
    gap: 20px;
  }
  .install-area {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .drop-zone {
    border: 2px dashed var(--border-color);
    border-radius: var(--radius-lg);
    padding: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    min-height: 120px;
  }
  .drop-zone:hover, .drop-zone.dragging {
    border-color: var(--accent);
    background: rgba(0, 122, 204, 0.05);
  }
  .drop-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: var(--text-secondary);
  }
  .drop-icon { font-size: 32px; }
  .install-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .keystore-config {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    padding: 12px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .progress-text { font-size: 12px; color: var(--text-secondary); }
  .progress-text.error { color: var(--error); }
  .progress-text.success { color: var(--success); }
  .history-section { flex: 1; min-height: 0; overflow: hidden; }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Installer.svelte src/lib/components/AppPreview.svelte src/lib/components/InstallHistory.svelte
git commit -m "feat: add Installer view with drag-drop, keystore config, and install history"
```

---

## Task 14: Wire Everything Together in App.svelte

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Update App.svelte to integrate all components**

Replace `src/App.svelte`:

```svelte
<script lang="ts">
  import './styles/global.css';
  import { onMount, onDestroy } from 'svelte';
  import DevicePanel from './lib/components/DevicePanel.svelte';
  import WifiDialog from './lib/components/WifiDialog.svelte';
  import Toolbar from './lib/components/Toolbar.svelte';
  import LogcatViewer from './lib/components/LogcatViewer.svelte';
  import Installer from './lib/components/Installer.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import { devices, activeDeviceSerial } from './lib/stores/devices';
  import { onDevicesChanged, getDevices } from './lib/utils/tauri';
  import type { ViewMode } from './lib/types';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let currentView: ViewMode = 'logcat';
  let showWifiDialog = false;
  let unlisten: UnlistenFn | null = null;

  onMount(async () => {
    try {
      const initial = await getDevices();
      $devices = initial;
      if (initial.length > 0 && !$activeDeviceSerial) {
        $activeDeviceSerial = initial[0].serial;
      }
    } catch (e) {
      console.error('Failed to get initial devices:', e);
    }

    unlisten = await onDevicesChanged((newDevices) => {
      $devices = newDevices;
      if (newDevices.length > 0 && !$activeDeviceSerial) {
        $activeDeviceSerial = newDevices[0].serial;
      }
      if ($activeDeviceSerial && !newDevices.find(d => d.serial === $activeDeviceSerial)) {
        $activeDeviceSerial = newDevices[0]?.serial ?? null;
      }
    });
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<main class="app-shell">
  <div class="sidebar">
    <DevicePanel onWifiClick={() => showWifiDialog = true} />
  </div>
  <div class="content">
    <div class="toolbar-area">
      <Toolbar bind:currentView />
    </div>
    <div class="main-area">
      {#if currentView === 'logcat'}
        <LogcatViewer />
      {:else}
        <Installer />
      {/if}
    </div>
    <div class="statusbar-area">
      <StatusBar />
    </div>
  </div>
</main>

<WifiDialog bind:open={showWifiDialog} />

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
  }
  .sidebar {
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
  }
  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .toolbar-area {
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  .main-area {
    flex: 1;
    overflow: hidden;
    position: relative;
  }
  .statusbar-area {
    height: var(--statusbar-height);
    background: var(--accent);
    color: white;
    display: flex;
    align-items: center;
    padding: 0 12px;
  }
</style>
```

- [ ] **Step 2: Run the app to verify layout**

```bash
cd /Users/children/Documents/Projects/AndroidQTools
npm run tauri dev
```

Expected: Window opens with dark theme, left sidebar (device panel), top toolbar with Logcat/Installer tabs, main area showing logcat viewer, bottom blue status bar. Clicking "Installer" tab switches to the installer view with drag-drop zone.

- [ ] **Step 3: Commit**

```bash
git add src/App.svelte
git commit -m "feat: wire all components together in App.svelte"
```

---

## Task 15: Final Integration and Verification

- [ ] **Step 1: Run all Rust tests**

```bash
cd /Users/children/Documents/Projects/AndroidQTools/src-tauri
cargo test
```

Expected: All tests PASS.

- [ ] **Step 2: Run TypeScript type check**

```bash
cd /Users/children/Documents/Projects/AndroidQTools
npx svelte-check --tsconfig ./tsconfig.json
```

Expected: No type errors.

- [ ] **Step 3: Fix any compilation or type errors found**

Resolve any issues from steps 1-2. Common fixes: missing imports, type mismatches, Svelte component prop warnings.

- [ ] **Step 4: Run the full app and test golden path**

```bash
npm run tauri dev
```

Test manually:
1. App opens with dark theme
2. If an Android device is connected via USB, it appears in the sidebar
3. Clicking device selects it (highlighted)
4. Logcat tab shows log entries streaming
5. Level filter works (select "Error" — only errors shown)
6. Unity mode button filters to Unity tags
7. Search box filters by keyword
8. Pause/Resume works
9. Switching to Installer tab shows drop zone
10. WiFi Connect dialog opens and closes

- [ ] **Step 5: Commit all remaining changes**

```bash
git add -A
git commit -m "feat: complete AndroidQTools integration — logcat viewer, installer, device management"
```

---

## Summary

| Task | Description | Commits |
|------|-------------|---------|
| 1 | Project scaffolding (Tauri 2 + Svelte + TS) | 1 |
| 2 | Embedded resource management | 1 |
| 3 | Device manager (Rust) | 1 |
| 4 | Logcat core parsing (Rust) | 1 |
| 5 | Unity log parser (Rust) | 1 |
| 6 | Logcat streaming engine (Rust) | 1 |
| 7 | SQLite persistence | 1 |
| 8 | Tauri commands + installer + main.rs | 1 |
| 9 | TypeScript types + stores | 1 |
| 10 | UI components | 1 |
| 11 | Device panel + toolbar + status bar | 1 |
| 12 | Logcat viewer (virtual scroll + filters) | 1 |
| 13 | Installer view | 1 |
| 14 | App.svelte integration | 1 |
| 15 | Final integration + verification | 1 |
