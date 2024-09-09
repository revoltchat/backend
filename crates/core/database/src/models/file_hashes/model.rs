use iso8601_timestamp::Timestamp;

auto_derived_partial!(
    /// File hash
    pub struct FileHash {
        /// Sha256 hash of the file
        #[serde(rename = "_id")]
        pub id: String,
        /// Sha256 hash of file after it has been processed
        pub processed_hash: String,

        /// When this file was created in system
        pub created_at: Timestamp,

        /// The bucket this file is stored in
        pub bucket_id: String,
        /// The path at which this file exists in
        pub path: String,
        /// Cryptographic nonce used to encrypt this file
        pub iv: String,

        /// Parsed metadata of this file
        pub metadata: Metadata,
        /// Raw content type of this file
        pub content_type: String,
        /// Size of this file (in bytes)
        pub size: isize,
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
        Image {
            width: isize,
            height: isize,
            // animated: bool // TODO: https://docs.rs/image/latest/image/trait.AnimationDecoder.html
        },
        /// File is a video with specific dimensions
        Video { width: isize, height: isize },
        /// File is audio
        Audio,
    }
);
