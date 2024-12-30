use revolt_database::mongodb::bson::doc;
use revolt_quark::models::user_white_list::UserWhiteList;
use revolt_quark::{Database, Result};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataCreateWhiteList {
    pub email: String,
    pub phone_number: Option<String>,
    pub name: Option<String>,
}
/// # Generate a new white list
///
/// Accept a json with emails to whitelist and delete previous table
#[openapi(tag = "Others")]
#[post("/whitelist", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    data: Json<Vec<DataCreateWhiteList>>,
) -> Result<Json<Vec<UserWhiteList>>> {
    db.empty_white_list().await?;

    for new_white_list_item in &data.0 {
        db.insert_white_list(&UserWhiteList {
            email: new_white_list_item.email.clone(),
            name: new_white_list_item.name.clone(),
            phone_number: new_white_list_item.phone_number.clone(),
        })
        .await?;
    }
    db.fetch_white_lists().await.map(Json)
}
