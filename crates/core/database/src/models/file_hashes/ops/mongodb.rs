use revolt_result::Result;

use crate::FileHash;
use crate::MongoDb;

use super::AbstractAttachmentHashes;

static COL: &str = "attachment_hashes";

#[async_trait]
impl AbstractAttachmentHashes for MongoDb {
    /// Fetch an attachment hash entry by sha256 hash.
    async fn fetch_attachment_hash(&self, hash: &str) -> Result<FileHash> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "_id": hash
            }
        )?
        .ok_or_else(|| create_error!(NotFound))
    }
}
