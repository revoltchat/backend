use super::Response;
use crate::database::{
    self, get_relationship, get_relationship_internal, message::Message, Permission,
    PermissionCalculator, Relationship,
};
use crate::guards::auth::UserRef;
use crate::guards::channel::ChannelRef;
use crate::util::vec_to_set;

use bson::{doc, from_bson, Bson, Bson::UtcDatetime};
use chrono::prelude::*;
use mongodb::options::FindOptions;
use num_enum::TryFromPrimitive;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const MAXGROUPSIZE: usize = 50;

#[derive(Debug, TryFromPrimitive)]
#[repr(usize)]
pub enum ChannelType {
    DM = 0,
    GROUPDM = 1,
    GUILDCHANNEL = 2,
}

macro_rules! with_permissions {
    ($user: expr, $target: expr) => {{
        let permissions = PermissionCalculator::new($user.clone())
            .channel($target.clone())
            .fetch_data();

        let value = permissions.as_permission();
        if !value.get_access() {
            return None;
        }

        value
    }};
}

#[derive(Serialize, Deserialize)]
pub struct CreateGroup {
    name: String,
    nonce: String,
    users: Vec<String>,
}

/// create a new group
#[post("/create", data = "<info>")]
pub fn create_group(user: UserRef, info: Json<CreateGroup>) -> Response {
    let name: String = info.name.chars().take(32).collect();
    let nonce: String = info.nonce.chars().take(32).collect();

    let mut set = vec_to_set(&info.users);
    set.insert(user.id.clone());

    if set.len() > MAXGROUPSIZE {
        return Response::BadRequest(json!({ "error": "Maximum group size is 50." }));
    }

    let col = database::get_collection("channels");
    if let Some(_) = col.find_one(doc! { "nonce": nonce.clone() }, None).unwrap() {
        return Response::BadRequest(json!({ "error": "Group already created!" }));
    }

    let mut query = vec![];
    for item in &set {
        if item == &user.id {
            continue;
        }

        query.push(Bson::String(item.clone()));
    }

    if let Ok(result) = database::get_collection("users").find(
        doc! {
            "_id": {
                "$in": &query
            }
        },
        FindOptions::builder().limit(query.len() as i64).build(),
    ) {
        if result.count() != query.len() {
            return Response::BadRequest(json!({ "error": "Specified non-existant user(s)." }));
        }

        let relationships = user.fetch_relationships();
        for item in set {
            if item == user.id {
                continue;
            }

            if get_relationship_internal(&user.id, &item, &relationships) != Relationship::Friend {
                return Response::BadRequest(json!({ "error": "Not friends with user(s)." }));
            }
        }

        query.push(Bson::String(user.id.clone()));

        let id = Ulid::new().to_string();
        if col
            .insert_one(
                doc! {
                    "_id": id.clone(),
                    "nonce": nonce,
                    "type": ChannelType::GROUPDM as u32,
                    "recipients": &query,
                    "name": name,
                    "owner": &user.id,
                },
                None,
            )
            .is_ok()
        {
            Response::Success(json!({ "id": id }))
        } else {
            Response::InternalServerError(json!({ "error": "Failed to create guild channel." }))
        }
    } else {
        Response::InternalServerError(json!({ "error": "Failed to validate users." }))
    }
}

/// fetch channel information
#[get("/<target>")]
pub fn channel(user: UserRef, target: ChannelRef) -> Option<Response> {
    with_permissions!(user, target);

    match target.channel_type {
        0 => Some(Response::Success(json!({
            "id": target.id,
            "type": target.channel_type,
            "last_message": target.last_message,
            "recipients": target.recipients,
        }))),
        1 => {
            if let Some(info) = target.fetch_data(doc! {
                "name": 1,
                "description": 1,
                "owner": 1,
            }) {
                Some(Response::Success(json!({
                    "id": target.id,
                    "type": target.channel_type,
                    "last_message": target.last_message,
                    "recipients": target.recipients,
                    "name": info.get_str("name").unwrap(),
                    "owner": info.get_str("owner").unwrap(),
                    "description": info.get_str("description").unwrap_or(""),
                })))
            } else {
                None
            }
        }
        2 => {
            if let Some(info) = target.fetch_data(doc! {
                "name": 1,
                "description": 1,
            }) {
                Some(Response::Success(json!({
                    "id": target.id,
                    "type": target.channel_type,
                    "guild": target.guild,
                    "name": info.get_str("name").unwrap(),
                    "description": info.get_str("description").unwrap_or(""),
                })))
            } else {
                None
            }
        }
        _ => unreachable!(),
    }
}

