use revolt_result::Result;

use crate::FileHash;

mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractAttachmentHashes: Sync + Send {
    /// Fetch an attachment hash entry by sha256 hash.
    async fn fetch_attachment_hash(&self, hash: &str) -> Result<FileHash>;
}
