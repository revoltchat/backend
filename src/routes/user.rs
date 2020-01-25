use crate::auth::User;

use rocket_contrib::json::{ JsonValue };
use bson::{ doc };

#[get("/@me")]
pub fn me(user: User) -> JsonValue {
	let User ( id, username, doc ) = user;

	json!({
		"id": id.to_string(),
		"username": username,
		"email": doc.get_str("email").expect("Missing email in user object!"),
		"verified": doc.get_document("email_verification").expect("DOC[email_verification]")
						.get_bool("verified").expect("DOC[verified]"),
		"created_timestamp": id.datetime().timestamp(),
	})
}