/// [groups] add user to channel
#[put("/<target>/recipients/<member>")]
pub fn add_member(user: UserRef, target: ChannelRef, member: UserRef) -> Option<Response> {
    if target.channel_type != 1 {
        return Some(Response::BadRequest(json!({ "error": "Not a group DM." })));
    }

    with_permissions!(user, target);

    let recp = target.recipients.unwrap();
    if recp.len() == 50 {
        return Some(Response::BadRequest(
            json!({ "error": "Maximum group size is 50." }),
        ));
    }

    let set = vec_to_set(&recp);
    if set.get(&member.id).is_some() {
        return Some(Response::BadRequest(
            json!({ "error": "User already in group!" }),
        ));
    }

    match get_relationship(&user, &member) {
        Relationship::Friend => {
            if database::get_collection("channels")
                .update_one(
                    doc! { "_id": &target.id },
                    doc! {
                        "$push": {
                            "recipients": &member.id
                        }
                    },
                    None,
                )
                .is_ok()
            {
                Some(Response::Result(super::Status::Ok))
            } else {
                Some(Response::InternalServerError(
                    json!({ "error": "Failed to add user to group." }),
                ))
            }
        }
        _ => Some(Response::BadRequest(
            json!({ "error": "Not friends with user." }),
        )),
    }
}

/// [groups] remove user from channel
#[delete("/<target>/recipients/<member>")]
pub fn remove_member(user: UserRef, target: ChannelRef, member: UserRef) -> Option<Response> {
    if target.channel_type != 1 {
        return Some(Response::BadRequest(json!({ "error": "Not a group DM." })));
    }

    if &user.id == &member.id {
        return Some(Response::BadRequest(
            json!({ "error": "Cannot kick yourself, leave the channel instead." }),
        ));
    }

    let permissions = with_permissions!(user, target);

    if !permissions.get_kick_members() {
        return Some(Response::LackingPermission(Permission::KickMembers));
    }

    let set = vec_to_set(&target.recipients.unwrap());
    if set.get(&member.id).is_none() {
        return Some(Response::BadRequest(
            json!({ "error": "User not in group!" }),
        ));
    }

    if database::get_collection("channels")
        .update_one(
            doc! { "_id": &target.id },
            doc! {
                "$pull": {
                    "recipients": &member.id
                }
            },
            None,
        )
        .is_ok()
    {
        Some(Response::Result(super::Status::Ok))
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to add user to group." }),
        ))
    }
}

