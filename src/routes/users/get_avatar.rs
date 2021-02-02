use mongodb::options::FindOneOptions;
use rocket::response::Redirect;
use mongodb::bson::doc;
use urlencoding;
use md5;

use crate::util::result::{Error, Result};
use crate::util::variables::PUBLIC_URL;
use crate::database::*;

#[get("/<target>/avatar")]
pub async fn req(target: Ref) -> Result<Redirect> {
    let doc = get_collection("accounts")
        .find_one(
            doc! {
                "_id": &target.id
            },
            FindOneOptions::builder()
                .projection(doc! { "email": 1 })
                .build()
        )
        .await
        .map_err(|_| Error::DatabaseError { operation: "find_one", with: "user" })?
        .ok_or_else(|| Error::UnknownUser)?;
    
    let email = doc
        .get_str("email")
        .map_err(|_| Error::DatabaseError { operation: "get_str(email)", with: "user" })?
        .to_lowercase();

        let url = format!(
            "https://www.gravatar.com/avatar/{:x}?s=128&d={}",
            md5::compute(email),
            urlencoding::encode(
                &format!(
                    "{}/users/{}/default_avatar",
                    *PUBLIC_URL,
                    &target.id
                )
            )
        );

        dbg!(&url);

    Ok(
        Redirect::to(url)
    )
}
