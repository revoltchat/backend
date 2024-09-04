use revolt_result::Result;

use crate::FileHash;
use crate::ReferenceDb;

use super::AbstractAttachmentHashes;

#[async_trait]
impl AbstractAttachmentHashes for ReferenceDb {
    /// Fetch an attachment hash entry by sha256 hash.
    async fn fetch_attachment_hash(&self, hash: &str) -> Result<FileHash> {
        let hashes = self.file_hashes.lock().await;
        if let Some(file) = hashes.get(hash) {
            if file.id == hash {
                Ok(file.clone())
            } else {
                Err(create_error!(NotFound))
            }
        } else {
            Err(create_error!(NotFound))
        }
    }
}
