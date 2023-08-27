use std::collections::{HashMap, HashSet};

use revolt_permissions::OverrideField;
use revolt_result::Result;
use ulid::Ulid;

use crate::{events::client::EventV1, Database, File};

auto_derived_partial!(
    /// Server
    pub struct Server {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// User id of the owner
        pub owner: String,

        /// Name of the server
        pub name: String,
        /// Description for the server
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Channels within this server
        // ! FIXME: this may be redundant
        pub channels: Vec<String>,
        /// Categories for this server
        #[serde(skip_serializing_if = "Option::is_none")]
        pub categories: Option<Vec<Category>>,
        /// Configuration for sending system event messages
        #[serde(skip_serializing_if = "Option::is_none")]
        pub system_messages: Option<SystemMessageChannels>,

        /// Roles for this server
        #[serde(
            default = "HashMap::<String, Role>::new",
            skip_serializing_if = "HashMap::<String, Role>::is_empty"
        )]
        pub roles: HashMap<String, Role>,
        /// Default set of server and channel permissions
        pub default_permissions: i64,

        /// Icon attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon: Option<File>,
        /// Banner attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        pub banner: Option<File>,

        /// Bitfield of server flags
        #[serde(skip_serializing_if = "Option::is_none")]
        pub flags: Option<i32>,

        /// Whether this server is flagged as not safe for work
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub nsfw: bool,
        /// Whether to enable analytics
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub analytics: bool,
        /// Whether this server should be publicly discoverable
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub discoverable: bool,
    },
    "PartialServer"
);

auto_derived_partial!(
    /// Role
    pub struct Role {
        /// Role name
        pub name: String,
        /// Permissions available to this role
        pub permissions: OverrideField,
        /// Colour used for this role
        ///
        /// This can be any valid CSS colour
        #[serde(skip_serializing_if = "Option::is_none")]
        pub colour: Option<String>,
        /// Whether this role should be shown separately on the member sidebar
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub hoist: bool,
        /// Ranking of this role
        #[serde(default)]
        pub rank: i64,
    },
    "PartialRole"
);

auto_derived!(
    /// Channel category
    pub struct Category {
        /// Unique ID for this category
        pub id: String,
        /// Title for this category
        pub title: String,
        /// Channels in this category
        pub channels: Vec<String>,
    }

    /// System message channel assignments
    pub struct SystemMessageChannels {
        /// ID of channel to send user join messages in
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_joined: Option<String>,
        /// ID of channel to send user left messages in
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_left: Option<String>,
        /// ID of channel to send user kicked messages in
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_kicked: Option<String>,
        /// ID of channel to send user banned messages in
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_banned: Option<String>,
    }

    /// Optional fields on server object
    pub enum FieldsServer {
        Description,
        Categories,
        SystemMessages,
        Icon,
        Banner,
    }

    /// Optional fields on server object
    pub enum FieldsRole {
        Colour,
    }
);

