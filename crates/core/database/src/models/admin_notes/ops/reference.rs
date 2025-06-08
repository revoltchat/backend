use revolt_result::Result;

use crate::AdminObjectNote;
use crate::ReferenceDb;

use super::AbstractAdminNotes;

#[async_trait]
impl AbstractAdminNotes for ReferenceDb {
    async fn admin_note_update(&self, note: AdminObjectNote) -> Result<()> {
        let mut admin_notes = self.admin_object_notes.lock().await;
        if let Some(existing_note) = admin_notes.get_mut(&note.id) {
            existing_note.apply_options(note.to_partial());
            Ok(())
        } else {
            admin_notes.insert(note.id.clone(), note);
            Ok(())
        }
    }

    async fn admin_note_fetch(&self, target_id: &str) -> Result<AdminObjectNote> {
        let admin_notes = self.admin_object_notes.lock().await;
        if let Some(note) = admin_notes.get(target_id) {
            Ok(note.clone())
        } else {
            Err(create_error!(NotFound))
        }
    }
}
