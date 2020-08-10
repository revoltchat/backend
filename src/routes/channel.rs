use super::Response;
use crate::database::{
    self, channel::Channel, get_relationship, get_relationship_internal, message::Message,
    Permission, PermissionCalculator, Relationship, user::User
};
use crate::notifications::{
    self,
    events::{groups::*, guilds::ChannelDelete, message::*, Notification},
};
use crate::util::vec_to_set;

use chrono::prelude::*;
use mongodb::bson::{doc, from_bson, Bson};
use mongodb::options::FindOptions;
use num_enum::TryFromPrimitive;
use rocket::request::Form;
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
pub fn create_group(user: User, info: Json<CreateGroup>) -> Response {
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

        for item in set {
            if item == user.id {
                continue;
            }

            if get_relationship_internal(&user.id, &item, &user.relations) != Relationship::Friend {
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
pub fn channel(user: User, target: Channel) -> Option<Response> {
    with_permissions!(user, target);

    match target.channel_type {
        0 => Some(Response::Success(json!({
            "id": target.id,
            "type": target.channel_type,
            "last_message": target.last_message,
            "recipients": target.recipients,
        }))),
        1 => {
            /*if let Some(info) = target.fetch_data(doc! {
                "name": 1,
                "description": 1,
                "owner": 1,
            }) {*/
            Some(Response::Success(json!({
                "id": target.id,
                "type": target.channel_type,
                "last_message": target.last_message,
                "recipients": target.recipients,
                "name": target.name,
                "owner": target.owner,
                "description": target.description,
            })))
            /*} else {
                None
            }*/
        }
        2 => {
            /*if let Some(info) = target.fetch_data(doc! {
                "name": 1,
                "description": 1,
            }) {*/
            Some(Response::Success(json!({
                "id": target.id,
                "type": target.channel_type,
                "guild": target.guild,
                "name": target.name,
                "description": target.description,
            })))
            /*} else {
                None
            }*/
        }
        _ => unreachable!(),
    }
}

/// [groups] add user to channel
#[put("/<target>/recipients/<member>")]
pub fn add_member(user: User, target: Channel, member: User) -> Option<Response> {
    if target.channel_type != 1 {
        return Some(Response::BadRequest(json!({ "error": "Not a group DM." })));
    }

    with_permissions!(user, target);

    let recp = target.recipients.as_ref().unwrap();
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
                    doc! { "_id": target.id.clone() },
                    doc! {
                        "$push": {
                            "recipients": &member.id
                        }
                    },
                    None,
                )
                .is_ok()
            {
                if (Message {
                    id: Ulid::new().to_string(),
                    nonce: None,
                    channel: target.id.clone(),
                    author: "system".to_string(),
                    content: format!("<@{}> added <@{}> to the group.", &user.id, &member.id),
                    edited: None,
                    previous_content: vec![],
                })
                .send(&target)
                {
                    notifications::send_message_given_channel(
                        Notification::group_user_join(UserJoin {
                            id: target.id.clone(),
                            user: member.id.clone(),
                        }),
                        &target,
                    );

                    Some(Response::Result(super::Status::Ok))
                } else {
                    Some(Response::PartialStatus(
                        json!({ "error": "Failed to send join message, but user has been added." }),
                    ))
                }
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
pub fn remove_member(user: User, target: Channel, member: User) -> Option<Response> {
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

    let set = vec_to_set(target.recipients.as_ref().unwrap());
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
        if (Message {
            id: Ulid::new().to_string(),
            nonce: None,
            channel: target.id.clone(),
            author: "system".to_string(),
            content: format!("<@{}> removed <@{}> from the group.", &user.id, &member.id),
            edited: None,
            previous_content: vec![],
        })
        .send(&target)
        {
            notifications::send_message_given_channel(
                Notification::group_user_leave(UserLeave {
                    id: target.id.clone(),
                    user: member.id.clone(),
                }),
                &target,
            );

            Some(Response::Result(super::Status::Ok))
        } else {
            Some(Response::PartialStatus(
                json!({ "error": "Failed to send join message, but user has been removed." }),
            ))
        }
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
pub fn delete(user: User, target: Channel) -> Option<Response> {
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
                    json!({ "error": "Failed to delete channel." }),
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
            let mut recipients = vec_to_set(
                target
                    .recipients
                    .as_ref()
                    .expect("Missing recipients on Group DM."),
            );
            let owner = target.owner.as_ref().expect("Missing owner on Group DM.");

            if recipients.len() == 1 {
                try_delete()
            } else {
                recipients.remove(&user.id);
                let new_owner = if owner == &user.id {
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
                    if (Message {
                        id: Ulid::new().to_string(),
                        nonce: None,
                        channel: target.id.clone(),
                        author: "system".to_string(),
                        content: format!("<@{}> left the group.", &user.id),
                        edited: None,
                        previous_content: vec![],
                    })
                    .send(&target)
                    {
                        notifications::send_message_given_channel(
                            Notification::group_user_leave(UserLeave {
                                id: target.id.clone(),
                                user: user.id.clone(),
                            }),
                            &target,
                        );

                        Some(Response::Result(super::Status::Ok))
                    } else {
                        Some(Response::PartialStatus(
                            json!({ "error": "Failed to send leave message, but you have left the group." }),
                        ))
                    }
                } else {
                    Some(Response::InternalServerError(
                        json!({ "error": "Failed to remove you from the group." }),
                    ))
                }
            }
        }
        2 => {
            let guild_id = target.guild.unwrap();
            if database::get_collection("guilds")
                .update_one(
                    doc! { "_id": &guild_id },
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
                notifications::send_message_threaded(
                    None,
                    guild_id.clone(),
                    Notification::guild_channel_delete(ChannelDelete {
                        id: guild_id.clone(),
                        channel: target.id.clone(),
                    }),
                );

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

#[derive(Serialize, Deserialize, FromForm)]
pub struct MessageFetchOptions {
    limit: Option<i64>,
    before: Option<String>,
    after: Option<String>,
}

/// fetch channel messages
#[get("/<target>/messages?<options..>")]
pub fn messages(
    user: User,
    target: Channel,
    options: Form<MessageFetchOptions>,
) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_read_messages() {
        return Some(Response::LackingPermission(Permission::ReadMessages));
    }

    // ! FIXME: update wiki to reflect changes
    let mut query = doc! { "channel": target.id };

    if let Some(before) = &options.before {
        query.insert("_id", doc! { "$lt": before });
    }

    if let Some(after) = &options.after {
        query.insert("_id", doc! { "$gt": after });
    }

    let limit = if let Some(limit) = options.limit {
        limit.min(100).max(0)
    } else {
        50
    };

    let col = database::get_collection("messages");
    let result = col
        .find(
            query,
            FindOptions::builder()
                .limit(limit)
                .sort(doc! {
                    "_id": -1
                })
                .build(),
        )
        .unwrap();

    let mut messages = Vec::new();
    for item in result {
        let message: Message =
            from_bson(Bson::Document(item.unwrap())).expect("Failed to unwrap message.");
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
    user: User,
    target: Channel,
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

    if content.len() == 0 {
        return Some(Response::NotAcceptable(
            json!({ "error": "No message content!" }),
        ));
    }

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
    let message = Message {
        id: id.clone(),
        nonce: Some(nonce),
        channel: target.id.clone(),
        author: user.id,
        content,
        edited: None,
        previous_content: vec![],
    };

    if message.send(&target) {
        Some(Response::Success(json!({ "id": id })))
    } else {
        Some(Response::BadRequest(
            json!({ "error": "Failed to send message." }),
        ))
    }
}

/// get a message
#[get("/<target>/messages/<message>")]
pub fn get_message(user: User, target: Channel, message: Message) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_read_messages() {
        return Some(Response::LackingPermission(Permission::ReadMessages));
    }

    // ! CHECK IF USER HAS PERMISSION TO VIEW EDITS OF MESSAGES
    let mut entries = vec![];
    for entry in message.previous_content {
        entries.push(json!({
            "content": entry.content,
            "time": entry.time.timestamp(),
        }));
    }

    Some(Response::Success(json!({
        "id": message.id,
        "author": message.author,
        "content": message.content,
        "edited": if let Some(t) = message.edited { Some(t.timestamp()) } else { None },
        "previous_content": entries,
    })))
}

#[derive(Serialize, Deserialize)]
pub struct EditMessage {
    content: String,
}

/// edit a message
#[patch("/<target>/messages/<message>", data = "<edit>")]
pub fn edit_message(
    user: User,
    target: Channel,
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
                "content": &edit.content,
                "edited": Bson::DateTime(edited)
            },
            "$push": {
                "previous_content": {
                    "content": &message.content,
                    "time": time,
                }
            },
        },
        None,
    ) {
        Ok(_) => {
            notifications::send_message_given_channel(
                Notification::message_edit(Edit {
                    id: message.id.clone(),
                    channel: target.id.clone(),
                    author: message.author.clone(),
                    content: edit.content.clone(),
                }),
                &target,
            );

            Some(Response::Result(super::Status::Ok))
        }
        Err(_) => Some(Response::InternalServerError(
            json!({ "error": "Failed to update message." }),
        )),
    }
}

/// delete a message
#[delete("/<target>/messages/<message>")]
pub fn delete_message(user: User, target: Channel, message: Message) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_manage_messages() {
        if message.author != user.id {
            return Some(Response::LackingPermission(Permission::ManageMessages));
        }
    }

    let col = database::get_collection("messages");

    match col.delete_one(doc! { "_id": &message.id }, None) {
        Ok(_) => {
            notifications::send_message_given_channel(
                Notification::message_delete(Delete {
                    id: message.id.clone(),
                }),
                &target,
            );

            Some(Response::Result(super::Status::Ok))
        }
        Err(_) => Some(Response::InternalServerError(
            json!({ "error": "Failed to delete message." }),
        )),
    }
}

#[options("/create")]
pub fn create_group_preflight() -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>")]
pub fn channel_preflight(_target: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/recipients/<_member>")]
pub fn member_preflight(_target: String, _member: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/messages")]
pub fn messages_preflight(_target: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/messages/<_message>")]
pub fn message_preflight(_target: String, _message: String) -> Response {
    Response::Result(super::Status::Ok)
}
