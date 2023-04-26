use crate::Database;

use revolt_result::Result;

auto_derived_partial!(
    /// File
    pub struct File {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Tag / bucket this file was uploaded to
        pub tag: String,
        /// Original filename
        pub filename: String,
        /// Parsed metadata of this file
        pub metadata: Metadata,
        /// Raw content type of this file
        pub content_type: String,
        /// Size of this file (in bytes)
        pub size: isize,

        /// Whether this file was deleted
        #[serde(skip_serializing_if = "Option::is_none")]
        pub deleted: Option<bool>,
        /// Whether this file was reported
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reported: Option<bool>,

        // TODO: migrate this mess to having:
        // - author_id
        // - parent: Parent { Message(id), User(id), etc }
        #[serde(skip_serializing_if = "Option::is_none")]
        pub message_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub server_id: Option<String>,

        /// Id of the object this file is associated with
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<String>,
    },
    "PartialFile"
);

auto_derived!(
    /// Metadata associated with a file
    #[serde(tag = "type")]
    #[derive(Default)]
    pub enum Metadata {
        /// File is just a generic uncategorised file
        #[default]
        File,
        /// File contains textual data and should be displayed as such
        Text,
        /// File is an image with specific dimensions
        Image { width: isize, height: isize },
        /// File is a video with specific dimensions
        Video { width: isize, height: isize },
        /// File is audio
        Audio,
    }
);

impl File {
    /// Use a file for a message attachment
    pub async fn use_attachment(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "attachments", "message", parent)
            .await
    }

    /// Use a file for a user profile background
    pub async fn use_background(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "backgrounds", "user", parent)
            .await
    }

    /// Use a file for a user avatar
    pub async fn use_avatar(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "avatars", "user", parent)
            .await
    }

    /// Use a file for an icon
    pub async fn use_icon(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "icons", "object", parent)
            .await
    }

    /// Use a file for a server icon
    pub async fn use_server_icon(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "icons", "object", parent)
            .await
    }

    /// Use a file for a server banner
    pub async fn use_banner(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "banners", "server", parent)
            .await
    }

    /// Use a file for an emoji
    pub async fn use_emoji(db: &Database, id: &str, parent: &str) -> Result<File> {
        db.find_and_use_attachment(id, "emojis", "object", parent)
            .await
    }
}
