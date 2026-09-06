use iso8601_timestamp::Timestamp;
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};

use crate::{
    events::client::EventV1, util::permissions::DatabasePermissionQuery, Channel, Database, File,
    Server, SystemMessage, User,
};

fn default_true() -> bool {
    true
}

fn is_true(x: &bool) -> bool {
    *x
}

auto_derived_partial!(
    /// Server Member
    pub struct Member {
        /// Unique member id
        #[serde(rename = "_id")]
        pub id: MemberCompositeKey,

        /// Time at which this user joined the server
        pub joined_at: Timestamp,

        /// Member's nickname
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nickname: Option<String>,
        /// Avatar attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar: Option<File>,

        /// Member's roles
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub roles: Vec<String>,
        /// Timestamp this member is timed out until
        #[serde(skip_serializing_if = "Option::is_none")]
        pub timeout: Option<Timestamp>,

        /// Whether the member is server-wide voice muted
        #[serde(skip_serializing_if = "is_true", default = "default_true")]
        pub can_publish: bool,
        /// Whether the member is server-wide voice deafened
        #[serde(skip_serializing_if = "is_true", default = "default_true")]
        pub can_receive: bool,

        /// Id of the user whose invite this member joined with, when known.
        ///
        /// This used to live ONLY on the "joined the server" system message, in
        /// a channel - so wiping channels erased every inviter the server had,
        /// and it was already empty. It belongs on the member, where a channel
        /// wipe cannot reach it.
        ///
        /// Deliberately NOT part of `v0::Member`: that model is broadcast to
        /// every member of the server, and who vouched for whom is not
        /// everyone's business. Admins read it through the attribution route.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub invited_by: Option<String>,

        /// The invite code this member joined with, when known.
        ///
        /// This is the attribution key: a labelled code is how we tell a
        /// member's personal invite apart from the website, a social post, or
        /// the one-off migration link.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub invite_code: Option<String>,
        // This value only exists in the database, not the models.
        // If it is not-None, the database layer should return None to member fetching queries.
        // pub pending_deletion_at: Option<Timestamp>
    },
    "PartialMember"
);

auto_derived!(
    /// How a member came to be in the server.
    ///
    /// Passed to `Member::create` so the attribution lands on the member record
    /// itself rather than only on a system message in a channel.
    pub struct InviteAttribution {
        /// Id of the user who created the invite that was used
        pub by: String,
        /// The invite code that was used
        pub code: String,
    }

    /// Composite primary key consisting of server and user id
    #[derive(Hash, Default)]
    pub struct MemberCompositeKey {
        /// Server Id
        pub server: String,
        /// User Id
        pub user: String,
    }

    /// Optional fields on server member object
    pub enum FieldsMember {
        Nickname,
        Avatar,
        Roles,
        Timeout,
        CanReceive,
        CanPublish,
        JoinedAt,
        VoiceChannel,
    }

    /// Member removal intention
    pub enum RemovalIntention {
        Leave,
        Kick,
        Ban,
    }
);

impl Default for Member {
    fn default() -> Self {
        Self {
            id: Default::default(),
            joined_at: Timestamp::now_utc(),
            nickname: None,
            avatar: None,
            roles: vec![],
            timeout: None,
            can_publish: true,
            can_receive: true,
            invited_by: None,
            invite_code: None,
        }
    }
}

#[allow(clippy::disallowed_methods)]
impl Member {
    /// Create a new member in a server
    ///
    /// `attribution` describes how the join happened, when it came from an
    /// invite: who created the code, and which code it was. Both are written
    /// onto the member record and the code is also carried onto the join
    /// system message, as before.
    pub async fn create(
        db: &Database,
        server: &Server,
        user: &User,
        channels: Option<Vec<Channel>>,
        attribution: Option<InviteAttribution>,
    ) -> Result<(Member, Vec<Channel>)> {
        let invited_by = attribution.as_ref().map(|a| a.by.clone());
        let invite_code = attribution.as_ref().map(|a| a.code.clone());
        if db.fetch_ban(&server.id, &user.id).await.is_ok() {
            return Err(create_error!(Banned));
        }

        if db.fetch_member(&server.id, &user.id).await.is_ok() {
            return Err(create_error!(AlreadyInServer));
        }

        // Land new members in the server's configured holding role, if it still
        // exists. Checking against `server.roles` keeps a stale id (role deleted
        // after being configured) from writing a dangling role onto the member.
        let default_roles = server
            .default_member_role
            .as_ref()
            .filter(|role_id| server.roles.contains_key(*role_id))
            .map(|role_id| vec![role_id.to_string()])
            .unwrap_or_default();

        let mut member = Member {
            id: MemberCompositeKey {
                server: server.id.to_string(),
                user: user.id.to_string(),
            },
            roles: default_roles,
            invited_by: invited_by.clone(),
            invite_code,
            ..Default::default()
        };

        if let Some(updated) = db.insert_or_merge_member(&member).await? {
            member = updated;
        }

        let should_fetch = channels.is_none();
        let mut channels = channels.unwrap_or_default();

        if should_fetch {
            let query = DatabasePermissionQuery::new(db, user).server(server);
            let existing_channels = db.fetch_channels(&server.channels).await?;

            for channel in existing_channels {
                let mut channel_query = query.clone().channel(&channel);

                if calculate_channel_permissions(&mut channel_query)
                    .await
                    .has_channel_permission(ChannelPermission::ViewChannel)
                {
                    channels.push(channel);
                }
            }
        }

        let emojis = db.fetch_emoji_by_parent_id(&server.id).await?;

        #[allow(unused_mut)]
        let mut voice_states = Vec::new();

        #[cfg(feature = "voice")]
        for channel in &channels {
            if let Ok(Some(voice_state)) = crate::voice::get_channel_voice_state(
                &crate::voice::UserVoiceChannel::from_channel(channel),
            )
            .await
            {
                voice_states.push(voice_state)
            }
        }

        EventV1::ServerMemberJoin {
            id: server.id.clone(),
            user: user.id.clone(),
            member: member.clone().into(),
        }
        .p(server.id.clone())
        .await;

        EventV1::ServerCreate {
            id: server.id.clone(),
            server: server.clone().into(),
            channels: channels
                .clone()
                .into_iter()
                .map(|channel| channel.into())
                .collect(),
            emojis: emojis.into_iter().map(|emoji| emoji.into()).collect(),
            voice_states,
        }
        .private(user.id.clone())
        .await;

        if let Some(id) = server
            .system_messages
            .as_ref()
            .and_then(|x| x.user_joined.as_ref())
        {
            SystemMessage::UserJoined {
                id: user.id.clone(),
                by: invited_by,
            }
            .into_message(id.to_string())
            .send_without_notifications(db, None, None, false, false, false)
            .await
            .ok();
        }

        Ok((member, channels))
    }

