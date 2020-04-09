use bson::{bson, doc, from_bson, Document};
use mongodb::options::FindOneOptions;
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::{Deserialize, Serialize};

use crate::database;

use database::channel::Channel;
use database::message::Message;

impl<'r> FromParam<'r> for Channel {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let col = database::get_db().collection("channels");
        let result = col
            .find_one(doc! { "_id": param.to_string() }, None)
            .unwrap();

        if let Some(channel) = result {
            Ok(from_bson(bson::Bson::Document(channel)).expect("Failed to unwrap channel."))
        } else {
            Err(param)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelRef {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: u8,

    // information required for permission calculations
    pub recipients: Option<Vec<String>>,
    pub guild: Option<String>,
}

impl ChannelRef {
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
        let id = param.to_string();
        let result = database::get_collection("channels")
            .find_one(
                doc! { "_id": id },
                FindOneOptions::builder()
                    .projection(doc! {
                        "_id": 1,
                        "type": 1,
                        "recipients": 1,
                        "guild": 1,
                    })
                    .build(),
            )
            .unwrap();

        if let Some(channel) = result {
            Ok(from_bson(bson::Bson::Document(channel)).expect("Failed to deserialize channel."))
        } else {
            Err(param)
        }
    }
}

impl<'r> FromParam<'r> for Message {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let col = database::get_db().collection("messages");
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
