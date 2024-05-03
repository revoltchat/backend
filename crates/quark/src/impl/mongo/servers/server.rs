use bson::{to_document, Bson, Document};

use crate::models::server::{FieldsRole, FieldsServer, PartialRole, PartialServer, Role, Server};
use crate::r#impl::mongo::IntoDocumentPath;
use crate::{AbstractServer, Database, Error, Result};

use super::super::MongoDb;

static COL: &str = "servers";

impl MongoDb {
    pub async fn delete_associated_server_objects(&self, server: &Server) -> Result<()> {
        // Check if there are any attachments we need to delete.
        self.delete_bulk_messages(doc! {
            "channel": {
                "$in": &server.channels
            }
        })
        .await?;

        // Delete all emoji.
        self.col::<Document>("emojis")
            .delete_many(
                doc! {
                    "parent.id": &server.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "emojis",
            })?;

        // Delete all channels.
        self.col::<Document>("channels")
            .delete_many(
                doc! {
                    "server": &server.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "channels",
            })?;

        // Delete any associated objects, e.g. unreads and invites.
        self.delete_associated_channel_objects(Bson::Document(doc! { "$in": &server.channels }))
            .await?;

        // Delete members and bans.
        for with in &["server_members", "server_bans"] {
            self.col::<Document>(with)
                .delete_many(
                    doc! {
                        "_id.server": &server.id
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "delete_many",
                    with,
                })?;
        }

        // Update many attachments with parent id.
        self.delete_many_attachments(doc! {
            "object_id": &server.id
        })
        .await?;

        Ok(())
    }
}

#[async_trait]
impl AbstractServer for MongoDb {
    async fn fetch_server(&self, id: &str) -> Result<Server> {
        self.find_one_by_id(COL, id).await
    }

    async fn fetch_servers<'a>(&self, ids: &'a [String]) -> Result<Vec<Server>> {
        self.find(
            COL,
            doc! {
                "_id": {
                    "$in": ids
                }
            },
        )
        .await
    }

    async fn insert_server(&self, server: &Server) -> Result<()> {
        self.insert_one(COL, server).await.map(|_| ())
    }

    async fn update_server(
        &self,
        id: &str,
        server: &PartialServer,
        remove: Vec<FieldsServer>,
    ) -> Result<()> {
        self.update_one_by_id(
            COL,
            id,
            server,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None,
        )
        .await
        .map(|_| ())
    }

    async fn delete_server(&self, server: &Server) -> Result<()> {
        self.delete_associated_server_objects(server).await?;
        self.delete_one_by_id(COL, &server.id).await.map(|_| ())
    }

    async fn insert_role(&self, server_id: &str, role_id: &str, role: &Role) -> Result<()> {
        self.col::<Database>(COL)
            .update_one(
                doc! {
                    "_id": server_id
                },
                doc! {
                    "$set": {
                        "roles.".to_owned() + role_id: to_document(role)
                            .map_err(|_| Error::DatabaseError {
                                operation: "to_document",
                                with: "role"
                            })?
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "server",
            })
    }

    async fn update_role(
        &self,
        server_id: &str,
        role_id: &str,
        role: &PartialRole,
        remove: Vec<FieldsRole>,
    ) -> Result<()> {
        self.update_one_by_id(
            COL,
            server_id,
            role,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            "roles.".to_owned() + role_id + ".",
        )
        .await
        .map(|_| ())
    }

    async fn delete_role(&self, server_id: &str, role_id: &str) -> Result<()> {
        self.col::<Document>("server_members")
            .update_many(
                doc! {
                    "_id.server": server_id
                },
                doc! {
                    "$pull": {
                        "roles": &role_id
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_many",
                with: "server_members",
            })?;

        self.col::<Document>("channels")
            .update_one(
                doc! {
                    "server": server_id
                },
                doc! {
                    "$unset": {
                        "role_permissions.".to_owned() + role_id: 1_i32
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channels",
            })?;

        self.col::<Document>("servers")
            .update_one(
                doc! {
                    "_id": server_id
                },
                doc! {
                    "$unset": {
                        "roles.".to_owned() + role_id: 1_i32
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "servers",
            })
            .map(|_| ())
    }
}

impl IntoDocumentPath for FieldsServer {
    fn as_path(&self) -> Option<&'static str> {
        Some(match self {
            FieldsServer::Banner => "banner",
            FieldsServer::Categories => "categories",
            FieldsServer::Description => "description",
            FieldsServer::Icon => "icon",
            FieldsServer::SystemMessages => "system_messages",
        })
    }
}

impl IntoDocumentPath for FieldsRole {
    fn as_path(&self) -> Option<&'static str> {
        Some(match self {
            FieldsRole::Colour => "colour",
        })
    }
}
