use chrono::Utc;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub struct Logger {
    level: LogLevel,
    message_counter: AtomicU64,
}

#[derive(Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    module: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<serde_json::Value>,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            message_counter: AtomicU64::new(0),
        }
    }

    pub fn error<S: AsRef<str>>(&self, message: S, error: Option<&dyn std::error::Error>) {
        self.log(LogLevel::Error, message, error, None);
    }

    pub fn warn<S: AsRef<str>>(&self, message: S) {
        self.log(LogLevel::Warn, message, None, None);
    }

    pub fn info<S: AsRef<str>>(&self, message: S) {
        self.log(LogLevel::Info, message, None, None);
    }

    pub fn debug<S: AsRef<str>>(&self, message: S, context: Option<serde_json::Value>) {
        self.log(LogLevel::Debug, message, None, context);
    }

    pub fn trace<S: AsRef<str>>(&self, message: S, context: Option<serde_json::Value>) {
        self.log(LogLevel::Trace, message, None, context);
    }

    fn log<S: AsRef<str>>(
        &self,
        level: LogLevel,
        message: S,
        error: Option<&dyn std::error::Error>,
        context: Option<serde_json::Value>,
    ) {
        if level > self.level {
            return;
        }

        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: format!("{:?}", level),
            message: message.as_ref().to_string(),
            module: module_path!().to_string(),
            error: error.map(|e| e.to_string()),
            context,
        };

        // 增加消息计数
        self.message_counter.fetch_add(1, Ordering::Relaxed);

        // 序列化并输出日志
        if let Ok(json) = serde_json::to_string(&entry) {
            println!("{}", json);
        }
    }

    pub fn get_message_count(&self) -> u64 {
        self.message_counter.load(Ordering::Relaxed)
    }
}