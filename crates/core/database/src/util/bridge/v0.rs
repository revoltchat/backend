use revolt_models::v0::*;

impl crate::Bot {
    pub fn into_public_bot(self, user: crate::User) -> PublicBot {
        #[cfg(debug_assertions)]
        assert_eq!(self.id, user.id);

        PublicBot {
            id: self.id,
            username: user.username,
            avatar: user.avatar.map(|x| x.id).unwrap_or_default(),
            description: user
                .profile
                .map(|profile| profile.content)
                .unwrap_or_default(),
        }
    }
}

impl From<crate::Bot> for Bot {
    fn from(value: crate::Bot) -> Self {
        Bot {
            id: value.id,
            owner_id: value.owner,
            token: value.token,
            public: value.public,
            analytics: value.analytics,
            discoverable: value.discoverable,
            interactions_url: value.interactions_url,
            terms_of_service_url: value.terms_of_service_url,
            privacy_policy_url: value.privacy_policy_url,
            flags: value.flags.unwrap_or_default() as u32,
        }
    }
}

impl From<FieldsBot> for crate::FieldsBot {
    fn from(value: FieldsBot) -> Self {
        match value {
            FieldsBot::InteractionsURL => crate::FieldsBot::InteractionsURL,
            FieldsBot::Token => crate::FieldsBot::Token,
        }
    }
}

impl From<crate::FieldsBot> for FieldsBot {
    fn from(value: crate::FieldsBot) -> Self {
        match value {
            crate::FieldsBot::InteractionsURL => FieldsBot::InteractionsURL,
            crate::FieldsBot::Token => FieldsBot::Token,
        }
    }
}

impl From<crate::Invite> for Invite {
    fn from(value: crate::Invite) -> Self {
        match value {
            crate::Invite::Group {
                code,
                creator,
                channel,
            } => Invite::Group {
                code,
                creator,
                channel,
            },
            crate::Invite::Server {
                code,
                server,
                creator,
                channel,
            } => Invite::Server {
                code,
                server,
                creator,
                channel,
            },
        }
    }
}

impl From<crate::ChannelUnread> for ChannelUnread {
    fn from(value: crate::ChannelUnread) -> Self {
        ChannelUnread {
            id: value.id.into(),
            last_id: value.last_id,
            mentions: value.mentions.unwrap_or_default(),
        }
    }
}

impl From<crate::ChannelCompositeKey> for ChannelCompositeKey {
    fn from(value: crate::ChannelCompositeKey) -> Self {
        ChannelCompositeKey {
            channel: value.channel,
            user: value.user,
        }
    }
}

impl From<crate::Webhook> for Webhook {
    fn from(value: crate::Webhook) -> Self {
        Webhook {
            id: value.id,
            name: value.name,
            avatar: value.avatar.map(|file| file.into()),
            channel_id: value.channel_id,
            token: value.token,
            permissions: value.permissions,
        }
    }
}

impl From<crate::PartialWebhook> for PartialWebhook {
    fn from(value: crate::PartialWebhook) -> Self {
        PartialWebhook {
            id: value.id,
            name: value.name,
            avatar: value.avatar.map(|file| file.into()),
            channel_id: value.channel_id,
            token: value.token,
            permissions: value.permissions,
        }
    }
}

impl From<FieldsWebhook> for crate::FieldsWebhook {
    fn from(_value: FieldsWebhook) -> Self {
        Self::Avatar
    }
}

impl From<crate::FieldsWebhook> for FieldsWebhook {
    fn from(_value: crate::FieldsWebhook) -> Self {
        Self::Avatar
    }
}

