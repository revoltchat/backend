use super::channel::ChannelType;
use super::Response;
use crate::database::{self, channel::Channel, Permission, PermissionCalculator};
use crate::guards::auth::UserRef;
use crate::guards::channel::ChannelRef;
use crate::guards::guild::{get_invite, get_member, GuildRef};
use crate::notifications::{
    self,
    events::{guilds::*, Notification},
};
use crate::util::gen_token;

use bson::{doc, from_bson, Bson};
use mongodb::options::{FindOneOptions, FindOptions};
use rocket::request::Form;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

macro_rules! with_permissions {
    ($user: expr, $target: expr) => {{
        let permissions = PermissionCalculator::new($user.clone())
            .guild($target.clone())
            .fetch_data();

        let value = permissions.as_permission();
        if !value.get_access() {
            return None;
        }

        (value, permissions.member.unwrap())
    }};
}

/// fetch your guilds
#[get("/@me")]
pub fn my_guilds(user: UserRef) -> Response {
    if let Ok(result) = database::get_collection("members").find(
        doc! {
            "_id.user": &user.id
        },
        None,
    ) {
        let mut guilds = vec![];
        for item in result {
            if let Ok(entry) = item {
                guilds.push(Bson::String(
                    entry
                        .get_document("_id")
                        .unwrap()
                        .get_str("guild")
                        .unwrap()
                        .to_string(),
                ));
            }
        }

        if let Ok(result) = database::get_collection("guilds").find(
            doc! {
                "_id": {
                    "$in": guilds
                }
            },
            FindOptions::builder()
                .projection(doc! {
                    "_id": 1,
                    "name": 1,
                    "description": 1,
                    "owner": 1,
                })
                .build(),
        ) {
            let mut parsed = vec![];
            for item in result {
                let doc = item.unwrap();
                parsed.push(json!({
                    "id": doc.get_str("_id").unwrap(),
                    "name": doc.get_str("name").unwrap(),
                    "description": doc.get_str("description").unwrap(),
                    "owner": doc.get_str("owner").unwrap(),
                }));
            }

            Response::Success(json!(parsed))
        } else {
            Response::InternalServerError(json!({ "error": "Failed to fetch guilds." }))
        }
    } else {
        Response::InternalServerError(json!({ "error": "Failed to fetch memberships." }))
    }
}

/// fetch a guild
#[get("/<target>")]
pub fn guild(user: UserRef, target: GuildRef) -> Option<Response> {
    with_permissions!(user, target);

    let col = database::get_collection("channels");
    match col.find(
        doc! {
            "type": 2,
            "guild": &target.id,
        },
        None,
    ) {
        Ok(results) => {
            let mut channels = vec![];
            for item in results {
                if let Ok(entry) = item {
                    if let Ok(channel) =
                        from_bson(bson::Bson::Document(entry)) as Result<Channel, _>
                    {
                        channels.push(json!({
                            "id": channel.id,
                            "name": channel.name,
                            "description": channel.description,
                        }));
                    }
                }
            }

            Some(Response::Success(json!({
                "id": target.id,
                "name": target.name,
                "description": target.description,
                "owner": target.owner,
                "channels": channels,
            })))
        }
        Err(_) => Some(Response::InternalServerError(
            json!({ "error": "Failed to fetch channels." }),
        )),
    }
}

