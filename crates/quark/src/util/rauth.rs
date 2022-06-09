use super::variables::delta::{
    APP_URL, HCAPTCHA_KEY, INVITE_ONLY, SMTP_FROM, SMTP_HOST, SMTP_PASSWORD, SMTP_USERNAME,
    USE_EMAIL, USE_HCAPTCHA,
};

use crate::rauth::config::{
    Captcha, Config, EmailVerificationConfig, SMTPSettings, Template, Templates,
};

pub fn config() -> Config {
    let mut config = Config {
        email_verification: if *USE_EMAIL {
            EmailVerificationConfig::Enabled {
                smtp: SMTPSettings {
                    from: (*SMTP_FROM).to_string(),
                    host: (*SMTP_HOST).to_string(),
                    username: (*SMTP_USERNAME).to_string(),
                    password: (*SMTP_PASSWORD).to_string(),
                    reply_to: Some("support@revolt.chat".into()),
                    port: None,
                    use_tls: None,
                },
                expiry: Default::default(),
                templates: Templates {
                    verify: Template {
                        title: "Verify your Revolt account.".into(),
                        text: include_str!(crate::asset!("templates/verify.txt")).into(),
                        url: format!("{}/login/verify/", *APP_URL),
                        html: None,
                    },
                    reset: Template {
                        title: "Reset your Revolt password.".into(),
                        text: include_str!(crate::asset!("templates/reset.txt")).into(),
                        url: format!("{}/login/reset/", *APP_URL),
                        html: None,
                    },
                    deletion: Template {
                        title: "Confirm account deletion.".into(),
                        text: include_str!(crate::asset!("templates/deletion.txt")).into(),
                        url: format!("{}/delete/", *APP_URL),
                        html: None,
                    },
                    welcome: None,
                },
            }
        } else {
            EmailVerificationConfig::Disabled
        },
        ..Default::default()
    };

    if *INVITE_ONLY {
        config.invite_only = true;
    }

    if *USE_HCAPTCHA {
        config.captcha = Captcha::HCaptcha {
            secret: HCAPTCHA_KEY.clone(),
        };
    }

    config
}
