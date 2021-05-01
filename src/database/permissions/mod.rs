pub use crate::database::*;

pub mod channel;
pub mod user;

pub use user::get_relationship;

pub struct PermissionCalculator<'a> {
    perspective: &'a User,

    user: Option<&'a User>,
    relationship: Option<&'a RelationshipStatus>,
    channel: Option<&'a Channel>,

    has_mutual_connection: bool,
}

impl<'a> PermissionCalculator<'a> {
    pub fn new(perspective: &'a User) -> PermissionCalculator {
        PermissionCalculator {
            perspective,

            user: None,
            relationship: None,
            channel: None,

            has_mutual_connection: false,
        }
    }

    pub fn with_user(self, user: &'a User) -> PermissionCalculator {
        PermissionCalculator {
            user: Some(&user),
            ..self
        }
    }

    pub fn with_relationship(self, relationship: &'a RelationshipStatus) -> PermissionCalculator {
        PermissionCalculator {
            relationship: Some(&relationship),
            ..self
        }
    }

    pub fn with_channel(self, channel: &'a Channel) -> PermissionCalculator {
        PermissionCalculator {
            channel: Some(&channel),
            ..self
        }
    }

    pub fn with_mutual_connection(self) -> PermissionCalculator<'a> {
        PermissionCalculator {
            has_mutual_connection: true,
            ..self
        }
    }
}