    /// Update member data
    pub async fn update(
        &mut self,
        db: &Database,
        partial: PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()> {
        for field in &remove {
            self.remove_field(field);
        }

        self.apply_options(partial.clone());

        db.update_member(&self.id, &partial, remove.clone()).await?;

        EventV1::ServerMemberUpdate {
            id: self.id.clone().into(),
            data: partial.into(),
            clear: remove.into_iter().map(|field| field.into()).collect(),
        }
        .p(self.id.server.clone())
        .await;

        Ok(())
    }

    pub fn remove_field(&mut self, field: &FieldsMember) {
        match field {
            FieldsMember::JoinedAt => {}
            FieldsMember::Avatar => self.avatar = None,
            FieldsMember::Nickname => self.nickname = None,
            FieldsMember::Roles => self.roles.clear(),
            FieldsMember::Timeout => self.timeout = None,
            FieldsMember::CanReceive => self.can_receive = true,
            FieldsMember::CanPublish => self.can_publish = true,
            FieldsMember::VoiceChannel => {}
        }
    }

    /// Get this user's current ranking
    pub fn get_ranking(&self, server: &Server) -> i64 {
        let mut value = i64::MAX;
        for role in &self.roles {
            if let Some(role) = server.roles.get(role) {
                if role.rank < value {
                    value = role.rank;
                }
            }
        }

        value
    }

    /// Check whether this member is in timeout
    pub fn in_timeout(&self) -> bool {
        if let Some(timeout) = self.timeout {
            *timeout > *Timestamp::now_utc()
        } else {
            false
        }
    }

    /// Remove member from server
    pub async fn remove(
        self,
        db: &Database,
        server: &Server,
        intention: RemovalIntention,
        silent: bool,
    ) -> Result<()> {
        db.soft_delete_member(&self.id).await?;

        EventV1::ServerMemberLeave {
            id: self.id.server.to_string(),
            user: self.id.user.to_string(),
            reason: intention.clone().into(),
        }
        .p(self.id.server.to_string())
        .await;

        if !silent {
            if let Some(id) = server
                .system_messages
                .as_ref()
                .and_then(|x| match intention {
                    RemovalIntention::Leave => x.user_left.as_ref(),
                    RemovalIntention::Kick => x.user_kicked.as_ref(),
                    RemovalIntention::Ban => x.user_banned.as_ref(),
                })
            {
                match intention {
                    RemovalIntention::Leave => SystemMessage::UserLeft { id: self.id.user },
                    RemovalIntention::Kick => SystemMessage::UserKicked { id: self.id.user },
                    RemovalIntention::Ban => SystemMessage::UserBanned { id: self.id.user },
                }
                .into_message(id.to_string())
                // TODO: support notifications here in the future?
                .send_without_notifications(db, None, None, false, false, false)
                .await
                .ok();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use iso8601_timestamp::{Duration, Timestamp};
    use revolt_models::v0::DataCreateServer;

    use crate::{Member, PartialMember, RemovalIntention, Server, User};

    #[async_std::test]
    async fn muted_member_rejoin() {
        database_test!(|db| async move {
            match db {
                crate::Database::Reference(_) => return,
                crate::Database::MongoDb(_) => (),
            }
            let owner = User::create(&db, "Server Owner".to_string(), None, None)
                .await
                .unwrap();

            let kickable_user = User::create(&db, "Member".to_string(), None, None)
                .await
                .unwrap();

            let server = Server::create(
                &db,
                DataCreateServer {
                    name: "Server".to_string(),
                    description: None,
                    nsfw: None,
                },
                &owner,
                false,
            )
            .await
            .unwrap()
            .0;

            Member::create(&db, &server, &owner, None, None)
                .await
                .unwrap();
            let mut kickable_member = Member::create(&db, &server, &kickable_user, None, None)
                .await
                .unwrap()
                .0;

            kickable_member
                .update(
                    &db,
                    PartialMember {
                        timeout: Some(Timestamp::now_utc() + Duration::minutes(5)),
                        ..Default::default()
                    },
                    vec![],
                )
                .await
                .unwrap();

            assert!(kickable_member.in_timeout());

            kickable_member
                .remove(&db, &server, RemovalIntention::Kick, false)
                .await
                .unwrap();

            let kickable_member = Member::create(&db, &server, &kickable_user, None, None)
                .await
                .unwrap()
                .0;

            assert!(kickable_member.in_timeout())
        });
    }
}
