use anyhow::Result;

use crate::core::config::AppConfig;
use crate::storage::{StorageLayout, StorageManager};

pub struct Engine {
    config: AppConfig,
    layout: StorageLayout,
}

impl Engine {
    pub fn open(config: AppConfig) -> Result<Self> {
        let layout = StorageManager::prepare(&config.data_dir)?;
        Ok(Self { config, layout })
    }

    pub fn layout(&self) -> &StorageLayout {
        &self.layout
    }

    pub fn describe(&self) -> String {
        format!(
            "AetherDB {}\nmode: foundation\ndata dir: {}\nwal dir: {}\nsnapshots dir: {}\nnext milestone: durable WAL + recovery tests",
            env!("CARGO_PKG_VERSION"),
            self.config.data_dir.display(),
            self.layout.wal.display(),
            self.layout.snapshots.display(),
        )
    }
}