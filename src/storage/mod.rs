use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct StorageLayout {
    pub root: PathBuf,
    pub catalog: PathBuf,
    pub wal: PathBuf,
    pub snapshots: PathBuf,
}

pub struct StorageManager;

impl StorageManager {
    pub fn prepare(root: &Path) -> Result<StorageLayout> {
        let layout = StorageLayout {
            root: root.to_path_buf(),
            catalog: root.join("catalog"),
            wal: root.join("wal"),
            snapshots: root.join("snapshots"),
        };

        fs::create_dir_all(&layout.catalog)?;
        fs::create_dir_all(&layout.wal)?;
        fs::create_dir_all(&layout.snapshots)?;

        Ok(layout)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn prepare_creates_expected_directories() {
        let root = unique_test_dir();
        let layout = StorageManager::prepare(&root).expect("storage layout should be created");

        assert!(layout.root.exists());
        assert!(layout.catalog.exists());
        assert!(layout.wal.exists());
        assert!(layout.snapshots.exists());

        fs::remove_dir_all(&root).expect("temporary test directory should be removed");
    }

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("aetherdb-storage-test-{nanos}"))
    }
}