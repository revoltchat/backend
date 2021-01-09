use crate::database::*;

use super::hive::get_hive;
use hive_pubsub::PubSub;

pub async fn generate_subscriptions(user: &User) -> Result<(), String> {
    let hive = get_hive();
    hive.subscribe(user.id.clone(), user.id.clone())?;

    if let Some(relations) = &user.relations {
        for relation in relations {
            hive.subscribe(user.id.clone(), relation.id.clone())?;
        }
    }

    Ok(())
}
