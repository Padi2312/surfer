use chrono::Local;

pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Logger
    }

    pub fn info(&self, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("{} INFO: {}", timestamp, message);
    }

    pub fn error(&self, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("{} ERROR: {}", timestamp, message);
    }
}