impl From<crate::Channel> for Channel {
    fn from(value: crate::Channel) -> Self {
        match value {
            crate::Channel::SavedMessages { id, user } => Channel::SavedMessages { id, user },
            crate::Channel::DirectMessage {
                id,
                active,
                recipients,
                last_message_id,
            } => Channel::DirectMessage {
                id,
                active,
                recipients,
                last_message_id,
            },
            crate::Channel::Group {
                id,
                name,
                owner,
                description,
                recipients,
                icon,
                last_message_id,
                permissions,
                nsfw,
            } => Channel::Group {
                id,
                name,
                owner,
                description,
                recipients,
                icon: icon.map(|file| file.into()),
                last_message_id,
                permissions,
                nsfw,
            },
            crate::Channel::TextChannel {
                id,
                server,
                name,
                description,
                icon,
                last_message_id,
                default_permissions,
                role_permissions,
                nsfw,
            } => Channel::TextChannel {
                id,
                server,
                name,
                description,
                icon: icon.map(|file| file.into()),
                last_message_id,
                default_permissions,
                role_permissions,
                nsfw,
            },
            crate::Channel::VoiceChannel {
                id,
                server,
                name,
                description,
                icon,
                default_permissions,
                role_permissions,
                nsfw,
            } => Channel::VoiceChannel {
                id,
                server,
                name,
                description,
                icon: icon.map(|file| file.into()),
                default_permissions,
                role_permissions,
                nsfw,
            },
        }
    }
}

impl From<crate::PartialChannel> for PartialChannel {
    fn from(value: crate::PartialChannel) -> Self {
        PartialChannel {
            name: value.name,
            owner: value.owner,
            description: value.description,
            icon: value.icon.map(|file| file.into()),
            nsfw: value.nsfw,
            active: value.active,
            permissions: value.permissions,
            role_permissions: value.role_permissions,
            default_permissions: value.default_permissions,
            last_message_id: value.last_message_id,
        }
    }
}

impl From<FieldsChannel> for crate::FieldsChannel {
    fn from(value: FieldsChannel) -> Self {
        match value {
            FieldsChannel::Description => crate::FieldsChannel::Description,
            FieldsChannel::Icon => crate::FieldsChannel::Icon,
            FieldsChannel::DefaultPermissions => crate::FieldsChannel::DefaultPermissions,
        }
    }
}

impl From<crate::FieldsChannel> for FieldsChannel {
    fn from(value: crate::FieldsChannel) -> Self {
        match value {
            crate::FieldsChannel::Description => FieldsChannel::Description,
            crate::FieldsChannel::Icon => FieldsChannel::Icon,
            crate::FieldsChannel::DefaultPermissions => FieldsChannel::DefaultPermissions,
        }
    }
}

impl From<crate::Emoji> for Emoji {
    fn from(value: crate::Emoji) -> Self {
        Emoji {
            id: value.id,
            parent: value.parent.into(),
            creator_id: value.creator_id,
            name: value.name,
            animated: value.animated,
            nsfw: value.nsfw,
        }
    }
}

impl From<crate::EmojiParent> for EmojiParent {
    fn from(value: crate::EmojiParent) -> Self {
        match value {
            crate::EmojiParent::Detached => EmojiParent::Detached,
            crate::EmojiParent::Server { id } => EmojiParent::Server { id },
        }
    }
}

impl From<EmojiParent> for crate::EmojiParent {
    fn from(value: EmojiParent) -> Self {
        match value {
            EmojiParent::Detached => crate::EmojiParent::Detached,
            EmojiParent::Server { id } => crate::EmojiParent::Server { id },
        }
    }
}

impl From<crate::File> for File {
    fn from(value: crate::File) -> Self {
        File {
            id: value.id,
            tag: value.tag,
            filename: value.filename,
            metadata: value.metadata.into(),
            content_type: value.content_type,
            size: value.size,
            deleted: value.deleted,
            reported: value.reported,
            message_id: value.message_id,
            user_id: value.user_id,
            server_id: value.server_id,
            object_id: value.object_id,
        }
    }
}

