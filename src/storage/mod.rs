use std::io;
use async_trait::async_trait;
use bytes::Bytes;
use std::ops::Range;
use std::path::Path;
use crate::storage::payload::PutPayload;

mod local;
mod payload;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("storage notfound error {0}")]
    NotFound(String),
    #[error("storage unauthorized error {0}")]
    Unauthorized(String),
    #[error("storage internal error {0}")]
    Internal(String),
    #[error("storage IO error {0}")]
    Io(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

#[async_trait]
pub trait Storage {
    async fn check_connectivity(&self) -> anyhow::Result<()>;

    async fn put(&self, path: &Path, data: Box<dyn PutPayload>) -> StorageResult<()>;

    async fn get_slice(&self, path: &Path, range: Range<usize>) -> StorageResult<Bytes>;

    async fn get(&self, path: &Path) -> StorageResult<Bytes>;
}

impl From<io::Error> for StorageError {
    fn from(err: io::Error) -> StorageError {
        match err.kind() {
            io::ErrorKind::NotFound => StorageError::NotFound(err.to_string()),
            _ => StorageError::Io(err.to_string()),
        }
    }
}