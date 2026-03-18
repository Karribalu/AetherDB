use crate::storage::payload::PutPayload;
use crate::storage::{Storage, StorageError, StorageResult};
use async_trait::async_trait;
use bytes::Bytes;
use futures::TryStreamExt;
use std::io::{SeekFrom, Write};
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

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

    async fn get(&self, path: &Path) -> StorageResult<Bytes> {
        let full_path = self.full_path(path)?;
        let content = tokio::fs::read(full_path).await?;

        Ok(Bytes::from(content))
    }

    async fn get_slice(&self, path: &Path, range: Range<usize>) -> StorageResult<Bytes> {
        if range.start > range.end {
            return Err(StorageError::Io(format!(
                "invalid range: start {} is greater than end {}",
                range.start, range.end
            )));
        }

        let full_path = self.full_path(path)?;
        let mut file = tokio::fs::File::open(full_path).await?;
        file.seek(SeekFrom::Start(range.start as u64)).await?;
        let mut content_bytes = vec![0u8; range.len()];
        file.read_exact(&mut content_bytes).await?;

        Ok(Bytes::from(content_bytes))
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

    #[tokio::test]
    async fn get_reads_back_existing_file() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root.clone());

        let relative_path = Path::new("segments/000001/full.bin");
        let expected = Bytes::from_static(b"full file content");
        let full_path = root.join(relative_path);
        std::fs::create_dir_all(full_path.parent().expect("parent directory")).expect("create parent");
        std::fs::write(&full_path, &expected).expect("write file");

        let actual = storage.get(relative_path).await.expect("get should succeed");
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn get_returns_not_found_for_missing_file() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root);

        let result = storage.get(Path::new("segments/missing.bin")).await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn get_rejects_forbidden_relative_path() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root);

        let result = storage.get(Path::new("../escape")).await;
        assert!(matches!(result, Err(StorageError::Unauthorized(_))));
    }

    #[tokio::test]
    async fn get_slice_reads_requested_range() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root.clone());

        let relative_path = Path::new("segments/000001/slice.bin");
        let full_path = root.join(relative_path);
        std::fs::create_dir_all(full_path.parent().expect("parent directory")).expect("create parent");
        std::fs::write(&full_path, b"abcdefghij").expect("write file");

        let actual = storage
            .get_slice(relative_path, 2..7)
            .await
            .expect("get_slice should succeed");
        assert_eq!(actual, Bytes::from_static(b"cdefg"));
    }

    #[tokio::test]
    async fn get_slice_returns_error_for_out_of_bounds_range() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root.clone());

        let relative_path = Path::new("segments/000001/slice_oob.bin");
        let full_path = root.join(relative_path);
        std::fs::create_dir_all(full_path.parent().expect("parent directory")).expect("create parent");
        std::fs::write(&full_path, b"abc").expect("write file");

        let result = storage.get_slice(relative_path, 1..10).await;
        assert!(matches!(result, Err(StorageError::Io(_))));
    }

    #[tokio::test]
    async fn get_slice_returns_empty_for_empty_range() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root.clone());

        let relative_path = Path::new("segments/000001/empty_range.bin");
        let full_path = root.join(relative_path);
        std::fs::create_dir_all(full_path.parent().expect("parent directory")).expect("create parent");
        std::fs::write(&full_path, b"abcdef").expect("write file");

        let actual = storage
            .get_slice(relative_path, 3..3)
            .await
            .expect("empty range should succeed");
        assert!(actual.is_empty());
    }

    #[tokio::test]
    async fn get_slice_rejects_forbidden_relative_path() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root);

        let result = storage.get_slice(Path::new("../escape"), 0..1).await;
        assert!(matches!(result, Err(StorageError::Unauthorized(_))));
    }

    #[tokio::test]
    async fn get_slice_rejects_invalid_reversed_range() {
        let tempdir = tempfile::tempdir().expect("create temp dir");
        let root = tempdir.path().join("storage-root");
        let storage = make_storage(root);

        let result = storage.get_slice(Path::new("segments/file.bin"), 4..1).await;
        assert!(matches!(result, Err(StorageError::Io(_))));
    }
}
