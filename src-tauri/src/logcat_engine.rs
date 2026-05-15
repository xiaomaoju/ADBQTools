use serde::{Deserialize, Serialize};
use regex::Regex;
use std::sync::LazyLock;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
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
        source,
        stack_frames: None,
        unity_script_info: None,
    })
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
