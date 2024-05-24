use rand::{thread_rng, Rng};
use revolt_quark::authifier::config::{EmailVerificationConfig, Template};
use revolt_quark::authifier::{models::Account, Authifier};
use revolt_quark::variables::delta::APP_URL;
use revolt_quark::{models::User, Database, EmptyResponse, Error, Result};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataApplication {
    pub email: String,
    pub content: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub country: String,
    pub city: String,
    pub occupation: String,
}

/// # Webhook for application
///
/// Receives an application
#[openapi(tag = "Webhooks")]
#[post("/application", format = "json", data = "<data>")]
pub async fn webhook_receive_application(
    authifier: &State<Authifier>,
    db: &State<Database>,
    data: Json<DataApplication>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;
    // Access data using `data` parameter
    let email = data.email.clone();
    let length = 10;
    let password: String = (0..length)
        .map(|_| {
            let mut rng = thread_rng();
            let choice: u8 = rng.gen_range(0..3);
            match choice {
                0 => rng.gen_range(b'A'..=b'Z') as char, // Uppercase letter
                1 => rng.gen_range(b'a'..=b'z') as char, // Lowercase letter
                _ => rng.gen_range(b'0'..=b'9') as char, // Number
            }
        })
        .collect();
    let password_cloned = password.clone();
    let _account = match Account::new(authifier, email, password, false).await {
        Ok(account) => account,
        Err(err) => {
            return Err(Error::InvalidOperation); // Return HTTP 500 Internal Server Error
        }
    };
    let session_name = data.email.clone() + "_webhook";
    let session = match Account::create_session(&_account, authifier, session_name).await {
        Ok(session) => session,
        Err(err) => {
            return Err(Error::InvalidSession); // Return HTTP 500 Internal Server Error
        }
    };
    let full_name = format!("{} {}", &data.first_name, &data.last_name);
    let username = User::validate_username(full_name)?;
    let mut user = User {
        id: session.user_id,
        discriminator: User::find_discriminator(db, &username, None).await?,
        username,
        ..Default::default()
    };
    let profile = data;
    let mut new_profile = user.profile.take().unwrap_or_default();
    let content = profile.content;
    new_profile.content = Some(content);
    let first_name = profile.first_name;
    new_profile.first_name = Some(first_name);
    let last_name = profile.last_name;
    new_profile.last_name = Some(last_name);
    let phone_number = profile.phone_number;
    new_profile.phone_number = Some(phone_number);
    let country = profile.country;
    new_profile.country = Some(country);
    let city = profile.city;
    new_profile.city = Some(city);
    let occupation = profile.occupation;
    new_profile.occupation = Some(occupation);
    user.profile = Some(new_profile);

    if let EmailVerificationConfig::Enabled {
        templates,
        expiry,
        smtp,
    } = &authifier.config.email_verification
    {
        let mailed = smtp.send_email(
            _account.email.clone(),
            &Template {
                title: "Welcome to Kimani Life!".into(),
                text: include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/",
                    "templates/welcome.txt"
                ))
                .into(),
                url: format!("{}/login/", *APP_URL),
                html: Some(
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/",
                        "templates/welcome.html"
                    ))
                    .into(),
                ),
            },
            json!({
                "email": _account.email.clone(),
                "url": format!("{}/login/", *APP_URL),
                "password": password_cloned
            }),
        );
    }
    db.insert_user(&user).await.map(|_| EmptyResponse)
}
