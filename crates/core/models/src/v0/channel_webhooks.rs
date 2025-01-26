#[cfg(feature = "validator")]
use validator::Validate;

use super::File;

auto_derived_partial!(
    /// Webhook
    pub struct Webhook {
        /// Webhook Id
        pub id: String,

        /// The name of the webhook
        pub name: String,

        /// The avatar of the webhook
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub avatar: Option<File>,

        /// User that created this webhook
        pub creator_id: String,

        /// The channel this webhook belongs to
        pub channel_id: String,

        /// The permissions for the webhook
        pub permissions: u64,

        /// The private token for the webhook
        pub token: Option<String>,
    },
    "PartialWebhook"
);

auto_derived!(
    /// Information about the webhook bundled with Message
    pub struct MessageWebhook {
        // The name of the webhook - 1 to 32 chars
        pub name: String,

        // The id of the avatar of the webhook, if it has one
        pub avatar: Option<String>,
    }

    /// New webhook information
    #[cfg_attr(feature = "validator", derive(validator::Validate))]
    pub struct DataEditWebhook {
        /// Webhook name
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
        pub name: Option<String>,

        /// Avatar ID
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 128)))]
        pub avatar: Option<String>,

        /// Webhook permissions
        pub permissions: Option<u64>,

        /// Fields to remove from webhook
        #[cfg_attr(feature = "serde", serde(default))]
        pub remove: Vec<FieldsWebhook>,
    }

    /// Webhook information
    pub struct ResponseWebhook {
        /// Webhook Id
        pub id: String,

        /// Webhook name
        pub name: String,

        /// Avatar ID
        pub avatar: Option<String>,

        /// The channel this webhook belongs to
        pub channel_id: String,

        /// The permissions for the webhook
        pub permissions: u64,
    }

    /// Optional fields on webhook object
    pub enum FieldsWebhook {
        Avatar,
    }

    /// Information for the webhook
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct CreateWebhookBody {
        #[validate(length(min = 1, max = 32))]
        pub name: String,

        #[validate(length(min = 1, max = 128))]
        pub avatar: Option<String>,
    }
);

impl From<Webhook> for MessageWebhook {
    fn from(value: Webhook) -> Self {
        MessageWebhook {
            name: value.name,
            avatar: value.avatar.map(|file| file.id),
        }
    }
}

impl From<Webhook> for ResponseWebhook {
    fn from(value: Webhook) -> Self {
        ResponseWebhook {
            id: value.id,
            name: value.name,
            avatar: value.avatar.map(|file| file.id),
            channel_id: value.channel_id,
            permissions: value.permissions,
        }
    }
}