impl From<crate::Metadata> for Metadata {
    fn from(value: crate::Metadata) -> Self {
        match value {
            crate::Metadata::File => Metadata::File,
            crate::Metadata::Text => Metadata::Text,
            crate::Metadata::Image { width, height } => Metadata::Image {
                width: width as usize,
                height: height as usize,
            },
            crate::Metadata::Video { width, height } => Metadata::Video {
                width: width as usize,
                height: height as usize,
            },
            crate::Metadata::Audio => Metadata::Audio,
        }
    }
}

impl From<crate::Message> for Message {
    fn from(value: crate::Message) -> Self {
        Message {
            id: value.id,
            nonce: value.nonce,
            channel: value.channel,
            author: value.author,
            webhook: value.webhook,
            content: value.content,
            system: value.system.map(|system| system.into()),
            attachments: value
                .attachments
                .map(|v| v.into_iter().map(|f| f.into()).collect()),
            edited: value.edited,
            embeds: value.embeds,
            mentions: value.mentions,
            replies: value.replies,
            reactions: value
                .reactions
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect()))
                .collect(),
            interactions: value.interactions.into(),
            masquerade: value.masquerade.map(|masq| masq.into()),
        }
    }
}

impl From<crate::PartialMessage> for PartialMessage {
    fn from(value: crate::PartialMessage) -> Self {
        PartialMessage {
            id: value.id,
            nonce: value.nonce,
            channel: value.channel,
            author: value.author,
            webhook: value.webhook,
            content: value.content,
            system: value.system.map(|system| system.into()),
            attachments: value
                .attachments
                .map(|v| v.into_iter().map(|f| f.into()).collect()),
            edited: value.edited,
            embeds: value.embeds,
            mentions: value.mentions,
            replies: value.replies,
            reactions: value.reactions.map(|reactions| {
                reactions
                    .into_iter()
                    .map(|(k, v)| (k, v.into_iter().collect()))
                    .collect()
            }),
            interactions: value.interactions.map(|interactions| interactions.into()),
            masquerade: value.masquerade.map(|masq| masq.into()),
        }
    }
}

impl From<crate::SystemMessage> for SystemMessage {
    fn from(value: crate::SystemMessage) -> Self {
        match value {
            crate::SystemMessage::ChannelDescriptionChanged { by } => {
                Self::ChannelDescriptionChanged { by }
            }
            crate::SystemMessage::ChannelIconChanged { by } => Self::ChannelIconChanged { by },
            crate::SystemMessage::ChannelOwnershipChanged { from, to } => {
                Self::ChannelOwnershipChanged { from, to }
            }
            crate::SystemMessage::ChannelRenamed { name, by } => Self::ChannelRenamed { name, by },
            crate::SystemMessage::Text { content } => Self::Text { content },
            crate::SystemMessage::UserAdded { id, by } => Self::UserAdded { id, by },
            crate::SystemMessage::UserBanned { id } => Self::UserBanned { id },
            crate::SystemMessage::UserJoined { id } => Self::UserJoined { id },
            crate::SystemMessage::UserKicked { id } => Self::UserKicked { id },
            crate::SystemMessage::UserLeft { id } => Self::UserLeft { id },
            crate::SystemMessage::UserRemove { id, by } => Self::UserRemove { id, by },
        }
    }
}

impl From<crate::Interactions> for Interactions {
    fn from(value: crate::Interactions) -> Self {
        Interactions {
            reactions: value
                .reactions
                .map(|reactions| reactions.into_iter().collect()),
            restrict_reactions: value.restrict_reactions,
        }
    }
}

impl From<Interactions> for crate::Interactions {
    fn from(value: Interactions) -> Self {
        crate::Interactions {
            reactions: value
                .reactions
                .map(|reactions| reactions.into_iter().collect()),
            restrict_reactions: value.restrict_reactions,
        }
    }
}

impl From<crate::AppendMessage> for AppendMessage {
    fn from(value: crate::AppendMessage) -> Self {
        AppendMessage {
            embeds: value.embeds,
        }
    }
}

