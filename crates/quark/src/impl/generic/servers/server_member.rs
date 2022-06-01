use crate::{
    events::client::EventV1,
    models::{
        server_member::{FieldsMember, PartialMember},
        Member, Server,
    },
    Database, Result,
};

impl Member {
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
        if let Some(roles) = &self.roles {
            let mut value = i64::MAX;
            for role in roles {
                if let Some(role) = server.roles.get(role) {
                    if role.rank < value {
                        value = role.rank;
                    }
                }
            }

            value
        } else {
            i64::MAX
        }
    }

    pub fn remove(&mut self, field: &FieldsMember) {
        match field {
            FieldsMember::Avatar => self.avatar = None,
            FieldsMember::Nickname => self.nickname = None,
            FieldsMember::Roles => self.roles = None,
        }
    }
}
