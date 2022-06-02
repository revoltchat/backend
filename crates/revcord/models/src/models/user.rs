use crate::{QuarkConversion, to_snowflake, to_ulid};
use revolt_quark::{models::{User as RevoltUser, user::{BotInformation, UserHint}}, Database};
use twilight_model::user::{User as DiscordUser};
use serde::{Serialize, Deserialize};
use rocket_okapi::{JsonSchema, okapi::{schemars::schema::{Schema, SchemaObject}, openapi3::{SecurityScheme, SecuritySchemeData}}, gen::OpenApiGenerator, request::{OpenApiFromRequest, RequestHeaderInput}};
use async_trait::async_trait;
use rocket::{Request, request::{self, FromRequest, Outcome}, http::Status};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct User(DiscordUser);

impl User {
    pub fn into_inner(self) -> DiscordUser {
        self.0
    }
}

#[async_trait]
impl QuarkConversion for User {
    type Type = RevoltUser;

    async fn to_quark(self) -> Self::Type {
        let DiscordUser { bot, id, name , ..} = self.into_inner();

        RevoltUser {
            id: to_ulid(id),
            username: name,
            avatar: None,  // TODO,
            relations: None,
            badges: None,  // TODO,
            status: None,  // TODO,
            profile: None, // TODO,
            flags: None,
            privileged: false,
            bot: if bot {
               Some(BotInformation {
                    owner: "0".to_string()
                })
            } else {
                None
            },
            relationship: None,
            online: None

        }
    }

    async fn from_quark(data: Self::Type) -> Self {
        Self(DiscordUser {
            accent_color: None,
            avatar: None,  // TODO
            bot: data.bot.is_some(),
            banner: None,  // TODO
            discriminator: 1,
            email: None,
            flags: None,
            id: to_snowflake(data.id),
            locale: None,
            mfa_enabled: None,
            name: data.username,
            premium_type: None,
            public_flags: None,
            system: None,
            verified: None
        })
    }
}

impl JsonSchema for User {
    fn schema_name() -> String {
        "User".to_string()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        Schema::Object(SchemaObject::default())
    }
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = rauth::util::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user: &Option<User> = request
            .local_cache_async(async {
                let db = request
                    .rocket()
                    .state::<Database>()
                    .expect("Database state not reachable!");

                let header_token = request
                    .headers()
                    .get("Authorization")
                    .next()
                    .and_then(|x| x.split(' ').nth(1).map(|s| s.to_string()));

                if let Some(token) = header_token {
                    if let Ok(user) = RevoltUser::from_token(db, &token, UserHint::Any).await {
                        return Some(User::from_quark(user).await);
                    }
                }

                None
            })
            .await;

        if let Some(user) = user {
            Outcome::Success(user.clone())
        } else {
            Outcome::Failure((Status::Unauthorized, rauth::util::Error::InvalidSession))
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for User {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let mut requirements = schemars::Map::new();
        requirements.insert("Api Key".to_owned(), vec![]);

        Ok(RequestHeaderInput::Security(
            "Api Key".to_owned(),
            SecurityScheme {
                data: SecuritySchemeData::ApiKey {
                    name: "Authorization".to_owned(),
                    location: "header".to_owned(),
                },
                description: Some("Session Token".to_owned()),
                extensions: schemars::Map::new(),
            },
            requirements,
        ))
    }
}
