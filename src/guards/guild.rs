use bson::{doc, from_bson, Document};
use mongodb::options::FindOneOptions;
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::database::guild::Guild;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildRef {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: String,

    pub channels: Vec<String>,
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
                    "channels": 1,
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

impl<'r> FromParam<'r> for Guild {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let col = database::get_db().collection("guilds");
        let result = col
            .find_one(doc! { "_id": param.to_string() }, None)
            .unwrap();

        if let Some(guild) = result {
            Ok(from_bson(bson::Bson::Document(guild)).expect("Failed to unwrap guild."))
        } else {
            Err(param)
        }
    }
}