/// delete or leave a guild
#[delete("/<target>")]
pub fn remove_guild(user: UserRef, target: GuildRef) -> Option<Response> {
    with_permissions!(user, target);

    if user.id == target.owner {
        let channels = database::get_collection("channels");
        if let Ok(result) = channels.find(
            doc! {
                "type": 2,
                "guild": &target.id
            },
            FindOptions::builder().projection(doc! { "_id": 1 }).build(),
        ) {
            let mut values = vec![];
            for item in result {
                if let Ok(doc) = item {
                    values.push(Bson::String(doc.get_str("_id").unwrap().to_string()));
                }
            }

            if database::get_collection("messages")
                .delete_many(
                    doc! {
                        "channel": {
                            "$in": values
                        }
                    },
                    None,
                )
                .is_ok()
            {
                if channels
                    .delete_many(
                        doc! {
                            "type": 2,
                            "guild": &target.id,
                        },
                        None,
                    )
                    .is_ok()
                {
                    if database::get_collection("guilds")
                        .delete_one(
                            doc! {
                                "_id": &target.id
                            },
                            None,
                        )
                        .is_ok()
                    {
                        notifications::send_message_threaded(
                            None,
                            target.id.clone(),
                            Notification::guild_delete(Delete {
                                id: target.id.clone(),
                            }),
                        );

                        Some(Response::Result(super::Status::Ok))
                    } else {
                        Some(Response::InternalServerError(
                            json!({ "error": "Failed to delete guild." }),
                        ))
                    }
                } else {
                    Some(Response::InternalServerError(
                        json!({ "error": "Failed to delete guild channels." }),
                    ))
                }
            } else {
                Some(Response::InternalServerError(
                    json!({ "error": "Failed to delete guild messages." }),
                ))
            }
        } else {
            Some(Response::InternalServerError(
                json!({ "error": "Could not fetch channels." }),
            ))
        }
    } else {
        if database::get_collection("members")
            .delete_one(
                doc! {
                    "_id.guild": &target.id,
                    "_id.user": &user.id,
                },
                None,
            )
            .is_ok()
        {
            notifications::send_message_threaded(
                None,
                target.id.clone(),
                Notification::guild_user_leave(UserLeave {
                    id: target.id.clone(),
                    user: user.id.clone(),
                    banned: false,
                }),
            );

            Some(Response::Result(super::Status::Ok))
        } else {
            Some(Response::InternalServerError(
                json!({ "error": "Failed to remove you from the guild." }),
            ))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateChannel {
    nonce: String,
    name: String,
    description: Option<String>,
}

/// create a new channel
#[post("/<target>/channels", data = "<info>")]
pub fn create_channel(
    user: UserRef,
    target: GuildRef,
    info: Json<CreateChannel>,
) -> Option<Response> {
    let (permissions, _) = with_permissions!(user, target);

    if !permissions.get_manage_channels() {
        return Some(Response::LackingPermission(Permission::ManageChannels));
    }

    let nonce: String = info.nonce.chars().take(32).collect();
    let name: String = info.name.chars().take(32).collect();
    let description: String = info
        .description
        .clone()
        .unwrap_or(String::new())
        .chars()
        .take(255)
        .collect();

    if let Ok(result) =
        database::get_collection("channels").find_one(doc! { "nonce": &nonce }, None)
    {
        if result.is_some() {
            return Some(Response::BadRequest(
                json!({ "error": "Channel already created." }),
            ));
        }

        let id = Ulid::new().to_string();
        if database::get_collection("channels")
            .insert_one(
                doc! {
                    "_id": &id,
                    "nonce": &nonce,
                    "type": 2,
                    "guild": &target.id,
                    "name": &name,
                    "description": &description,
                },
                None,
            )
            .is_ok()
        {
            notifications::send_message_threaded(
                None,
                target.id.clone(),
                Notification::guild_channel_create(ChannelCreate {
                    id: target.id.clone(),
                    channel: id.clone(),
                    name: name.clone(),
                    description: description.clone(),
                }),
            );

            Some(Response::Success(json!({ "id": &id })))
        } else {
            Some(Response::BadRequest(
                json!({ "error": "Couldn't create channel." }),
            ))
        }
    } else {
        Some(Response::BadRequest(
            json!({ "error": "Failed to check if channel was made." }),
        ))
    }
}

#[derive(Serialize, Deserialize)]
pub struct InviteOptions {
    // ? TODO: add options
}

/// create a new invite
#[post("/<target>/channels/<channel>/invite", data = "<_options>")]
pub fn create_invite(
    user: UserRef,
    target: GuildRef,
    channel: ChannelRef,
    _options: Json<InviteOptions>,
) -> Option<Response> {
    let (permissions, _) = with_permissions!(user, target);

    if !permissions.get_create_invite() {
        return Some(Response::LackingPermission(Permission::CreateInvite));
    }

    let code = gen_token(7);
    if database::get_collection("guilds")
        .update_one(
            doc! { "_id": target.id },
            doc! {
                "$push": {
                    "invites": {
                        "code": &code,
                        "creator": user.id,
                        "channel": channel.id,
                    }
                }
            },
            None,
        )
        .is_ok()
    {
        Some(Response::Success(json!({ "code": code })))
    } else {
        Some(Response::BadRequest(
            json!({ "error": "Failed to create invite." }),
        ))
    }
}

/// remove an invite
#[delete("/<target>/invites/<code>")]
pub fn remove_invite(user: UserRef, target: GuildRef, code: String) -> Option<Response> {
    let (permissions, _) = with_permissions!(user, target);

    if let Some((guild_id, _, invite)) = get_invite(&code, None) {
        if invite.creator != user.id {
            if !permissions.get_manage_server() {
                return Some(Response::LackingPermission(Permission::ManageServer));
            }
        }

        if database::get_collection("guilds")
            .update_one(
                doc! {
                    "_id": &guild_id,
                },
                doc! {
                    "$pull": {
                        "invites": {
                            "code": &code
                        }
                    }
                },
                None,
            )
            .is_ok()
        {
            Some(Response::Result(super::Status::Ok))
        } else {
            Some(Response::BadRequest(
                json!({ "error": "Failed to delete invite." }),
            ))
        }
    } else {
        Some(Response::NotFound(
            json!({ "error": "Failed to fetch invite or code is invalid." }),
        ))
    }
}

/// fetch all guild invites
#[get("/<target>/invites")]
pub fn fetch_invites(user: UserRef, target: GuildRef) -> Option<Response> {
    let (permissions, _) = with_permissions!(user, target);

    if !permissions.get_manage_server() {
        return Some(Response::LackingPermission(Permission::ManageServer));
    }

    if let Some(doc) = target.fetch_data(doc! {
        "invites": 1,
    }) {
        Some(Response::Success(json!(doc.get_array("invites").unwrap())))
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to fetch invites." }),
        ))
    }
}

/// view an invite before joining
#[get("/join/<code>", rank = 1)]
pub fn fetch_invite(user: UserRef, code: String) -> Response {
    if let Some((guild_id, name, invite)) = get_invite(&code, user.id) {
        if let Some(channel) = ChannelRef::from(invite.channel) {
            Response::Success(json!({
                "guild": {
                    "id": guild_id,
                    "name": name,
                },
                "channel": {
                    "id": channel.id,
                    "name": channel.name,
                }
            }))
        } else {
            Response::BadRequest(json!({ "error": "Failed to fetch channel." }))
        }
    } else {
        Response::NotFound(json!({ "error": "Failed to fetch invite or code is invalid." }))
    }
}

/// join a guild using an invite
#[post("/join/<code>", rank = 1)]
pub fn use_invite(user: UserRef, code: String) -> Response {
    if let Some((guild_id, _, invite)) = get_invite(&code, Some(user.id.clone())) {
        if let Ok(result) = database::get_collection("members").find_one(
            doc! {
                "_id.guild": &guild_id,
                "_id.user": &user.id
            },
            FindOneOptions::builder()
                .projection(doc! { "_id": 1 })
                .build(),
        ) {
            if result.is_none() {
                if database::get_collection("members")
                    .insert_one(
                        doc! {
                            "_id": {
                                "guild": &guild_id,
                                "user": &user.id
                            }
                        },
                        None,
                    )
                    .is_ok()
                {
                    notifications::send_message_threaded(
                        None,
                        guild_id.clone(),
                        Notification::guild_user_join(UserJoin {
                            id: guild_id.clone(),
                            user: user.id.clone(),
                        }),
                    );

                    Response::Success(json!({
                        "guild": &guild_id,
                        "channel": &invite.channel,
                    }))
                } else {
                    Response::InternalServerError(
                        json!({ "error": "Failed to add you to the guild." }),
                    )
                }
            } else {
                Response::BadRequest(json!({ "error": "Already in the guild." }))
            }
        } else {
            Response::InternalServerError(
                json!({ "error": "Failed to check if you're in the guild." }),
            )
        }
    } else {
        Response::NotFound(json!({ "error": "Failed to fetch invite or code is invalid." }))
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateGuild {
    name: String,
    description: Option<String>,
    nonce: String,
}

/// create a new guild
#[post("/create", data = "<info>")]
pub fn create_guild(user: UserRef, info: Json<CreateGuild>) -> Response {
    if !user.email_verified {
        return Response::Unauthorized(json!({ "error": "Email not verified!" }));
    }

    let name: String = info.name.chars().take(32).collect();
    let description: String = info
        .description
        .clone()
        .unwrap_or("No description.".to_string())
        .chars()
        .take(255)
        .collect();
    let nonce: String = info.nonce.chars().take(32).collect();

    let channels = database::get_collection("channels");
    let col = database::get_collection("guilds");
    if col
        .find_one(doc! { "nonce": nonce.clone() }, None)
        .unwrap()
        .is_some()
    {
        return Response::BadRequest(json!({ "error": "Guild already created!" }));
    }

    let id = Ulid::new().to_string();
    let channel_id = Ulid::new().to_string();
    if channels
        .insert_one(
            doc! {
                "_id": channel_id.clone(),
                "type": ChannelType::GUILDCHANNEL as u32,
                "name": "general",
                "description": "",
                "guild": id.clone(),
            },
            None,
        )
        .is_err()
    {
        return Response::InternalServerError(
            json!({ "error": "Failed to create guild channel." }),
        );
    }

    if database::get_collection("members")
        .insert_one(
            doc! {
                "_id": {
                    "guild": &id,
                    "user": &user.id
                }
            },
            None,
        )
        .is_err()
    {
        return Response::InternalServerError(
            json!({ "error": "Failed to add you to members list." }),
        );
    }

    if col
        .insert_one(
            doc! {
                "_id": &id,
                "nonce": nonce,
                "name": name,
                "description": description,
                "owner": &user.id,
                "invites": [],
                "bans": [],
                "default_permissions": 51,
            },
            None,
        )
        .is_ok()
    {
        Response::Success(json!({ "id": id }))
    } else {
        channels
            .delete_one(doc! { "_id": channel_id }, None)
            .expect("Failed to delete the channel we just made.");

        Response::InternalServerError(json!({ "error": "Failed to create guild." }))
    }
}

/// fetch a guild's member
#[get("/<target>/members")]
pub fn fetch_members(user: UserRef, target: GuildRef) -> Option<Response> {
    with_permissions!(user, target);

    if let Ok(result) =
        database::get_collection("members").find(doc! { "_id.guild": target.id }, None)
    {
        let mut users = vec![];

        for item in result {
            if let Ok(doc) = item {
                users.push(json!({
                    "id": doc.get_document("_id").unwrap().get_str("user").unwrap(),
                    "nickname": doc.get_str("nickname").ok(),
                }));
            }
        }

        Some(Response::Success(json!(users)))
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to fetch members." }),
        ))
    }
}

/// fetch a guild member
#[get("/<target>/members/<other>")]
pub fn fetch_member(user: UserRef, target: GuildRef, other: String) -> Option<Response> {
    with_permissions!(user, target);

    if let Some(member) = get_member(&target.id, &other) {
        Some(Response::Success(json!({
            "id": member.id.user,
            "nickname": member.nickname,
        })))
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to fetch member or user does not exist." }),
        ))
    }
}

