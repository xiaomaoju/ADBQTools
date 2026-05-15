use regex::Regex;
use std::sync::LazyLock;
use crate::logcat_engine::{StackFrame, ScriptInfo};

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
