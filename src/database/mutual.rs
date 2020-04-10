use super::get_collection;

use bson::doc;
use mongodb::options::{FindOptions, FindOneOptions};

pub fn find_mutual_guilds(user_id: &str, target_id: &str) -> Vec<String> {
    let col = get_collection("guilds");
    if let Ok(result) = col.find(
        doc! {
            "$and": [
                { "members": { "$elemMatch": { "id": user_id   } } },
                { "members": { "$elemMatch": { "id": target_id } } },
            ]
        },
        FindOptions::builder()
            .projection(doc! { "_id": 1 })
            .build(),
    ) {
        let mut results = vec![];

        for doc in result {
            if let Ok(guild) = doc {
                results.push(guild.get_str("_id").unwrap().to_string());
            }
        }

        results
    } else {
        vec![]
    }
}

pub fn find_mutual_friends(user_id: &str, target_id: &str) -> Vec<String> {
    let col = get_collection("users");
    if let Ok(result) = col.find(
        doc! {
            "$and": [
                { "relations": { "$elemMatch": { "id": user_id,   "status": 0 } } },
                { "relations": { "$elemMatch": { "id": target_id, "status": 0 } } },
            ]
        },
        FindOptions::builder()
            .projection(doc! { "_id": 1 })
            .build(),
    ) {
        let mut results = vec![];

        for doc in result {
            if let Ok(user) = doc {
                results.push(user.get_str("_id").unwrap().to_string());
            }
        }

        results
    } else {
        vec![]
    }
}

pub fn find_mutual_groups(user_id: &str, target_id: &str) -> Vec<String> {
    let col = get_collection("channels");
    if let Ok(result) = col.find(
        doc! {
            "type": 1,
            "$and": [
                { "recipients": user_id },
                { "recipients": target_id },
            ]
        },
        FindOptions::builder()
            .projection(doc! { "_id": 1 })
            .build(),
    ) {
        let mut results = vec![];

        for doc in result {
            if let Ok(group) = doc {
                results.push(group.get_str("_id").unwrap().to_string());
            }
        }

        results
    } else {
        vec![]
    }
}

pub fn has_mutual_connection(user_id: &str, target_id: &str) -> bool {
    let col = get_collection("guilds");
    if let Ok(result) = col.find_one(
        doc! {
            "$and": [
                { "members": { "$elemMatch": { "id": user_id   } } },
                { "members": { "$elemMatch": { "id": target_id } } },
            ]
        },
        FindOneOptions::builder()
            .projection(doc! { "_id": 1 }) // ? TODO: fetch permissions
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
