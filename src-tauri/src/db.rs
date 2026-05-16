use rusqlite::{Connection, params};
use std::path::PathBuf;
use crate::logcat_engine::LogEntry;

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
                    package_name: String::new(),
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
            package_name: String::new(),
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