/// kick a guild member
#[delete("/<target>/members/<other>")]
pub fn kick_member(user: UserRef, target: GuildRef, other: String) -> Option<Response> {
    let (permissions, _) = with_permissions!(user, target);

    if user.id == other {
        return Some(Response::BadRequest(
            json!({ "error": "Cannot kick yourself." }),
        ));
    }

    if !permissions.get_kick_members() {
        return Some(Response::LackingPermission(Permission::KickMembers));
    }

    if get_member(&target.id, &other).is_none() {
        return Some(Response::BadRequest(
            json!({ "error": "User not part of guild." }),
        ));
    }

    if database::get_collection("members")
        .delete_one(
            doc! {
                "_id.guild": &target.id,
                "_id.user": &other,
            },
            None,
        )
        .is_ok()
    {
        notifications::send_message_threaded(
            None,
            target.id.clone(),
            Notification::guild_user_leave(UserLeave {
                id: target.id.clone(),
                user: other.clone(),
                banned: false,
            }),
        );

        Some(Response::Result(super::Status::Ok))
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to kick member." }),
        ))
    }
}

#[derive(Serialize, Deserialize, FromForm)]
pub struct BanOptions {
    reason: Option<String>,
}

/// ban a guild member
#[put("/<target>/members/<other>/ban?<options..>")]
pub fn ban_member(
    user: UserRef,
    target: GuildRef,
    other: String,
    options: Form<BanOptions>,
) -> Option<Response> {
    let (permissions, _) = with_permissions!(user, target);
    let reason: String = options
        .reason
        .clone()
        .unwrap_or("No reason specified.".to_string())
        .chars()
        .take(64)
        .collect();

    if user.id == other {
        return Some(Response::BadRequest(
            json!({ "error": "Cannot ban yourself." }),
        ));
    }

    if !permissions.get_ban_members() {
        return Some(Response::LackingPermission(Permission::BanMembers));
    }

    if get_member(&target.id, &other).is_none() {
        return Some(Response::BadRequest(
            json!({ "error": "User not part of guild." }),
        ));
    }

    if database::get_collection("guilds")
        .update_one(
            doc! { "_id": &target.id },
            doc! {
                "$push": {
                    "bans": {
                        "id": &other,
                        "reason": reason,
                    }
                }
            },
            None,
        )
        .is_err()
    {
        return Some(Response::BadRequest(
            json!({ "error": "Failed to add ban to guild." }),
        ));
    }

    if database::get_collection("members")
        .delete_one(
            doc! {
                "_id.guild": &target.id,
                "_id.user": &other,
            },
            None,
        )
        .is_ok()
    {
        notifications::send_message_threaded(
            None,
            target.id.clone(),
            Notification::guild_user_leave(UserLeave {
                id: target.id.clone(),
                user: other.clone(),
                banned: true,
            }),
        );

        Some(Response::Result(super::Status::Ok))
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to kick member after adding to ban list." }),
        ))
    }
}

