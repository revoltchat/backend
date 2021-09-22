use std::collections::HashSet;

use crate::{database::*, notifications::events::ClientboundNotification};
use crate::{
    database::{db_conn, entities::User},
    util::result::{Error, Result},
};
use futures::StreamExt;
use mongodb::bson::{doc, from_document};

pub async fn generate_ready(mut user: User) -> Result<ClientboundNotification> {
    let mut user_ids: HashSet<String> = HashSet::new();

    if let Some(relationships) = &user.relations {
        user_ids.extend(
            relationships
                .iter()
                .map(|relationship| relationship.id.clone()),
        );
    }

    let members = User::fetch_memberships(&user.id).await?;
    let server_ids: Vec<String> = members.iter().map(|x| x.id.server.clone()).collect();
    let servers = db_conn().get_servers(&server_ids).await?;
    let channel_ids = servers
        .iter()
        .map(|&e| e.channels)
        .flatten()
        .collect::<Vec<String>>();

    let channels = db_conn()
        .get_sms_dms_groups_where_user_is_recipient(&channel_ids, &user.id)
        .await?;

    for channel in channels {
        if let Channel::Group { recipients, .. } = &channel {
            user_ids.extend(recipients.iter().cloned());
        } else if let Channel::DirectMessage { recipients, .. } = &channel {
            user_ids.extend(recipients.iter().cloned());
        }
    }

    user_ids.remove(&user.id);
    let mut users = if user_ids.len() > 0 {
        user.fetch_multiple_users(user_ids.into_iter().collect::<Vec<String>>())
            .await?
    } else {
        vec![]
    };

    user.relationship = Some(RelationshipStatus::User);
    user.online = Some(true);
    users.push(user.apply_badges());

    Ok(ClientboundNotification::Ready {
        users,
        servers,
        channels,
        members,
    })
}
