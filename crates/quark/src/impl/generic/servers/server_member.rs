use iso8601_timestamp::Timestamp;

use crate::{
    events::client::EventV1,
    models::{
        server_member::{FieldsMember, MemberCompositeKey, PartialMember},
        Member, Server,
    },
    Database, Result,
};

impl Member {
    pub fn new(server_id: String, user_id: String) -> Self {
        Self {
            id: MemberCompositeKey {
                server: server_id,
                user: user_id,
            },
            joined_at: Timestamp::now_utc(),
            nickname: None,
            avatar: None,
            roles: vec![],
            timeout: None,
        }
    }

    /// Update member data
    pub async fn update<'a>(
        &mut self,
        db: &Database,
        partial: PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()> {
        for field in &remove {
            self.remove(field);
        }

        self.apply_options(partial.clone());

        db.update_member(&self.id, &partial, remove.clone()).await?;

        EventV1::ServerMemberUpdate {
            id: self.id.clone(),
            data: partial,
            clear: remove,
        }
        .p(self.id.server.clone())
        .await;

        Ok(())
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

    pub fn remove(&mut self, field: &FieldsMember) {
        match field {
            FieldsMember::Avatar => self.avatar = None,
            FieldsMember::Nickname => self.nickname = None,
            FieldsMember::Roles => self.roles.clear(),
            FieldsMember::Timeout => self.timeout = None,
        }
    }
}