/// unban a guild member
#[delete("/<target>/members/<other>/ban")]
pub fn unban_member(user: UserRef, target: GuildRef, other: String) -> Option<Response> {
    let (permissions, _) = with_permissions!(user, target);

    if user.id == other {
        return Some(Response::BadRequest(
            json!({ "error": "Cannot unban yourself (not checking if you're banned)." }),
        ));
    }

    if !permissions.get_ban_members() {
        return Some(Response::LackingPermission(Permission::BanMembers));
    }

    if target
        .fetch_data_given(
            doc! {
                "bans": {
                    "$elemMatch": {
                        "id": &other
                    }
                }
            },
            doc! {},
        )
        .is_none()
    {
        return Some(Response::BadRequest(json!({ "error": "User not banned." })));
    }

    if database::get_collection("guilds")
        .update_one(
            doc! {
                "_id": &target.id
            },
            doc! {
                "$pull": {
                    "bans": {
                        "$elemMatch": {
                            "id": &other
                        }
                    }
                }
            },
            None,
        )
        .is_ok()
    {
        Some(Response::Result(super::Status::Ok))
    } else {
        Some(Response::BadRequest(
            json!({ "error": "Failed to remove ban." }),
        ))
    }
}

#[options("/<_target>")]
pub fn guild_preflight(_target: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/channels")]
pub fn create_channel_preflight(_target: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/channels/<_channel>/invite")]
pub fn create_invite_preflight(_target: String, _channel: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/invites/<_code>")]
pub fn remove_invite_preflight(_target: String, _code: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/invites")]
pub fn fetch_invites_preflight(_target: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/join/<_code>", rank = 1)]
pub fn invite_preflight(_code: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/create")]
pub fn create_guild_preflight() -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/members")]
pub fn fetch_members_preflight(_target: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/members/<_other>")]
pub fn fetch_member_preflight(_target: String, _other: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/<_target>/members/<_other>/ban")]
pub fn ban_member_preflight(_target: String, _other: String) -> Response {
    Response::Result(super::Status::Ok)
}
