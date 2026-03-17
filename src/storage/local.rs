use crate::storage::payload::PutPayload;
use crate::storage::{Storage, StorageError, StorageResult};
use async_trait::async_trait;
use bytes::Bytes;
use futures::TryStreamExt;
use std::io::Write;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub struct LocalStorage {
    uri: String,
    root: PathBuf,
}
impl LocalStorage {
    fn full_path(&self, path: &Path) -> StorageResult<PathBuf> {
        ensure_valid_relative_path(path)?;
        Ok(self.root.join(path))
    }
}
#[async_trait]
impl Storage for LocalStorage {
    /**
     This function checks if the program is allowed to read/write in the root path
    **/
    async fn check_connectivity(&self) -> anyhow::Result<()> {
        // We are trying to create the directory to check if we are allowed
        if !self.root.try_exists()? {
            tokio::fs::create_dir_all(&self.root).await?
        }
        Ok(())
    }

    async fn put(&self, path: &Path, data: Box<dyn PutPayload>) -> StorageResult<()> {
        let full_path = self.full_path(path)?;
        let parent_dir = full_path.parent().ok_or_else(|| {
            StorageError::Internal(format!("no parent directory for ${full_path:?}"))
        })?;

        tokio::fs::create_dir_all(parent_dir).await?;
        let named_temp_file = tempfile::NamedTempFile::new_in(parent_dir)?;
        let mut stream = data.byte_stream().await?;
        let (mut temp_std_file, temp_file_path) = named_temp_file.into_parts();
        while let Some(chunk) = stream.try_next().await? {
            temp_std_file.write_all(&chunk)?;
        }

        temp_std_file.flush()?;
        temp_std_file.sync_all()?;
        drop(temp_std_file);

        temp_file_path
            .persist(&full_path)
            .map_err(|err| StorageError::Io(err.error.to_string()))?;

        Ok(())
    }

    async fn get_slice(&self, path: &Path, range: Range<usize>) -> StorageResult<Bytes> {
        todo!()
    }

    async fn get(&self, path: &Path) -> StorageResult<Bytes> {
        todo!()
    }
}

fn ensure_valid_relative_path(path: &Path) -> StorageResult<()> {
    for component in path.components() {
        match component {
            Component::RootDir | Component::ParentDir | Component::Prefix(_) => {
                return Err(StorageError::Unauthorized(format!(
                    "path: `{}` is forbidden. Only simple relative paths are allowed",
                    path.display()
                )));
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::payload::ByteStream;
    use futures::stream;
    use std::io;

    struct TestPayload {
        bytes: Bytes,
    }

    #[async_trait]
    impl PutPayload for TestPayload {
        fn len(&self) -> u64 {
            self.bytes.len() as u64
        }

        async fn range_byte_stream(&self, range: Range<u64>) -> io::Result<ByteStream> {
            let len = self.bytes.len() as u64;
            if range.start > range.end || range.end > len {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid byte range",
                ));
            }

            let chunk = self
                .bytes
                .slice((range.start as usize)..(range.end as usize));
            Ok(Box::pin(stream::iter(vec![Ok(chunk)])))
        }

        async fn byte_stream(&self) -> io::Result<ByteStream> {
            Ok(Box::pin(stream::iter(vec![Ok(self.bytes.clone())])))
        }
    }

    fn make_storage(root: PathBuf) -> LocalStorage {
        LocalStorage {
            uri: "file://local-test".to_string(),
            root,
        }
    }

    #[test]
    fn ensure_valid_relative_path_accepts_simple_relative_path() {
        let path = Path::new("segments/000001.seg");
        assert!(ensure_valid_relative_path(path).is_ok());
    }

    #[test]
    fn ensure_valid_relative_path_rejects_parent_dir_path() {
        let path = Path::new("../etc/passwd");
        let result = ensure_valid_relative_path(path);

        assert!(matches!(result, Err(StorageError::Unauthorized(_))));
    }

    #[test]
    fn ensure_valid_relative_path_rejects_absolute_path() {
        let path = Path::new("/tmp/segment.seg");
        let result = ensure_valid_relative_path(path);

        assert!(matches!(result, Err(StorageError::Unauthorized(_))));
    }

    #[tokio::test]
    async fn check_connectivity_creates_root_directory() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root.clone());

        storage
            .check_connectivity()
            .await
            .expect("connectivity check should create root directory");

        assert!(root.is_dir());
    }

    #[tokio::test]
    async fn put_writes_bytes_and_creates_parent_directories() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root.clone());

        let data = Bytes::from_static(b"hello aetherdb");
        let payload = TestPayload {
            bytes: data.clone(),
        };
        let relative_path = Path::new("segments/000001/docs.bin");

        storage
            .put(relative_path, Box::new(payload))
            .await
            .expect("put should succeed");

        let full_path = root.join(relative_path);
        let written = std::fs::read(full_path).expect("read written file");
        assert_eq!(written, data);
    }

    #[tokio::test]
    async fn put_rejects_forbidden_relative_path() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root);

        let payload = TestPayload {
            bytes: Bytes::from_static(b"test"),
        };

        let result = storage.put(Path::new("../escape"), Box::new(payload)).await;
        assert!(matches!(result, Err(StorageError::Unauthorized(_))));
    }
}