#[allow(clippy::disallowed_methods)]
impl Server {
    /// Create a server
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_server(self).await
    }

    /// Update server data
    pub async fn update<'a>(
        &mut self,
        db: &Database,
        partial: PartialServer,
        remove: Vec<FieldsServer>,
    ) -> Result<()> {
        for field in &remove {
            self.remove_field(field);
        }

        self.apply_options(partial.clone());

        db.update_server(&self.id, &partial, remove.clone()).await?;

        EventV1::ServerUpdate {
            id: self.id.clone(),
            data: partial.into(),
            clear: remove.into_iter().map(|v| v.into()).collect(),
        }
        .p(self.id.clone())
        .await;

        Ok(())
    }

    /// Delete a server
    pub async fn delete(self, db: &Database) -> Result<()> {
        EventV1::ServerDelete {
            id: self.id.clone(),
        }
        .p(self.id.clone())
        .await;

        db.delete_server(&self.id).await
    }

    /// Remove a field from Server
    pub fn remove_field(&mut self, field: &FieldsServer) {
        match field {
            FieldsServer::Description => self.description = None,
            FieldsServer::Categories => self.categories = None,
            FieldsServer::SystemMessages => self.system_messages = None,
            FieldsServer::Icon => self.icon = None,
            FieldsServer::Banner => self.banner = None,
        }
    }

    /// Set role permission on a server
    pub async fn set_role_permission(
        &mut self,
        db: &Database,
        role_id: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        if let Some(role) = self.roles.get_mut(role_id) {
            role.update(
                db,
                &self.id,
                role_id,
                PartialRole {
                    permissions: Some(permissions),
                    ..Default::default()
                },
                vec![],
            )
            .await?;

            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    /* /// Create a new member in a server
    pub async fn create_member(
        &self,
        db: &Database,
        user: User,
        channels: Option<Vec<Channel>>,
    ) -> Result<Vec<Channel>> {
        if db.fetch_ban(&self.id, &user.id).await.is_ok() {
            return Err(Error::Banned);
        }

        let member = Member {
            id: MemberCompositeKey {
                server: self.id.clone(),
                user: user.id.clone(),
            },
            joined_at: Timestamp::now_utc(),
            nickname: None,
            avatar: None,
            roles: vec![],
            timeout: None,
        };

        db.insert_member(&member).await?;

        let should_fetch = channels.is_none();
        let mut channels = channels.unwrap_or_default();

        if should_fetch {
            let perm = perms(&user).server(self).member(&member);
            let existing_channels = db.fetch_channels(&self.channels).await?;
            for channel in existing_channels {
                if perm
                    .clone()
                    .channel(&channel)
                    .has_permission(db, Permission::ViewChannel)
                    .await?
                {
                    channels.push(channel);
                }
            }
        }

        /* // TODO: EventV1::ServerMemberJoin {
            id: self.id.clone(),
            user: user.id.clone(),
        }
        .p(self.id.clone())
        .await;

        EventV1::ServerCreate {
            id: self.id.clone(),
            server: self.clone(),
            channels: channels.clone(),
        }
        .private(user.id.clone())
        .await; */

        if let Some(id) = self
            .system_messages
            .as_ref()
            .and_then(|x| x.user_joined.as_ref())
        {
            SystemMessage::UserJoined {
                id: user.id.clone(),
            }
            .into_message(id.to_string())
            .create_no_web_push(db, id, false)
            .await
            .ok();
        }

        Ok(channels)
    }

    /// Remove a member from a server
    pub async fn remove_member(
        &self,
        db: &Database,
        member: Member,
        intention: RemovalIntention,
        silent: bool,
    ) -> Result<()> {
        db.delete_member(&member.id).await?;

        /* // TODO: EventV1::ServerMemberLeave {
            id: self.id.to_string(),
            user: member.id.user.clone(),
        }
        .p(member.id.server)
        .await; */

        if !silent {
            if let Some(id) = self.system_messages.as_ref().and_then(|x| match intention {
                RemovalIntention::Leave => x.user_left.as_ref(),
                RemovalIntention::Kick => x.user_kicked.as_ref(),
                RemovalIntention::Ban => x.user_banned.as_ref(),
            }) {
                match intention {
                    RemovalIntention::Leave => SystemMessage::UserLeft { id: member.id.user },
                    RemovalIntention::Kick => SystemMessage::UserKicked { id: member.id.user },
                    RemovalIntention::Ban => SystemMessage::UserBanned { id: member.id.user },
                }
                .into_message(id.to_string())
                .create_no_web_push(db, id, false)
                .await
                .ok();
            }
        }

        Ok(())
    }

    /// Create ban
    pub async fn ban_user(
        self,
        db: &Database,
        id: MemberCompositeKey,
        reason: Option<String>,
    ) -> Result<ServerBan> {
        let ban = ServerBan { id, reason };
        db.insert_ban(&ban).await?;
        Ok(ban)
    }

    /// Ban a member from a server
    pub async fn ban_member(
        self,
        db: &Database,
        member: Member,
        reason: Option<String>,
    ) -> Result<ServerBan> {
        self.remove_member(db, member.clone(), RemovalIntention::Ban, false)
            .await?;

        self.ban_user(db, member.id, reason).await
    } */
}

impl Role {
    /// Into optional struct
    pub fn into_optional(self) -> PartialRole {
        PartialRole {
            name: Some(self.name),
            permissions: Some(self.permissions),
            colour: self.colour,
            hoist: Some(self.hoist),
            rank: Some(self.rank),
        }
    }

    /// Create a role
    pub async fn create(&self, db: &Database, server_id: &str) -> Result<String> {
        let role_id = Ulid::new().to_string();
        db.insert_role(server_id, &role_id, self).await?;

        EventV1::ServerRoleUpdate {
            id: server_id.to_string(),
            role_id: role_id.to_string(),
            data: self.clone().into_optional().into(),
            clear: vec![],
        }
        .p(server_id.to_string())
        .await;

        Ok(role_id)
    }

    /// Update server data
    pub async fn update<'a>(
        &mut self,
        db: &Database,
        server_id: &str,
        role_id: &str,
        partial: PartialRole,
        remove: Vec<FieldsRole>,
    ) -> Result<()> {
        for field in &remove {
            self.remove_field(field);
        }

        self.apply_options(partial.clone());

        db.update_role(server_id, role_id, &partial, remove.clone())
            .await?;

        EventV1::ServerRoleUpdate {
            id: server_id.to_string(),
            role_id: role_id.to_string(),
            data: partial.into(),
            clear: vec![],
        }
        .p(server_id.to_string())
        .await;

        Ok(())
    }

    /// Remove field from Role
    pub fn remove_field(&mut self, field: &FieldsRole) {
        match field {
            FieldsRole::Colour => self.colour = None,
        }
    }

    /// Delete a role
    pub async fn delete(self, db: &Database, server_id: &str, role_id: &str) -> Result<()> {
        EventV1::ServerRoleDelete {
            id: server_id.to_string(),
            role_id: role_id.to_string(),
        }
        .p(server_id.to_string())
        .await;

        db.delete_role(server_id, role_id).await
    }
}

impl SystemMessageChannels {
    pub fn into_channel_ids(self) -> HashSet<String> {
        let mut ids = HashSet::new();

        if let Some(id) = self.user_joined {
            ids.insert(id);
        }

        if let Some(id) = self.user_left {
            ids.insert(id);
        }

        if let Some(id) = self.user_kicked {
            ids.insert(id);
        }

        if let Some(id) = self.user_banned {
            ids.insert(id);
        }

        ids
    }
}
