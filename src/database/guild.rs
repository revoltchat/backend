use bson::{bson, doc};
use serde::{Deserialize, Serialize};

use super::get_collection;
use mongodb::options::FindOneOptions;

bitfield! {
    pub struct MemberPermissions(MSB0 [u8]);
    u8;
    pub get_access, set_access: 7;
    pub get_create_invite, set_create_invite: 6;
    pub get_kick_members, set_kick_members: 5;
    pub get_ban_members, set_ban_members: 4;
    pub get_read_messages, set_read_messages: 3;
    pub get_send_messages, set_send_messages: 2;
}

pub fn find_member_permissions<C: Into<Option<String>>>(
    id: String,
    guild: String,
    channel: C,
) -> u8 {
    let col = get_collection("guilds");

    match col.find_one(
        doc! {
            "_id": &guild,
            "members": {
                "$elemMatch": {
                    "id": &id,
                }
            }
        },
        FindOneOptions::builder()
            .projection(doc! {
                "members.$": 1,
                "owner": 1,
                "default_permissions": 1,
            })
            .build(),
    ) {
        Ok(result) => {
            if let Some(doc) = result {
                if doc.get_str("owner").unwrap() == id {
                    return u8::MAX;
                }

                doc.get_i32("default_permissions").unwrap() as u8
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Member {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Invite {
    pub id: String,
    pub custom: bool,
    pub channel: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Guild {
    #[serde(rename = "_id")]
    pub id: String,
    // pub nonce: String, used internally
    pub name: String,
    pub description: String,
    pub owner: String,

    pub channels: Vec<String>,
    pub members: Vec<Member>,
    pub invites: Vec<Invite>,

    pub default_permissions: u32,
}
