use bytes::Bytes;
use futures::stream::BoxStream;
use futures::TryStreamExt;
use std::io;
use std::ops::Range;
use async_trait::async_trait;

pub type ByteStream = BoxStream<'static, io::Result<Bytes>>;

#[async_trait]
pub trait PutPayload: Send + Sync {
    fn len(&self) -> u64;

    async fn range_byte_stream(&self, range: Range<u64>) -> io::Result<ByteStream>;

    async fn byte_stream(&self) -> io::Result<ByteStream>;

    /// Load the entire file into the memory
    async fn read_all(&self) -> io::Result<Bytes> {
        let total_len = self.len();

        let mut stream = self.range_byte_stream(0..total_len).await?;
        let mut data = Vec::with_capacity(total_len as usize);

        while let Some(chunk) = stream.try_next().await? {
            data.extend_from_slice(&chunk);
        }

        Ok(Bytes::from(data))
    }
}
