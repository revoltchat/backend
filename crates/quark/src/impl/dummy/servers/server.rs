use crate::models::server::{FieldsRole, FieldsServer, PartialRole, PartialServer, Role, Server};
use crate::{AbstractServer, Result, DEFAULT_PERMISSION_SERVER};

use super::super::DummyDb;

#[async_trait]
impl AbstractServer for DummyDb {
    async fn fetch_server(&self, id: &str) -> Result<Server> {
        Ok(Server {
            id: id.into(),
            owner: "owner".into(),

            name: "server".into(),
            description: Some("server description".into()),

            channels: vec!["channel".into()],
            categories: None,
            system_messages: None,

            roles: std::collections::HashMap::new(),
            default_permissions: *DEFAULT_PERMISSION_SERVER as i64,

            icon: None,
            banner: None,

            flags: None,

            nsfw: false,
            analytics: true,
            discoverable: true,
        })
    }

    async fn fetch_servers<'a>(&self, _ids: &'a [String]) -> Result<Vec<Server>> {
        Ok(vec![self.fetch_server("sus").await.unwrap()])
    }

    async fn insert_server(&self, server: &Server) -> Result<()> {
        info!("Insert {server:?}");
        Ok(())
    }

    async fn update_server(
        &self,
        id: &str,
        server: &PartialServer,
        remove: Vec<FieldsServer>,
    ) -> Result<()> {
        info!("Update {id} with {server:?} and remove {remove:?}");
        Ok(())
    }

    async fn delete_server(&self, server: &Server) -> Result<()> {
        info!("Delete {server:?}");
        Ok(())
    }

    async fn insert_role(&self, server_id: &str, role_id: &str, role: &Role) -> Result<()> {
        info!("Create {role:?} on {server_id} as {role_id}");
        Ok(())
    }

    async fn update_role(
        &self,
        server_id: &str,
        role_id: &str,
        role: &PartialRole,
        remove: Vec<FieldsRole>,
    ) -> Result<()> {
        info!("Update {role_id} on {server_id} with {role:?} and remove {remove:?}");
        Ok(())
    }

    async fn delete_role(&self, server_id: &str, role_id: &str) -> Result<()> {
        info!("Delete {role_id} on {server_id}");
        Ok(())
    }
}
