use bson::{doc, from_bson, Bson, Document};
use mongodb::options::FindOneOptions;
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::database::guild::{Ban, Invite, Member};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildRef {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: String,

    pub bans: Vec<Ban>,

    pub default_permissions: i32,
}

impl GuildRef {
    pub fn from(id: String) -> Option<GuildRef> {
        match database::get_collection("guilds").find_one(
            doc! { "_id": id },
            FindOneOptions::builder()
                .projection(doc! {
                    "name": 1,
                    "description": 1,
                    "owner": 1,
                    "bans": 1,
                    "default_permissions": 1
                })
                .build(),
        ) {
            Ok(result) => match result {
                Some(doc) => {
                    Some(from_bson(bson::Bson::Document(doc)).expect("Failed to unwrap guild."))
                }
                None => None,
            },
            Err(_) => None,
        }
    }

    pub fn fetch_data(&self, projection: Document) -> Option<Document> {
        database::get_collection("guilds")
            .find_one(
                doc! { "_id": &self.id },
                FindOneOptions::builder().projection(projection).build(),
            )
            .expect("Failed to fetch guild from database.")
    }

    pub fn fetch_data_given(&self, mut filter: Document, projection: Document) -> Option<Document> {
        filter.insert("_id", self.id.clone());
        database::get_collection("guilds")
            .find_one(
                filter,
                FindOneOptions::builder().projection(projection).build(),
            )
            .expect("Failed to fetch guild from database.")
    }
}

impl<'r> FromParam<'r> for GuildRef {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        if let Some(guild) = GuildRef::from(param.to_string()) {
            Ok(guild)
        } else {
            Err(param)
        }
    }
}

pub fn get_member(guild_id: &String, member: &String) -> Option<Member> {
    if let Ok(result) = database::get_collection("members").find_one(
        doc! {
            "_id.guild": &guild_id,
            "_id.user": &member,
        },
        None,
    ) {
        if let Some(doc) = result {
            Some(from_bson(Bson::Document(doc)).expect("Failed to unwrap member."))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_invite(code: &String) -> Option<(String, String, Invite)> {
    if let Ok(result) = database::get_collection("guilds").find_one(
        doc! {
            "invites": {
                "$elemMatch": {
                    "code": &code
                }
            }
        },
        FindOneOptions::builder()
            .projection(doc! {
                "_id": 1,
                "name": 1,
                "invites.$": 1,
            })
            .build(),
    ) {
        if let Some(doc) = result {
            let invite = doc
                .get_array("invites")
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .as_document()
                .unwrap();

            Some((
                doc.get_str("_id").unwrap().to_string(),
                doc.get_str("name").unwrap().to_string(),
                from_bson(Bson::Document(invite.clone())).unwrap(),
            ))
        } else {
            None
        }
    } else {
        None
    }
}
