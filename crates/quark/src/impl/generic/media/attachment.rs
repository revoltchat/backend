use crate::{models::File, Database, Result};

impl File {
    pub async fn use_attachment(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "attachments", "message", parent)
            .await
    }

    pub async fn use_background(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "backgrounds", "user", parent)
            .await
    }

    pub async fn use_avatar(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "avatars", "user", parent)
            .await
    }

    pub async fn use_icon(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "icons", "object", parent)
            .await
    }

    pub async fn use_server_icon(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "icons", "object", parent)
            .await
    }

    pub async fn use_banner(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "banners", "server", parent)
            .await
    }

    pub async fn use_emoji(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "emojis", "object", parent)
            .await
    }
}