impl From<crate::Masquerade> for Masquerade {
    fn from(value: crate::Masquerade) -> Self {
        Masquerade {
            name: value.name,
            avatar: value.avatar,
            colour: value.colour,
        }
    }
}

impl From<Masquerade> for crate::Masquerade {
    fn from(value: Masquerade) -> Self {
        crate::Masquerade {
            name: value.name,
            avatar: value.avatar,
            colour: value.colour,
        }
    }
}

impl From<crate::ServerBan> for ServerBan {
    fn from(value: crate::ServerBan) -> Self {
        ServerBan {
            id: value.id.into(),
            reason: value.reason,
        }
    }
}

impl From<crate::Member> for Member {
    fn from(value: crate::Member) -> Self {
        Member {
            id: value.id.into(),
            joined_at: value.joined_at,
            nickname: value.nickname,
            avatar: value.avatar.map(|f| f.into()),
            roles: value.roles,
            timeout: value.timeout,
        }
    }
}

impl From<crate::PartialMember> for PartialMember {
    fn from(value: crate::PartialMember) -> Self {
        PartialMember {
            id: value.id.map(|id| id.into()),
            joined_at: value.joined_at,
            nickname: value.nickname,
            avatar: value.avatar.map(|f| f.into()),
            roles: value.roles,
            timeout: value.timeout,
        }
    }
}

impl From<crate::MemberCompositeKey> for MemberCompositeKey {
    fn from(value: crate::MemberCompositeKey) -> Self {
        MemberCompositeKey {
            server: value.server,
            user: value.user,
        }
    }
}

impl From<crate::FieldsMember> for FieldsMember {
    fn from(value: crate::FieldsMember) -> Self {
        match value {
            crate::FieldsMember::Avatar => FieldsMember::Avatar,
            crate::FieldsMember::Nickname => FieldsMember::Nickname,
            crate::FieldsMember::Roles => FieldsMember::Roles,
            crate::FieldsMember::Timeout => FieldsMember::Timeout,
        }
    }
}

impl From<crate::RemovalIntention> for RemovalIntention {
    fn from(value: crate::RemovalIntention) -> Self {
        match value {
            crate::RemovalIntention::Ban => RemovalIntention::Ban,
            crate::RemovalIntention::Kick => RemovalIntention::Kick,
            crate::RemovalIntention::Leave => RemovalIntention::Leave,
        }
    }
}

impl From<crate::Server> for Server {
    fn from(value: crate::Server) -> Self {
        Server {
            id: value.id,
            owner: value.owner,
            name: value.name,
            description: value.description,
            channels: value.channels,
            categories: value
                .categories
                .map(|categories| categories.into_iter().map(|v| v.into()).collect()),
            system_messages: value.system_messages.map(|v| v.into()),
            roles: value
                .roles
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            default_permissions: value.default_permissions,
            icon: value.icon.map(|f| f.into()),
            banner: value.banner.map(|f| f.into()),
            flags: value.flags.unwrap_or_default() as u32,
            nsfw: value.nsfw,
            analytics: value.analytics,
            discoverable: value.discoverable,
        }
    }
}

impl From<crate::PartialServer> for PartialServer {
    fn from(value: crate::PartialServer) -> Self {
        PartialServer {
            id: value.id,
            owner: value.owner,
            name: value.name,
            description: value.description,
            channels: value.channels,
            categories: value
                .categories
                .map(|categories| categories.into_iter().map(|v| v.into()).collect()),
            system_messages: value.system_messages.map(|v| v.into()),
            roles: value
                .roles
                .map(|roles| roles.into_iter().map(|(k, v)| (k, v.into())).collect()),
            default_permissions: value.default_permissions,
            icon: value.icon.map(|f| f.into()),
            banner: value.banner.map(|f| f.into()),
            flags: value.flags.map(|v| v as u32),
            nsfw: value.nsfw,
            analytics: value.analytics,
            discoverable: value.discoverable,
        }
    }
}

