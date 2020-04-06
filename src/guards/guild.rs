use bson::{bson, doc, from_bson};
use rocket::http::RawStr;
use rocket::request::FromParam;

use crate::database;

use database::guild::Guild;

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
