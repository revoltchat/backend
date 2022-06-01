use nanoid::nanoid;

use crate::{
    models::{Channel, Invite, User},
    Database, Error, Result,
};

lazy_static! {
    static ref ALPHABET: [char; 54] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
        'J', 'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z'
    ];
}

impl Invite {
    /// Get the invite code for this invite
    pub fn code(&'_ self) -> &'_ str {
        match self {
            Invite::Server { code, .. } | Invite::Group { code, .. } => code,
        }
    }

    /// Get the ID of the user who created this invite
    pub fn creator(&'_ self) -> &'_ str {
        match self {
            Invite::Server { creator, .. } | Invite::Group { creator, .. } => creator,
        }
    }

    /// Create a new invite from given information
    pub async fn create(db: &Database, creator: &User, target: &Channel) -> Result<Invite> {
        let code = nanoid!(8, &*ALPHABET);
        let invite = match &target {
            Channel::Group { id, .. } => Ok(Invite::Group {
                code,
                creator: creator.id.clone(),
                channel: id.clone(),
            }),
            Channel::TextChannel { id, server, .. } | Channel::VoiceChannel { id, server, .. } => {
                Ok(Invite::Server {
                    code,
                    creator: creator.id.clone(),
                    server: server.clone(),
                    channel: id.clone(),
                })
            }
            _ => Err(Error::InvalidOperation),
        }?;

        db.insert_invite(&invite).await?;
        Ok(invite)
    }

    /// Resolve an invite by its ID or by a public server ID
    pub async fn find(db: &Database, code: &str) -> Result<Invite> {
        if let Ok(invite) = db.fetch_invite(code).await {
            return Ok(invite);
        } else if let Ok(server) = db.fetch_server(code).await {
            if server.discoverable {
                if let Some(channel) = server.channels.into_iter().next() {
                    return Ok(Invite::Server {
                        code: code.to_string(),
                        server: server.id,
                        creator: server.owner,
                        channel,
                    });
                }
            }
        }

        Err(Error::NotFound)
    }
}
