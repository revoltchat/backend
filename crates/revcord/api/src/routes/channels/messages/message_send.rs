use std::collections::HashMap;

use revolt_quark::{Database, Ref, Result, EmptyResponse, Error};
use revcord_models::{ user::User};
use rocket::{serde::json::Json, form::{FromForm, Form, ValueField, Result as FormResult, DataField, Errors as FormErrors, Error as FormError}, data::{FromData, Outcome, Data}, http::Status, fs::TempFile, };
use rocket_okapi::request::OpenApiFromData;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, FromForm)]
pub struct MessagePostForm<'r> {
    pub payload_json: Option<MessagePostJson>,
    pub files: HashMap<String, TempFile<'r>>
}

#[derive(Serialize, Deserialize, Validate, JsonSchema, Debug, Clone)]
pub struct MessagePostJson {
    pub content: Option<String>,
    pub tts: Option<bool>,
    // pub embeds: Option<Vec<DiscordEmbed>>,
    // pub allows_mentions: Option<DiscordAllowedMentions>,
    // pub message_reference: Option<DiscordMessageReference>,
    // pub attachments: Option<Vec<DiscordAttachment>>,
    pub flags: Option<u64>
}

#[async_trait]
impl<'r> rocket::form::FromFormField<'r> for MessagePostJson {  // specify the full path because otherwise Json::from_data messes up
    fn from_value(field: ValueField<'r>) -> FormResult<'r, Self> {
        serde_json::from_str(field.value).map_err(|e| {
            let mut errors = FormErrors::new();
            errors.push(FormError::validation(format!("{e:?}")));
            errors
        })
    }

    async fn from_data(_: DataField<'r, '_>) -> FormResult<'r, Self> {
        let mut errors = FormErrors::new();
        errors.push(FormError::validation("payload_json must be a string"));
        Err(errors)
    }
}

#[derive(Debug)]
pub enum MessageBody<'r> {
    Form(MessagePostForm<'r>),
    Json(MessagePostJson)
}

#[async_trait]
impl<'r> FromData<'r> for MessageBody<'r> {
    type Error = Error;

    async fn from_data(req: &'r rocket::Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        if let Some(content_type) = req.headers().get_one("content-type") {
            println!("{content_type}");
            match content_type.split(';').next().unwrap() {
                "application/json" => {
                    match Json::<MessagePostJson>::from_data(req, data).await {
                        Outcome::Success(json) => Outcome::Success(Self::Json(json.into_inner())),
                        Outcome::Failure(_) => Outcome::Failure((Status::BadRequest, Error::LabelMe)),
                        _ => unreachable!()
                    }
                },
                "multipart/form-data" => {
                    match Form::<MessagePostForm>::from_data(req, data).await {
                        Outcome::Success(form) => Outcome::Success(Self::Form(form.into_inner())),
                        Outcome::Failure(_) => Outcome::Failure((Status::BadRequest, Error::LabelMe)),
                        _ => unreachable!()
                    }
                },
                _ => Outcome::Failure((Status::BadRequest, Error::LabelMe))
            }
        } else {
            Outcome::Failure((Status::BadRequest, Error::LabelMe))
        }
    }
}

impl<'r> OpenApiFromData<'r> for MessageBody<'r> {
    fn request_body(gen: &mut rocket_okapi::gen::OpenApiGenerator) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::RequestBody> {
        Ok(rocket_okapi::okapi::openapi3::RequestBody::default())
    }
}

#[openapi(tag = "Messages")]
#[post("/<target>/messages", data="<body>")]
pub async fn req(user: User, target: Ref, body: MessageBody<'_>) -> Result<EmptyResponse> {
    println!("{body:?}");

    Ok(EmptyResponse)
}
