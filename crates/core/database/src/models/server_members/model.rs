use iso8601_timestamp::Timestamp;
use revolt_result::Result;

use crate::{Database, File, Server};

auto_derived_partial!(
    /// Server Member
    pub struct Member {
        /// Unique member id
        #[serde(rename = "_id")]
        pub id: MemberCompositeKey,

        /// Time at which this user joined the server
        #[serde(skip_serializing_if = "Option::is_none")]
        pub joined_at: Option<Timestamp>,

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
    },
    "PartialMember"
);

auto_derived!(
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
    }

    /// Member removal intention
    pub enum RemovalIntention {
        Leave,
        Kick,
        Ban,
    }
);

impl Member {
    /// Update member data
    pub async fn update<'a>(
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

        /* // TODO: EventV1::ServerMemberUpdate {
            id: self.id.clone(),
            data: partial,
            clear: remove,
        }
        .p(self.id.server.clone())
        .await; */

        Ok(())
    }

    pub fn remove_field(&mut self, field: &FieldsMember) {
        match field {
            FieldsMember::Avatar => self.avatar = None,
            FieldsMember::Nickname => self.nickname = None,
            FieldsMember::Roles => self.roles.clear(),
            FieldsMember::Timeout => self.timeout = None,
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
}
