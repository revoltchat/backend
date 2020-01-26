use rocket::Outcome;
use rocket::http::{ Status, RawStr };
use rocket::request::{ self, Request, FromRequest, FromParam };

use bson::{ bson, doc, ordered::OrderedDocument };
use std::convert::TryFrom;
use ulid::Ulid;

use crate::database;
use crate::routes::channel::ChannelType;

pub struct Channel (
	pub Ulid,
	pub ChannelType,
	pub OrderedDocument,
);

pub struct Message (
	pub Ulid,
	pub OrderedDocument,
);

impl<'r> FromParam<'r> for Channel {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
		let col = database::get_db().collection("channels");
		let result = col.find_one(doc! { "_id": param.to_string() }, None).unwrap();

		if let Some(channel) = result {
			Ok(Channel (
				Ulid::from_string(channel.get_str("_id").unwrap()).unwrap(),
				ChannelType::try_from(channel.get_i32("type").unwrap() as usize).unwrap(),
				channel
			))
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
			Ok(Message (
				Ulid::from_string(message.get_str("_id").unwrap()).unwrap(),
				message
			))
		} else {
			Err(param)
		}
    }
}