/// delete channel
/// or leave group DM
/// or close DM conversation
#[delete("/<target>")]
pub fn delete(user: UserRef, target: ChannelRef) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_manage_channels() {
        return Some(Response::LackingPermission(Permission::ManageChannels));
    }

    let col = database::get_collection("channels");
    let target_id = target.id.clone();

    let try_delete = || {
        let messages = database::get_collection("messages");

        if messages
            .delete_many(doc! { "channel": &target_id }, None)
            .is_ok()
        {
            if col.delete_one(doc! { "_id": &target_id }, None).is_ok() {
                Some(Response::Result(super::Status::Ok))
            } else {
                Some(Response::InternalServerError(
                    json!({ "error": "Failed to delete group." }),
                ))
            }
        } else {
            Some(Response::InternalServerError(
                json!({ "error": "Failed to delete messages." }),
            ))
        }
    };

    match target.channel_type {
        0 => {
            if col
                .update_one(
                    doc! { "_id": &target_id },
                    doc! { "$set": { "active": false } },
                    None,
                )
                .is_ok()
            {
                Some(Response::Result(super::Status::Ok))
            } else {
                Some(Response::InternalServerError(
                    json!({ "error": "Failed to close channel." }),
                ))
            }
        }
        1 => {
            let mut recipients =
                vec_to_set(&target.recipients.expect("Missing recipients on Group DM."));
            let owner = target.owner.expect("Missing owner on Group DM.");

            if recipients.len() == 1 {
                try_delete()
            } else {
                recipients.remove(&user.id);
                let new_owner = if owner == user.id {
                    recipients.iter().next().unwrap()
                } else {
                    &owner
                };

                if col
                    .update_one(
                        doc! { "_id": target_id },
                        doc! {
                            "$set": {
                                "owner": new_owner,
                            },
                            "$pull": {
                                "recipients": &user.id,
                            }
                        },
                        None,
                    )
                    .is_ok()
                {
                    Some(Response::Result(super::Status::Ok))
                } else {
                    Some(Response::InternalServerError(
                        json!({ "error": "Failed to remove you from the group." }),
                    ))
                }
            }
        }
        2 => {
            if database::get_collection("guilds")
                .update_one(
                    doc! { "_id": target.guild.unwrap() },
                    doc! {
                        "$pull": {
                            "invites": {
                                "channel": &target.id
                            }
                        }
                    },
                    None,
                )
                .is_ok()
            {
                try_delete()
            } else {
                Some(Response::InternalServerError(
                    json!({ "error": "Failed to remove invites." }),
                ))
            }
        }
        _ => Some(Response::InternalServerError(
            json!({ "error": "Unknown error has occurred." }),
        )),
    }
}

/// fetch channel messages
#[get("/<target>/messages")]
pub fn messages(user: UserRef, target: ChannelRef) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_read_messages() {
        return Some(Response::LackingPermission(Permission::ReadMessages));
    }

    let col = database::get_collection("messages");
    let result = col.find(doc! { "channel": target.id }, None).unwrap();

    let mut messages = Vec::new();
    for item in result {
        let message: Message =
            from_bson(bson::Bson::Document(item.unwrap())).expect("Failed to unwrap message.");
        messages.push(json!({
            "id": message.id,
            "author": message.author,
            "content": message.content,
            "edited": if let Some(t) = message.edited { Some(t.timestamp()) } else { None }
        }));
    }

    Some(Response::Success(json!(messages)))
}

#[derive(Serialize, Deserialize)]
pub struct SendMessage {
    content: String,
    nonce: String,
}

/// send a message to a channel
#[post("/<target>/messages", data = "<message>")]
pub fn send_message(
    user: UserRef,
    target: ChannelRef,
    message: Json<SendMessage>,
) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_send_messages() {
        if target.channel_type == 0 {
            return Some(Response::LackingPermission(Permission::SendDirectMessages));
        }

        return Some(Response::LackingPermission(Permission::SendMessages));
    }

    let content: String = message.content.chars().take(2000).collect();
    let nonce: String = message.nonce.chars().take(32).collect();

    let col = database::get_collection("messages");
    if col
        .find_one(doc! { "nonce": nonce.clone() }, None)
        .unwrap()
        .is_some()
    {
        return Some(Response::BadRequest(
            json!({ "error": "Message already sent!" }),
        ));
    }

    let id = Ulid::new().to_string();
    Some(
        if col
            .insert_one(
                doc! {
                    "_id": &id,
                    "nonce": nonce,
                    "channel": &target.id,
                    "author": &user.id,
                    "content": &content,
                },
                None,
            )
            .is_ok()
        {
            let short_content: String = content.chars().take(24).collect();
            let col = database::get_collection("channels");

            // !! this stuff can be async
            if target.channel_type == ChannelType::DM as u8
                || target.channel_type == ChannelType::GROUPDM as u8
            {
                let mut update = doc! {
                    "$set": {
                        "last_message": {
                            "id": &id,
                            "user_id": &user.id,
                            "short_content": short_content,
                        }
                    }
                };

                if target.channel_type == ChannelType::DM as u8 {
                    update
                        .get_document_mut("$set")
                        .unwrap()
                        .insert("active", true);
                }

                if col
                    .update_one(doc! { "_id": &target.id }, update, None)
                    .is_ok()
                {
                    Response::Success(json!({ "id": id }))
                } else {
                    Response::InternalServerError(json!({ "error": "Failed to update channel." }))
                }
            } else {
                Response::Success(json!({ "id": id }))
            }

        /*websocket::queue_message(
            get_recipients(&target),
            json!({
                "type": "message",
                "data": {
                    "id": id.clone(),
                    "nonce": nonce,
                    "channel": target.id,
                    "author": user.id,
                    "content": content,
                },
            })
            .to_string(),
        );*/
        } else {
            Response::InternalServerError(json!({
                "error": "Failed database query."
            }))
        },
    )
}

