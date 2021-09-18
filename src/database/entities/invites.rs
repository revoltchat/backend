use serde::{Deserialize, Serialize};

use crate::database::{db_conn, Queries};
use crate::util::result::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Invite {
    Server {
        #[serde(rename = "_id")]
        code: String,
        server: String,
        creator: String,
        channel: String,
    },
    Group {
        #[serde(rename = "_id")]
        code: String,
        creator: String,
        channel: String,
    }, /* User {
           code: String,
           user: String
       } */
}

impl Invite {
    pub fn code(&self) -> &String {
        match &self {
            Invite::Server { code, .. } => code,
            Invite::Group { code, .. } => code,
        }
    }

    pub fn creator(&self) -> &String {
        match &self {
            Invite::Server { creator, .. } => creator,
            Invite::Group { creator, .. } => creator,
        }
    }

    pub async fn get(code: &str) -> Result<Invite> {
        db_conn().get_invite_by_id(&code).await
    }

    pub async fn save(self) -> Result<()> {
        db_conn().add_invite(&self).await
    }

    pub async fn delete(&self) -> Result<()> {
        db_conn().delete_invite(self.code()).await
    }
}
