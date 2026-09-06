use authifier::{models::Invite as AuthInvite, Authifier};
use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, Invite, User,
};
use revolt_models::v0;
use revolt_permissions::{
    calculate_channel_permissions, calculate_server_permissions, ChannelPermission,
};

use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Create Invite
///
/// Creates an invite to this channel.
///
/// Channel must be a `TextChannel`.
///
/// An optional body may set a human-readable `label` and a `max_uses` limit.
///
/// The DEFAULT differs by who is asking, and deliberately so:
///
/// * a member with only `InviteOthers` gets a **single-use** code. Invite
///   capability belongs to vetted members, and a single-use code is what keeps
///   "invite a friend" from becoming "post the link publicly".
/// * someone with `ManageServer` may choose: single-use, a count, or unlimited.
///   Only they can lift the limit, and only they can label a code.
#[openapi(tag = "Channel Invites")]
#[post("/<target>/invites", data = "<data>")]
pub async fn create_invite(
    db: &State<Database>,
    authifier: &State<Authifier>,
    user: User,
    target: Reference<'_>,
    data: Option<Json<v0::DataCreateInvite>>,
) -> Result<Json<v0::Invite>> {
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    let channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::InviteOthers)?;

    let data = data.map(|d| d.into_inner()).unwrap_or_default();

    // Only ManageServer may lift the single-use default or label a code.
    // Checked against the SERVER, not the channel, because the label and the
    // use limit are server-administration concerns rather than per-channel
    // ones - a member holding InviteOthers on one channel should not be able
    // to mint an unlimited code from it.
    let is_admin = match channel.server() {
        Some(server_id) => {
            let server = db.fetch_server(server_id).await?;
            let mut server_query = DatabasePermissionQuery::new(db, &user).server(&server);
            calculate_server_permissions(&mut server_query)
                .await
                .has_channel_permission(ChannelPermission::ManageServer)
        }
        None => false,
    };

    if !is_admin && (data.label.is_some() || data.max_uses.is_some()) {
        return Err(create_error!(MissingPermission {
            permission: "ManageServer".to_string()
        }));
    }

    // A member's code is single-use. An admin gets what they asked for, and
    // `unlimited: true` is spelled out rather than inferred from a missing
    // field, so "I forgot to set it" and "I meant unlimited" cannot look alike.
    let max_uses = if is_admin {
        if data.unlimited {
            None
        } else {
            data.max_uses.or(Some(1))
        }
    } else {
        Some(1)
    };

    let invite =
        Invite::create_channel_invite(db, &user, &channel, data.label.clone(), max_uses).await?;

    // When registration is invite-gated, authifier validates the submitted code
    // against its OWN invite store, which is separate from channel invites. A
    // server invite that isn't mirrored there would let a brand-new user open
    // the link but then fail to create an account -- so register it too.
    //
    // Server invites only: a group invite points at a DM group, not the
    // community, and shouldn't be able to authorise a registration.
    //
    // The error is propagated rather than ignored on purpose. A half-working
    // invite that joins but can't register is exactly the silent failure this
    // is meant to prevent; failing loudly lets the creator simply retry.
    if let Invite::Server { code, .. } = &invite {
        authifier
            .database
            .save_invite(&AuthInvite {
                id: code.clone(),
                used: false,
                claimed_by: None,
            })
            .await
            .map_err(|_| create_error!(InternalError))?;
    }

    Ok(Json(invite.into()))
}
