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
