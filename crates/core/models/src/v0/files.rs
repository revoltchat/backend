auto_derived!(
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
    }

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
        Image { width: usize, height: usize },
        /// File is a video with specific dimensions
        Video { width: usize, height: usize },
        /// File is audio
        Audio,
    }
);

#[cfg(feature = "from_database")]
impl From<revolt_database::File> for File {
    fn from(value: revolt_database::File) -> Self {
        File {
            id: value.id,
            tag: value.tag,
            filename: value.filename,
            metadata: value.metadata.into(),
            content_type: value.content_type,
            size: value.size,
            deleted: value.deleted,
            reported: value.reported,
            message_id: value.message_id,
            user_id: value.user_id,
            server_id: value.server_id,
            object_id: value.object_id,
        }
    }
}

#[cfg(feature = "from_database")]
impl From<revolt_database::Metadata> for Metadata {
    fn from(value: revolt_database::Metadata) -> Self {
        match value {
            revolt_database::Metadata::File => Metadata::File,
            revolt_database::Metadata::Text => Metadata::Text,
            revolt_database::Metadata::Image { width, height } => Metadata::Image {
                width: width as usize,
                height: height as usize,
            },
            revolt_database::Metadata::Video { width, height } => Metadata::Video {
                width: width as usize,
                height: height as usize,
            },
            revolt_database::Metadata::Audio => Metadata::Audio,
        }
    }
}
