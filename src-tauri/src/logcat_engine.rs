use serde::{Deserialize, Serialize};
use regex::Regex;
use std::sync::LazyLock;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncReadExt};
use tauri::{AppHandle, Emitter};
use crate::unity_parser;

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
    pub package_name: String,
    pub source: LogSource,
    pub stack_frames: Option<Vec<StackFrame>>,
    pub unity_script_info: Option<ScriptInfo>,
}

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
        package_name: String::new(),
        source,
        stack_frames: None,
        unity_script_info: None,
    })
}

/// Periodically resolves PID → package name via `adb shell ps`
#[derive(Clone)]
pub struct PidResolver {
    cache: Arc<Mutex<HashMap<u32, String>>>,
    adb_path: PathBuf,
    serial: String,
    running: Arc<AtomicBool>,
}

impl PidResolver {
    pub fn new(adb_path: PathBuf, serial: String, running: Arc<AtomicBool>) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            adb_path,
            serial,
            running,
        }
    }

    pub async fn refresh(&self) {
        let output = crate::util::create_command(&self.adb_path)
            .args(["-s", &self.serial, "shell", "ps", "-A", "-o", "PID,NAME"])
            .output()
            .await;
        if let Ok(output) = output {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut map = HashMap::new();
            for line in text.lines().skip(1) {
                let parts: Vec<&str> = line.trim().splitn(2, char::is_whitespace).collect();
                if parts.len() == 2 {
                    if let Ok(pid) = parts[0].trim().parse::<u32>() {
                        let name = parts[1].trim().to_string();
                        if !name.is_empty() {
                            map.insert(pid, name);
                        }
                    }
                }
            }
            *self.cache.lock().await = map;
        }
    }

    pub async fn resolve(&self, pid: u32) -> String {
        let cache = self.cache.lock().await;
        cache.get(&pid).cloned().unwrap_or_default()
    }

    pub fn start_background_refresh(self) {
        tokio::spawn(async move {
            while self.running.load(Ordering::Relaxed) {
                self.refresh().await;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }
}

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

        let pid_resolver = PidResolver::new(adb_path.clone(), serial.clone(), running.clone());
        pid_resolver.refresh().await;
        pid_resolver.clone().start_background_refresh();

        tokio::spawn(async move {
            let _ = app.emit("logcat-error", format!("[{}] Starting: adb -s {} logcat -v threadtime", serial, serial));

            let mut child = match crate::util::create_command(&adb_path)
                .args(["-s", &serial, "logcat", "-v", "threadtime"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    let _ = app.emit("logcat-error", format!("[{}] Failed to spawn: {}", serial, e));
                    return;
                }
            };

            let _ = app.emit("logcat-error", format!("[{}] Process spawned OK, waiting for data...", serial));

            // Spawn a task to read stderr and report errors
            let stderr = child.stderr.take().unwrap();
            let err_app = app.clone();
            let err_serial = serial.clone();
            tokio::spawn(async move {
                let mut err_reader = BufReader::new(stderr);
                let mut err_buf = Vec::new();
                loop {
                    err_buf.clear();
                    match err_reader.read_until(b'\n', &mut err_buf).await {
                        Ok(0) => break,
                        Ok(_) => {
                            let line = String::from_utf8_lossy(&err_buf).trim().to_string();
                            if !line.is_empty() {
                                let _ = err_app.emit("logcat-error", format!("[{}] stderr: {}", err_serial, line));
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout);
            let mut raw_buf: Vec<u8> = Vec::with_capacity(4096);
            let mut batch: Vec<LogEntry> = Vec::with_capacity(50);
            let mut last_flush = tokio::time::Instant::now();
            let mut start_time = tokio::time::Instant::now();
            let mut got_data = false;
            let mut retried = false;

            while running.load(Ordering::Relaxed) {
                raw_buf.clear();
                let timeout = tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    reader.read_until(b'\n', &mut raw_buf),
                );

                match timeout.await {
                    Ok(Ok(0)) => {
                        // EOF — process exited
                        let _ = app.emit("logcat-error", format!(
                            "[{}] Process exited (EOF). got_data={}, elapsed={}s",
                            serial, got_data, start_time.elapsed().as_secs()
                        ));
                        break;
                    }
                    Ok(Ok(_n)) => {
                        if !got_data {
                            let _ = app.emit("logcat-error", format!("[{}] First data received!", serial));
                        }
                        got_data = true;
                        // Use lossy conversion to handle non-UTF-8 bytes (e.g. GBK on Chinese devices)
                        let line = String::from_utf8_lossy(&raw_buf);
                        let line = line.trim_end_matches('\n').trim_end_matches('\r');
                        let id = id_counter.fetch_add(1, Ordering::Relaxed);
                        if let Some(mut entry) = parse_logcat_line(line, id) {
                            entry.package_name = pid_resolver.resolve(entry.pid).await;
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
                    Ok(Err(e)) => {
                        let _ = app.emit("logcat-error", format!("[{}] Read error: {}", serial, e));
                        break;
                    }
                    Err(_) => {
                        // Timeout — check if we've been waiting too long with no data
                        if !got_data && !retried && start_time.elapsed().as_secs() >= 8 {
                            retried = true;
                            let _ = app.emit("logcat-error", format!("[{}] No data after 8s, retrying...", serial));
                            let _ = child.kill().await;
                            match crate::util::create_command(&adb_path)
                                .args(["-s", &serial, "logcat", "-v", "threadtime"])
                                .stdout(std::process::Stdio::piped())
                                .stderr(std::process::Stdio::null())
                                .spawn()
                            {
                                Ok(mut new_child) => {
                                    let new_stdout = new_child.stdout.take().unwrap();
                                    reader = BufReader::new(new_stdout);
                                    child = new_child;
                                    got_data = false;
                                    start_time = tokio::time::Instant::now();
                                }
                                Err(e) => {
                                    let _ = app.emit("logcat-error", format!("[{}] Retry failed: {}", serial, e));
                                    break;
                                }
                            }
                            continue;
                        }
                    }
                }

                let should_flush = batch.len() >= 50
                    || (last_flush.elapsed().as_millis() >= 100 && !batch.is_empty());

                if should_flush {
                    let _ = app.emit("logcat-data", serde_json::json!({
                        "serial": &serial,
                        "entries": &batch,
                    }));
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
            let _ = app.emit("logcat-data", serde_json::json!({
                "serial": &self.serial,
                "entries": &entries,
            }));
        }
    }

    pub async fn clear_logcat(&self) -> Result<(), String> {
        crate::util::create_command(&self.adb_path)
            .args(["-s", &self.serial, "logcat", "-c"])
            .output()
            .await
            .map_err(|e| format!("logcat clear failed: {}", e))?;
        Ok(())
    }
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
