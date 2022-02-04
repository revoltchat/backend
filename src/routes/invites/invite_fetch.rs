use revolt_quark::models::File;
use revolt_quark::Result;

use rocket::serde::json::Value;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum InviteResponse {
    Server {
        server_id: String,
        server_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_icon: Option<File>,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_banner: Option<File>,
        channel_id: String,
        channel_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_description: Option<String>,
        user_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_avatar: Option<File>,
        member_count: i64,
    },
}

#[get("/<target>")]
pub async fn req(/*target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
