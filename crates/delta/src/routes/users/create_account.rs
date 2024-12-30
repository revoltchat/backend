//! Create a new account
//! POST /account/create
use revolt_quark::authifier::config::ShieldValidationInput;
use revolt_quark::authifier::{models::Account, Authifier, Error, Result};
use revolt_quark::Db;
use rocket::serde::json::Json;
use rocket::State;
use rocket_empty::EmptyResponse;
use serde::{Deserialize, Serialize};

/// # Account Data
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataCreateAccount {
    /// Valid email address
    pub email: String,
    /// Password
    pub password: String,
    /// Invite code
    pub invite: Option<String>,
    /// Captcha verification code
    pub captcha: Option<String>,
}

/// # Create Account
///
/// Create a new account.
#[openapi(tag = "Account")]
#[post("/create", data = "<data>")]
pub async fn req(
    db: &Db,
    authifier: &State<Authifier>,
    data: Json<DataCreateAccount>,
    mut shield: ShieldValidationInput,
) -> Result<EmptyResponse> {
    let data = data.into_inner();
    let email = &data.email.clone();
    // Find authorized email
    if let Err(err) = db.fetch_white_list(email).await {
        return Err(Error::EmailFailed);
    }
    // Check Captcha token
    authifier.config.captcha.check(data.captcha).await?;

    // Validate the request
    shield.email = Some(data.email.to_string());
    authifier.config.shield.validate(shield).await?;

    // Make sure email is valid and not blocked
    authifier
        .config
        .email_block_list
        .validate_email(&data.email)?;

    // Ensure password is safe to use
    authifier
        .config
        .password_scanning
        .assert_safe(&data.password)
        .await?;

    // If required, fetch valid invite
    let invite = if authifier.config.invite_only {
        if let Some(invite) = data.invite {
            Some(authifier.database.find_invite(&invite).await?)
        } else {
            return Err(Error::MissingInvite);
        }
    } else {
        None
    };

    // Create account
    let account = Account::new(authifier, data.email, data.password, true).await?;

    // Use up the invite
    if let Some(mut invite) = invite {
        invite.claimed_by = Some(account.id);
        invite.used = true;

        authifier.database.save_invite(&invite).await?;
    }

    Ok(EmptyResponse)
}
