use super::{get_collection, MemberPermissions};

use bson::doc;
use mongodb::options::FindOptions;

pub fn find_mutual_guilds(user_id: &str, target_id: &str) -> Vec<String> {
    let col = get_collection("members");
    if let Ok(result) = col.find(
        doc! {
            "$and": [
                { "id": user_id   },
                { "id": target_id },
            ]
        },
        FindOptions::builder().projection(doc! { "_id": 1 }).build(),
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
        FindOptions::builder().projection(doc! { "_id": 1 }).build(),
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
        FindOptions::builder().projection(doc! { "_id": 1 }).build(),
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

pub fn has_mutual_connection(user_id: &str, target_id: &str, with_permission: bool) -> bool {
    let mut doc = doc! { "_id": 1 };

    if with_permission {
        doc.insert("default_permissions", 1);
    }

    let opt = FindOptions::builder().projection(doc);

    if let Ok(result) = get_collection("guilds").find(
        doc! {
            "$and": [
                { "members": { "$elemMatch": { "id": user_id   } } },
                { "members": { "$elemMatch": { "id": target_id } } },
            ]
        },
        if with_permission {
            opt.build()
        } else {
            opt.limit(1).build()
        },
    ) {
        if with_permission {
            for item in result {
                // ? logic should match permissions.rs#calculate
                if let Ok(guild) = item {
                    if guild.get_str("owner").unwrap() == user_id {
                        return true;
                    }

                    let permissions = guild.get_i32("default_permissions").unwrap() as u32;

                    if MemberPermissions([permissions]).get_send_direct_messages() {
                        return true;
                    }
                }
            }

            false
        } else {
            if result.count() > 0 {
                true
            } else {
                false
            }
        }
    } else {
        false
    }
}