/// get a message
#[get("/<target>/messages/<message>")]
pub fn get_message(user: UserRef, target: ChannelRef, message: Message) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_read_messages() {
        return Some(Response::LackingPermission(Permission::ReadMessages));
    }

    let prev =
        // ! CHECK IF USER HAS PERMISSION TO VIEW EDITS OF MESSAGES
        if let Some(previous) = message.previous_content {
            let mut entries = vec![];
            for entry in previous {
                entries.push(json!({
                    "content": entry.content,
                    "time": entry.time.timestamp(),
                }));
            }

            Some(entries)
        } else {
            None
        };

    Some(Response::Success(json!({
        "id": message.id,
        "author": message.author,
        "content": message.content,
        "edited": if let Some(t) = message.edited { Some(t.timestamp()) } else { None },
        "previous_content": prev,
    })))
}

#[derive(Serialize, Deserialize)]
pub struct EditMessage {
    content: String,
}

/// edit a message
#[patch("/<target>/messages/<message>", data = "<edit>")]
pub fn edit_message(
    user: UserRef,
    target: ChannelRef,
    message: Message,
    edit: Json<EditMessage>,
) -> Option<Response> {
    with_permissions!(user, target);

    if message.author != user.id {
        return Some(Response::Unauthorized(
            json!({ "error": "You did not send this message." }),
        ));
    }

    let col = database::get_collection("messages");
    let time = if let Some(edited) = message.edited {
        edited.0
    } else {
        Ulid::from_string(&message.id).unwrap().datetime()
    };

    let edited = Utc::now();
    match col.update_one(
        doc! { "_id": message.id.clone() },
        doc! {
            "$set": {
                "content": edit.content.clone(),
                "edited": UtcDatetime(edited)
            },
            "$push": {
                "previous_content": {
                    "content": message.content,
                    "time": time,
                }
            },
        },
        None,
    ) {
        Ok(_) => {
            /*websocket::queue_message(
                get_recipients(&target),
                json!({
                    "type": "message_update",
                    "data": {
                        "id": message.id,
                        "channel": target.id,
                        "content": edit.content.clone(),
                        "edited": edited.timestamp()
                    },
                })
                .to_string(),
            );*/

            Some(Response::Result(super::Status::Ok))
        }
        Err(_) => Some(Response::InternalServerError(
            json!({ "error": "Failed to update message." }),
        )),
    }
}

/// delete a message
#[delete("/<target>/messages/<message>")]
pub fn delete_message(user: UserRef, target: ChannelRef, message: Message) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_manage_messages() {
        if message.author != user.id {
            return Some(Response::LackingPermission(Permission::ManageMessages));
        }
    }

    let col = database::get_collection("messages");

    match col.delete_one(doc! { "_id": &message.id }, None) {
        Ok(_) => {
            /*websocket::queue_message(
                get_recipients(&target),
                json!({
                    "type": "message_delete",
                    "data": {
                        "id": message.id,
                        "channel": target.id
                    },
                })
                .to_string(),
            );*/

            Some(Response::Result(super::Status::Ok))
        }
        Err(_) => Some(Response::InternalServerError(
            json!({ "error": "Failed to delete message." }),
        )),
    }
}
