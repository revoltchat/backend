use futures::StreamExt;
use revolt_result::Result;

use crate::{FieldsMember, Member, MemberCompositeKey, PartialMember};
use crate::{IntoDocumentPath, MongoDb};

use super::AbstractServerMembers;

static COL: &str = "server_members";

#[async_trait]
impl AbstractServerMembers for MongoDb {
    /// Insert a new server member into the database
    async fn insert_member(&self, member: &Member) -> Result<()> {
        query!(self, insert_one, COL, &member).map(|_| ())
    }

    /// Fetch a server member by their id
    async fn fetch_member(&self, server_id: &str, user_id: &str) -> Result<Member> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "_id.server": server_id,
                "_id.user": user_id
            }
        )?
        .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch all members in a server
    async fn fetch_all_members<'a>(&self, server_id: &str) -> Result<Vec<Member>> {
        Ok(self
            .col::<Member>(COL)
            .find(
                doc! {
                    "_id.server": server_id
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("find", COL))?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }

    /// Fetch all memberships for a user
    async fn fetch_all_memberships<'a>(&self, user_id: &str) -> Result<Vec<Member>> {
        Ok(self
            .col::<Member>(COL)
            .find(
                doc! {
                    "_id.user": user_id
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("find", COL))?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }

    /// Fetch multiple members by their ids
    async fn fetch_members<'a>(&self, server_id: &str, ids: &'a [String]) -> Result<Vec<Member>> {
        Ok(self
            .col::<Member>(COL)
            .find(
                doc! {
                    "_id.server": server_id,
                    "_id.user": {
                        "$in": ids
                    }
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("find", COL))?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }

    /// Fetch member count of a server
    async fn fetch_member_count(&self, server_id: &str) -> Result<usize> {
        self.col::<Member>(COL)
            .count_documents(
                doc! {
                    "_id.server": server_id
                },
                None,
            )
            .await
            .map(|c| c as usize)
            .map_err(|_| create_database_error!("count_documents", COL))
    }

    /// Fetch server count of a user
    async fn fetch_server_count(&self, user_id: &str) -> Result<usize> {
        self.col::<Member>(COL)
            .count_documents(
                doc! {
                    "_id.user": user_id
                },
                None,
            )
            .await
            .map(|c| c as usize)
            .map_err(|_| create_database_error!("count_documents", COL))
    }

    /// Update information for a server member
    async fn update_member(
        &self,
        id: &MemberCompositeKey,
        partial: &PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()> {
        query!(
            self,
            update_one,
            COL,
            doc! {
                "_id.server": &id.server,
                "_id.user": &id.user
            },
            partial,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None
        )
        .map(|_| ())
    }

    /// Delete a server member by their id
    async fn delete_member(&self, id: &MemberCompositeKey) -> Result<()> {
        query!(
            self,
            delete_one,
            COL,
            doc! {
                "_id.server": &id.server,
                "_id.user": &id.user
            }
        )
        .map(|_| ())
    }
}

impl IntoDocumentPath for FieldsMember {
    fn as_path(&self) -> Option<&'static str> {
        Some(match self {
            FieldsMember::Avatar => "avatar",
            FieldsMember::Nickname => "nickname",
            FieldsMember::Roles => "roles",
            FieldsMember::Timeout => "timeout",
        })
    }
}
