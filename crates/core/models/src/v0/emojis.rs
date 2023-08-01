auto_derived!(
    /// Emoji
    pub struct Emoji {
        /// Unique Id
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: String,
        /// What owns this emoji
        pub parent: EmojiParent,
        /// Uploader user id
        pub creator_id: String,
        /// Emoji name
        pub name: String,
        /// Whether the emoji is animated
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_false", default)
        )]
        pub animated: bool,
        /// Whether the emoji is marked as nsfw
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_false", default)
        )]
        pub nsfw: bool,
    }

    /// Parent Id of the emoji
    #[serde(tag = "type")]
    pub enum EmojiParent {
        Server { id: String },
        Detached,
    }
);
