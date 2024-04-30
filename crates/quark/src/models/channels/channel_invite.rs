use serde::{Deserialize, Serialize};

/// Representation of an invite to a channel on Revolt
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum Invite {
    /// Invite to a specific server channel
    Server {
        /// Invite code
        #[serde(rename = "_id")]
        code: String,
        /// Id of the server this invite points to
        server: String,
        /// Id of user who created this invite
        creator: String,
        /// Id of the server channel this invite points to
        channel: String,
    },
    /// Invite to a group channel
    Group {
        /// Invite code
        #[serde(rename = "_id")]
        code: String,
        /// Id of user who created this invite
        creator: String,
        /// Id of the group channel this invite points to
        channel: String,
    }, /* User {
           code: String,
           user: String
       } */
}
