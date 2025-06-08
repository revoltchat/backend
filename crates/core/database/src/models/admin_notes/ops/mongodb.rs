use revolt_result::Result;

use crate::AdminObjectNote;
use crate::MongoDb;

use super::AbstractAdminNotes;

static COL: &str = "admin_notes";

#[async_trait]
impl AbstractAdminNotes for MongoDb {
    async fn admin_note_update(&self, note: AdminObjectNote) -> Result<()> {
        let resp: Result<()> = query!(self, insert_one, COL, note.clone()).map(|_| ());
        if resp.is_err() {
            query!(
                self,
                update_one_by_id,
                COL,
                note.id.as_str(),
                note.to_partial(),
                vec![],
                None
            )
            .map(|_| ())
        } else {
            Ok(())
        }
    }

    async fn admin_note_fetch(&self, target_id: &str) -> Result<AdminObjectNote> {
        query!(self, find_one_by_id, COL, target_id)?
            .ok_or_else(|| create_database_error!("find_one", COL))
    }
}