impl From<crate::FieldsServer> for FieldsServer {
    fn from(value: crate::FieldsServer) -> Self {
        match value {
            crate::FieldsServer::Banner => FieldsServer::Banner,
            crate::FieldsServer::Categories => FieldsServer::Categories,
            crate::FieldsServer::Description => FieldsServer::Description,
            crate::FieldsServer::Icon => FieldsServer::Icon,
            crate::FieldsServer::SystemMessages => FieldsServer::SystemMessages,
        }
    }
}

impl From<crate::Category> for Category {
    fn from(value: crate::Category) -> Self {
        Category {
            id: value.id,
            title: value.title,
            channels: value.channels,
        }
    }
}

impl From<crate::SystemMessageChannels> for SystemMessageChannels {
    fn from(value: crate::SystemMessageChannels) -> Self {
        SystemMessageChannels {
            user_joined: value.user_joined,
            user_left: value.user_left,
            user_kicked: value.user_kicked,
            user_banned: value.user_banned,
        }
    }
}

impl From<crate::Role> for Role {
    fn from(value: crate::Role) -> Self {
        Role {
            name: value.name,
            permissions: value.permissions,
            colour: value.colour,
            hoist: value.hoist,
            rank: value.rank,
        }
    }
}

impl From<crate::PartialRole> for PartialRole {
    fn from(value: crate::PartialRole) -> Self {
        PartialRole {
            name: value.name,
            permissions: value.permissions,
            colour: value.colour,
            hoist: value.hoist,
            rank: value.rank,
        }
    }
}

impl From<crate::FieldsRole> for FieldsRole {
    fn from(value: crate::FieldsRole) -> Self {
        match value {
            crate::FieldsRole::Colour => FieldsRole::Colour,
        }
    }
}

