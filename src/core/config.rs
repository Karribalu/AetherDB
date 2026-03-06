use std::path::PathBuf;

use clap::ValueEnum;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub log_format: LogFormat,
}

impl AppConfig {
    pub fn new(data_dir: PathBuf, log_format: LogFormat) -> Self {
        Self {
            data_dir,
            log_format,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogFormat {
    Text,
    Json,
}
