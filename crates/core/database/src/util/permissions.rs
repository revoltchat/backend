use std::borrow::Cow;

use revolt_permissions::{
    calculate_user_permissions, ChannelType, Override, PermissionQuery, RelationshipStatus,
};

use crate::{Channel, Database, Member, Server, User};

/// Permissions calculator
#[derive(Clone)]
pub struct DatabasePermissionQuery<'a> {
    #[allow(dead_code)]
    database: &'a Database,

    perspective: &'a User,
    user: Option<Cow<'a, User>>,
    channel: Option<Cow<'a, Channel>>,
    server: Option<Cow<'a, Server>>,
    member: Option<Cow<'a, Member>>,

    // flag_known_relationship: Option<&'a RelationshipStatus>,
    cached_user_permission: Option<u32>,
    cached_permission: Option<u64>,
}

#[async_trait]
impl PermissionQuery for DatabasePermissionQuery<'_> {
    // * For calculating user permission

    /// Is our perspective user privileged?
    async fn are_we_privileged(&mut self) -> bool {
        self.perspective.privileged
    }

    /// Is our perspective user a bot?
    async fn are_we_a_bot(&mut self) -> bool {
        self.perspective.bot.is_some()
    }

    /// Is our perspective user and the currently selected user the same?
    async fn are_the_users_same(&mut self) -> bool {
        if let Some(other_user) = &self.user {
            self.perspective.id == other_user.id
        } else {
            false
        }
    }

    /// Get the relationship with have with the currently selected user
    async fn user_relationship(&mut self) -> RelationshipStatus {
        if let Some(other_user) = &self.user {
            if let Some(relations) = &self.perspective.relations {
                for entry in relations {
                    if entry.id == other_user.id {
                        return match entry.status {
                            crate::RelationshipStatus::None => RelationshipStatus::None,
                            crate::RelationshipStatus::User => RelationshipStatus::User,
                            crate::RelationshipStatus::Friend => RelationshipStatus::Friend,
                            crate::RelationshipStatus::Outgoing => RelationshipStatus::Outgoing,
                            crate::RelationshipStatus::Incoming => RelationshipStatus::Incoming,
                            crate::RelationshipStatus::Blocked => RelationshipStatus::Blocked,
                            crate::RelationshipStatus::BlockedOther => {
                                RelationshipStatus::BlockedOther
                            }
                        };
                    }
                }
            }
        }

        RelationshipStatus::None
    }

    /// Whether the currently selected user is a bot
    async fn user_is_bot(&mut self) -> bool {
        if let Some(other_user) = &self.user {
            other_user.bot.is_some()
        } else {
            false
        }
    }

    /// Do we have a mutual connection with the currently selected user?
    async fn have_mutual_connection(&mut self) -> bool {
        // TODO: User::has_mutual_connection
        false
    }

    // * For calculating server permission

    /// Is our perspective user the server's owner?
    async fn are_we_server_owner(&mut self) -> bool {
        todo!()
    }

    /// Is our perspective user a member of the server?
    async fn are_we_a_member(&mut self) -> bool {
        todo!()
    }

    /// Get default server permission
    async fn get_default_server_permissions(&mut self) -> u64 {
        todo!()
    }

    /// Get the ordered role overrides (from lowest to highest) for this member in this server
    async fn get_our_server_role_overrides(&mut self) -> Vec<Override> {
        todo!()
    }

    /// Is our perspective user timed out on this server?
    async fn are_we_timed_out(&mut self) -> bool {
        todo!()
    }

    // * For calculating channel permission

    /// Get the type of the channel
    async fn get_channel_type(&mut self) -> ChannelType {
        todo!()
    }

    /// Get the default channel permissions
    /// Group channel defaults should be mapped to an allow-only override
    async fn get_default_channel_permissions(&mut self) -> Override {
        todo!()
    }

    /// Get the ordered role overrides (from lowest to highest) for this member in this channel
    async fn get_our_channel_role_overrides(&mut self) -> Vec<Override> {
        todo!()
    }

    /// Do we own this group or saved messages channel if it is one of those?
    async fn do_we_own_the_channel(&mut self) -> bool {
        todo!()
    }

    /// Are we a recipient of this channel?
    async fn are_we_part_of_the_channel(&mut self) -> bool {
        todo!()
    }

    /// Set the current user as the recipient of this channel
    /// (this will only ever be called for DirectMessage channels, use unimplemented!() for other code paths)
    async fn set_recipient_as_user(&mut self) {
        todo!()
    }

    /// Set the current server as the server owning this channel
    /// (this will only ever be called for server channels, use unimplemented!() for other code paths)
    async fn set_server_from_channel(&mut self) {
        todo!()
    }
}

impl<'a> DatabasePermissionQuery<'a> {
    /// Create a new permission calculator
    pub fn new(database: &'a Database, perspective: &'a User) -> DatabasePermissionQuery<'a> {
        DatabasePermissionQuery {
            database,
            perspective,
            user: None,
            channel: None,
            server: None,
            member: None,

            cached_user_permission: None,
            cached_permission: None,
        }
    }

    /// Calculate the user permission value
    pub async fn calc_user(mut self) -> DatabasePermissionQuery<'a> {
        if self.cached_user_permission.is_some() {
            return self;
        }

        if self.user.is_none() {
            panic!("Expected `PermissionCalculator.user to exist.");
        }

        DatabasePermissionQuery {
            cached_user_permission: Some(calculate_user_permissions(&mut self).await),
            ..self
        }
    }

    /// Calculate the permission value
    pub async fn calc(self) -> DatabasePermissionQuery<'a> {
        if self.cached_permission.is_some() {
            return self;
        }

        self
    }

    /// Use user
    pub fn user(self, user: &'a User) -> DatabasePermissionQuery {
        DatabasePermissionQuery {
            user: Some(Cow::Borrowed(user)),
            ..self
        }
    }

    /// Use channel
    pub fn channel(self, channel: &'a Channel) -> DatabasePermissionQuery {
        DatabasePermissionQuery {
            channel: Some(Cow::Borrowed(channel)),
            ..self
        }
    }

    /// Use server
    pub fn server(self, server: &'a Server) -> DatabasePermissionQuery {
        DatabasePermissionQuery {
            server: Some(Cow::Borrowed(server)),
            ..self
        }
    }

    /// Use member
    pub fn member(self, member: &'a Member) -> DatabasePermissionQuery {
        DatabasePermissionQuery {
            member: Some(Cow::Borrowed(member)),
            ..self
        }
    }
}

/// Short-hand for creating a permission calculator
pub fn perms<'a>(database: &'a Database, perspective: &'a User) -> DatabasePermissionQuery<'a> {
    DatabasePermissionQuery::new(database, perspective)
}
