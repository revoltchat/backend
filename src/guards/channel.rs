use rocket::http::{ RawStr };
use rocket::request::{ FromParam };
use bson::{ bson, doc, from_bson };

use crate::database::{ self, user::User };

use database::channel::Channel;
use database::message::Message;

impl<'r> FromParam<'r> for Channel {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
		let col = database::get_db().collection("channels");
		let result = col.find_one(doc! { "_id": param.to_string() }, None).unwrap();

		if let Some(channel) = result {
			Ok(from_bson(bson::Bson::Document(channel)).expect("Failed to unwrap channel."))
		} else {
			Err(param)
		}
    }
}

impl<'r> FromParam<'r> for Message {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
		let col = database::get_db().collection("messages");
		let result = col.find_one(doc! { "_id": param.to_string() }, None).unwrap();

		if let Some(message) = result {
			Ok(from_bson(bson::Bson::Document(message)).expect("Failed to unwrap message."))
		} else {
			Err(param)
		}
    }
}
