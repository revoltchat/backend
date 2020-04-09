use super::get_collection;

use bson::doc;
use mongodb::options::FindOneOptions;

/*pub struct MutualGuild {

}

pub fn find_mutual_guilds(user_id: String, target_id: String) -> Vec<> {

}*/

pub fn has_mutual_connection(user_id: String, target_id: String) -> bool {
    let col = get_collection("guilds");
    if let Ok(result) = col.find_one(
        doc! {
            "$and": [
                { "members": { "$elemMatch": { "id": user_id   } } },
                { "members": { "$elemMatch": { "id": target_id } } },
            ]
        },
        FindOneOptions::builder()
            .projection(doc! { "_id": 1 })
            .build(),
    ) {
        if result.is_some() {
            true
        } else {
            false
        }
    } else {
        false
    }
}