impl crate::User {
    pub async fn into<'a, P>(self, perspective: P) -> User
    where
        P: Into<Option<&'a crate::User>>,
    {
        let relationship = if let Some(perspective) = perspective.into() {
            if perspective.id == self.id {
                RelationshipStatus::User
            } else {
                perspective
                    .relations
                    .as_ref()
                    .map(|relations| {
                        relations
                            .iter()
                            .find(|relationship| relationship.id == self.id)
                            .map(|relationship| relationship.status.clone().into())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default()
            }
        } else {
            RelationshipStatus::None
        };

        // do permission stuff here
        // TODO: implement permissions =)
        let can_see_profile = false;

        User {
            username: self.username,
            discriminator: self.discriminator,
            display_name: self.display_name,
            avatar: self.avatar.map(|file| file.into()),
            relations: vec![],
            badges: self.badges.unwrap_or_default() as u32,
            status: None,
            profile: None,
            flags: self.flags.unwrap_or_default() as u32,
            privileged: self.privileged,
            bot: self.bot.map(|bot| bot.into()),
            relationship,
            online: can_see_profile && revolt_presence::is_online(&self.id).await,
            id: self.id,
        }
    }

    pub async fn into_self(self) -> User {
        User {
            username: self.username,
            discriminator: self.discriminator,
            display_name: self.display_name,
            avatar: self.avatar.map(|file| file.into()),
            relations: self
                .relations
                .map(|relationships| {
                    relationships
                        .into_iter()
                        .map(|relationship| relationship.into())
                        .collect()
                })
                .unwrap_or_default(),
            badges: self.badges.unwrap_or_default() as u32,
            status: self.status.map(|status| status.into()),
            profile: self.profile.map(|profile| profile.into()),
            flags: self.flags.unwrap_or_default() as u32,
            privileged: self.privileged,
            bot: self.bot.map(|bot| bot.into()),
            relationship: RelationshipStatus::User,
            online: revolt_presence::is_online(&self.id).await,
            id: self.id,
        }
    }
}

impl From<crate::PartialUser> for PartialUser {
    fn from(value: crate::PartialUser) -> Self {
        PartialUser {
            username: value.username,
            discriminator: value.discriminator,
            display_name: value.display_name,
            avatar: value.avatar.map(|file| file.into()),
            relations: value.relations.map(|relationships| {
                relationships
                    .into_iter()
                    .map(|relationship| relationship.into())
                    .collect()
            }),
            badges: value.badges.map(|badges| badges as u32),
            status: value.status.map(|status| status.into()),
            profile: value.profile.map(|profile| profile.into()),
            flags: value.flags.map(|flags| flags as u32),
            privileged: value.privileged,
            bot: value.bot.map(|bot| bot.into()),
            relationship: None,
            online: None,
            id: value.id,
        }
    }
}

impl From<FieldsUser> for crate::FieldsUser {
    fn from(value: FieldsUser) -> Self {
        match value {
            FieldsUser::Avatar => crate::FieldsUser::Avatar,
            FieldsUser::ProfileBackground => crate::FieldsUser::ProfileBackground,
            FieldsUser::ProfileContent => crate::FieldsUser::ProfileContent,
            FieldsUser::StatusPresence => crate::FieldsUser::StatusPresence,
            FieldsUser::StatusText => crate::FieldsUser::StatusText,
        }
    }
}

impl From<crate::FieldsUser> for FieldsUser {
    fn from(value: crate::FieldsUser) -> Self {
        match value {
            crate::FieldsUser::Avatar => FieldsUser::Avatar,
            crate::FieldsUser::ProfileBackground => FieldsUser::ProfileBackground,
            crate::FieldsUser::ProfileContent => FieldsUser::ProfileContent,
            crate::FieldsUser::StatusPresence => FieldsUser::StatusPresence,
            crate::FieldsUser::StatusText => FieldsUser::StatusText,
        }
    }
}

impl From<crate::RelationshipStatus> for RelationshipStatus {
    fn from(value: crate::RelationshipStatus) -> Self {
        match value {
            crate::RelationshipStatus::None => RelationshipStatus::None,
            crate::RelationshipStatus::User => RelationshipStatus::User,
            crate::RelationshipStatus::Friend => RelationshipStatus::Friend,
            crate::RelationshipStatus::Outgoing => RelationshipStatus::Outgoing,
            crate::RelationshipStatus::Incoming => RelationshipStatus::Incoming,
            crate::RelationshipStatus::Blocked => RelationshipStatus::Blocked,
            crate::RelationshipStatus::BlockedOther => RelationshipStatus::BlockedOther,
        }
    }
}

impl From<crate::Relationship> for Relationship {
    fn from(value: crate::Relationship) -> Self {
        Self {
            user_id: value.id,
            status: value.status.into(),
        }
    }
}

impl From<crate::Presence> for Presence {
    fn from(value: crate::Presence) -> Self {
        match value {
            crate::Presence::Online => Presence::Online,
            crate::Presence::Idle => Presence::Idle,
            crate::Presence::Focus => Presence::Focus,
            crate::Presence::Busy => Presence::Busy,
            crate::Presence::Invisible => Presence::Invisible,
        }
    }
}

impl From<crate::UserStatus> for UserStatus {
    fn from(value: crate::UserStatus) -> Self {
        UserStatus {
            text: value.text,
            presence: value.presence.map(|presence| presence.into()),
        }
    }
}

impl From<crate::UserProfile> for UserProfile {
    fn from(value: crate::UserProfile) -> Self {
        UserProfile {
            content: value.content,
            background: value.background.map(|file| file.into()),
        }
    }
}

impl From<crate::BotInformation> for BotInformation {
    fn from(value: crate::BotInformation) -> Self {
        BotInformation {
            owner_id: value.owner,
        }
    }
}
