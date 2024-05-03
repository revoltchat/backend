use bson::Document;

use crate::models::server_member::{FieldsMember, Member, MemberCompositeKey, PartialMember};
use crate::r#impl::mongo::IntoDocumentPath;
use crate::{AbstractServerMember, Error, Result};

use super::super::MongoDb;

static COL: &str = "server_members";

#[async_trait]
impl AbstractServerMember for MongoDb {
    async fn fetch_member(&self, server: &str, user: &str) -> Result<Member> {
        self.find_one(
            COL,
            doc! {
                "_id.server": server,
                "_id.user": user
            },
        )
        .await
    }

    async fn insert_member(&self, member: &Member) -> Result<()> {
        self.insert_one(COL, member).await.map(|_| ())
    }

    async fn update_member(
        &self,
        id: &MemberCompositeKey,
        member: &PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()> {
        self.update_one(
            COL,
            doc! {
                "_id.server": &id.server,
                "_id.user": &id.user
            },
            member,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None,
        )
        .await
        .map(|_| ())
    }

    async fn delete_member(&self, id: &MemberCompositeKey) -> Result<()> {
        self.delete_one(
            COL,
            doc! {
                "_id.server": &id.server,
                "_id.user": &id.user
            },
        )
        .await
        .map(|_| ())
    }

    async fn fetch_all_members<'a>(&self, server: &str) -> Result<Vec<Member>> {
        self.find(
            COL,
            doc! {
                "_id.server": server
            },
        )
        .await
    }

    async fn fetch_all_memberships<'a>(&self, user: &str) -> Result<Vec<Member>> {
        self.find(
            COL,
            doc! {
                "_id.user": user
            },
        )
        .await
    }

    async fn fetch_members<'a>(&self, server: &str, ids: &'a [String]) -> Result<Vec<Member>> {
        self.find(
            COL,
            doc! {
                "_id.server": server,
                "_id.user": {
                    "$in": ids
                }
            },
        )
        .await
    }

    async fn fetch_member_count(&self, server: &str) -> Result<usize> {
        self.col::<Document>(COL)
            .count_documents(
                doc! {
                    "_id.server": server
                },
                None,
            )
            .await
            .map(|c| c as usize)
            .map_err(|_| Error::DatabaseError {
                operation: "count_documents",
                with: "server_members",
            })
    }

    async fn fetch_server_count(&self, user: &str) -> Result<usize> {
        self.col::<Document>(COL)
            .count_documents(
                doc! {
                    "_id.user": user
                },
                None,
            )
            .await
            .map(|c| c as usize)
            .map_err(|_| Error::DatabaseError {
                operation: "count_documents",
                with: "server_members",
            })
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
