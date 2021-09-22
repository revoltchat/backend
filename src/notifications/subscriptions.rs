use crate::database::*;

use super::hive::get_hive;
use futures::StreamExt;
use hive_pubsub::PubSub;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::options::FindOptions;

pub async fn generate_subscriptions(user: &User) -> Result<(), String> {
    let hive = get_hive();
    hive.subscribe(user.id.clone(), user.id.clone())?;

    if let Some(relations) = &user.relations {
        for relation in relations {
            hive.subscribe(user.id.clone(), relation.id.clone())?;
        }
    }

    let server_ids = User::fetch_server_ids(&user.id)
        .await
        .map_err(|_| "Failed to fetch memberships.".to_string())?;

    let channel_ids = db_conn().get_channel_ids_from_servers(&server_ids).await?;

    for id in server_ids {
        hive.subscribe(user.id.clone(), id)?;
    }

    for id in channel_ids {
        hive.subscribe(user.id.clone(), id)?;
    }

    let channel_ids_to_subscribe = db_conn()
        .get_channel_ids_from_sms_dms_groups_where_user_is_recipient(&user.id).await?;

    for channel_id in channel_ids_to_subscribe {
        hive.subscribe(user.id.clone(), channel_id)?;
    }


    Ok(())
}
