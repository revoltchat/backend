use bson::{doc, from_bson, Document};
use mongodb::options::FindOneOptions;
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::{Deserialize, Serialize};

use crate::database;

use database::channel::LastMessage;
use database::message::Message;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelRef {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: u8,

    pub name: Option<String>,
    pub last_message: Option<LastMessage>,

    // information required for permission calculations
    pub recipients: Option<Vec<String>>,
    pub guild: Option<String>,
    pub owner: Option<String>,
}

impl ChannelRef {
    pub fn from(id: String) -> Option<ChannelRef> {
        let channel = database::channel::fetch_channel(&id);
        Some(ChannelRef {
            id: channel.id,
            channel_type: channel.channel_type,

            name: channel.name,
            last_message: channel.last_message,
            
            recipients: channel.recipients,
            guild: channel.guild,
            owner: channel.owner
        })

        /*match database::get_collection("channels").find_one(
            doc! { "_id": id },
            FindOneOptions::builder()
                .projection(doc! {
                    "_id": 1,
                    "type": 1,
                    "name": 1,
                    "last_message": 1,
                    "recipients": 1,
                    "guild": 1,
                    "owner": 1,
                })
                .build(),
        ) {
            Ok(result) => match result {
                Some(doc) => {
                    Some(from_bson(bson::Bson::Document(doc)).expect("Failed to unwrap channel."))
                }
                None => None,
            },
            Err(_) => None,
        }*/
    }

    pub fn fetch_data(&self, projection: Document) -> Option<Document> {
        database::get_collection("channels")
            .find_one(
                doc! { "_id": &self.id },
                FindOneOptions::builder().projection(projection).build(),
            )
            .expect("Failed to fetch channel from database.")
    }
}

impl<'r> FromParam<'r> for ChannelRef {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        if let Some(channel) = ChannelRef::from(param.to_string()) {
            Ok(channel)
        } else {
            Err(param)
        }
    }
}

impl<'r> FromParam<'r> for Message {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let col = database::get_collection("messages");
        let result = col
            .find_one(doc! { "_id": param.to_string() }, None)
            .unwrap();

        if let Some(message) = result {
            Ok(from_bson(bson::Bson::Document(message)).expect("Failed to unwrap message."))
        } else {
            Err(param)
        }
    }
}
