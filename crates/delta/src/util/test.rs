use std::collections::HashMap;

use authifier::{
    models::{Account, EmailVerification, Session},
    Authifier,
};
use futures::StreamExt;
use lapin::types::AMQPValue;
use rand::Rng;
use revolt_broker::event_stream;
use revolt_database::{
    events::client::EventV1, Channel, Database, Member, Message, Server, User, AMQP,
};
use revolt_database::{util::idempotency::IdempotencyKey, Role};
use revolt_models::v0;
use revolt_permissions::OverrideField;
use rocket::http::Header;
use rocket::local::asynchronous::{Client, LocalRequest, LocalResponse};

pub struct TestHarness {
    pub client: Client,
    authifier: Authifier,
    pub db: Database,
    pub amqp: AMQP,
    consumer: lapin::Consumer,
    event_buffer: Vec<(String, EventV1)>,
}

impl TestHarness {
    pub async fn new() -> TestHarness {
        let config = revolt_config::config().await;

        let client = Client::tracked(crate::web().await)
            .await
            .expect("valid rocket instance");

        let tag: String = rand::thread_rng()
            .sample_iter::<char, _>(&rand::distributions::Standard)
            .take(32)
            .collect();

        static QUEUE_NAME: &str = "revolt.events";
        let consumer = event_stream::get_channel()
            .await
            .basic_consume(QUEUE_NAME, &tag, Default::default(), Default::default())
            .await
            .unwrap();

        let db = client
            .rocket()
            .state::<Database>()
            .expect("`Database`")
            .clone();

        let authifier = client
            .rocket()
            .state::<Authifier>()
            .expect("`Authifier`")
            .clone();

        let connection = amqprs::connection::Connection::open(
            &amqprs::connection::OpenConnectionArguments::new(
                &config.rabbit.host,
                config.rabbit.port,
                &config.rabbit.username,
                &config.rabbit.password,
            ),
        )
        .await
        .unwrap();
        let channel = connection.open_channel(None).await.unwrap();

        let amqp = AMQP::new(connection, channel);

        TestHarness {
            client,
            authifier,
            db,
            amqp,
            consumer,
            event_buffer: vec![],
        }
    }

    pub fn rand_string() -> String {
        let mut rng = rand::thread_rng();
        (&mut rng)
            .sample_iter(rand::distributions::Alphanumeric)
            .take(20)
            .map(char::from)
            .collect()
    }

    pub async fn new_user(&self) -> (Account, Session, User) {
        let user = User::create(&self.db, TestHarness::rand_string(), None, None)
            .await
            .expect("`User`");

        let (account, session) = self.account_from_user(user.id.clone()).await;

        (account, session, user)
    }

    pub async fn account_from_user(&self, id: String) -> (Account, Session) {
        let account = Account {
            id,
            email: format!("{}@revolt.chat", TestHarness::rand_string()),
            password: Default::default(),
            email_normalised: Default::default(),
            deletion: None,
            disabled: false,
            lockout: None,
            mfa: Default::default(),
            password_reset: None,
            verification: EmailVerification::Verified,
        };

        self.authifier
            .database
            .save_account(&account)
            .await
            .expect("`Account`");

        let session = account
            .create_session(&self.authifier, String::new())
            .await
            .expect("`Session`");

        (account, session)
    }

    pub async fn new_server(&self, user: &User) -> (Server, Vec<Channel>) {
        Server::create(
            &self.db,
            v0::DataCreateServer {
                name: "Test Server".to_string(),
                ..Default::default()
            },
            user,
            true,
        )
        .await
        .expect("Failed to create test server")
    }

    pub async fn new_role(
        &self,
        server: &Server,
        rank: i64,
        overrides: Option<OverrideField>,
    ) -> (String, Role) {
        let role = Role {
            name: TestHarness::rand_string(),
            permissions: overrides.unwrap_or(OverrideField { a: 0, d: 0 }),
            rank,
            colour: None,
            hoist: false,
        };

        let id = role
            .create(&self.db, &server.id)
            .await
            .expect("Failed to create test role");

        (id, role)
    }

    pub async fn new_channel(&self, server: &Server) -> Channel {
        Channel::create_server_channel(
            &self.db,
            &mut server.clone(),
            v0::DataCreateServerChannel {
                channel_type: v0::LegacyServerChannelType::Text,
                name: "Test Channel".to_string(),
                description: None,
                nsfw: Some(false),
            },
            true,
        )
        .await
        .expect("Failed to make test channel")
    }

    pub async fn new_message(
        &self,
        user: &User,
        server: &Server,
        channels: Vec<Channel>,
    ) -> (Channel, Member, Message) {
        let (member, channels) = Member::create(&self.db, server, user, Some(channels))
            .await
            .expect("Failed to create member");
        let channel = &channels[0];
        let message = Message::create_from_api(
            &self.db,
            None,
            channel.clone(),
            v0::DataMessageSend {
                content: Some("Test message".to_string()),
                nonce: None,
                attachments: None,
                replies: None,
                embeds: None,
                masquerade: None,
                interactions: None,
                flags: None,
            },
            v0::MessageAuthor::User(&user.clone().into(&self.db, Some(user)).await),
            Some(user.clone().into(&self.db, Some(user)).await),
            Some(member.clone().into()),
            user.limits().await,
            IdempotencyKey::unchecked_from_string("0".to_string()),
            false,
            false,
        )
        .await
        .expect("Failed to create message");
        (channel.clone(), member, message)
    }

    pub async fn with_session(session: Session, request: LocalRequest<'_>) -> LocalResponse<'_> {
        request
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await
    }

    pub async fn wait_for_event<F>(&mut self, topic: &str, predicate: F) -> EventV1
    where
        F: Fn(&EventV1) -> bool,
    {
        for (msg_topic, event) in &self.event_buffer {
            if topic == msg_topic && predicate(event) {
                // does not remove from buffer
                return event.clone();
            }
        }

        while let Some(Ok(delivery)) = self.consumer.next().await {
            let headers: HashMap<String, AMQPValue> = delivery
                .properties
                .headers()
                .as_ref()
                .map(|table| {
                    table
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.clone()))
                        .collect()
                })
                .unwrap_or_default();

            let filter_value = headers
                .get("x-stream-filter-value")
                .expect("`x-stream-filter-value` not present in message!")
                .as_long_string()
                .unwrap()
                .to_string();

            let payload: EventV1 = rmp_serde::from_slice(&delivery.data).unwrap();

            if topic == filter_value && predicate(&payload) {
                return payload;
            }

            self.event_buffer.push((filter_value, payload));
        }

        // WARNING: if predicate is never satisfied, this will never return
        //          should add a timeout for events so tests can fail gracefully

        unreachable!()
    }

    pub async fn wait_for_message(&mut self, channel_id: &str) -> v0::Message {
        dbg!(&self.event_buffer);

        match self
            .wait_for_event(channel_id, |event| match event {
                EventV1::Message(v0::Message { channel, .. }) => channel == channel_id,
                _ => false,
            })
            .await
        {
            EventV1::Message(message) => message,
            _ => unreachable!(),
        }
    }
}
