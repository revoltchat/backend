use bson::UtcDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserEmailVerification {
    pub verified: bool,
    pub target: Option<String>,
    pub expiry: Option<UtcDateTime>,
    pub rate_limit: Option<UtcDateTime>,
    pub code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRelationship {
    pub id: String,
    pub status: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub access_token: Option<String>,
    pub email_verification: UserEmailVerification,
    pub relations: Option<Vec<UserRelationship>>,
}
