use authifier::config::{ResolveIp, Shield};

use super::variables::delta::{
    APP_URL, AUTHIFIER_SHIELD_KEY, HCAPTCHA_KEY, INVITE_ONLY, SMTP_FROM, SMTP_HOST, SMTP_PASSWORD,
    SMTP_USERNAME, USE_EMAIL, USE_HCAPTCHA,
};

use crate::authifier::config::{
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
                        html: Some(include_str!(crate::asset!("templates/verify.html")).into()),
                    },
                    reset: Template {
                        title: "Reset your Revolt password.".into(),
                        text: include_str!(crate::asset!("templates/reset.txt")).into(),
                        url: format!("{}/login/reset/", *APP_URL),
                        html: Some(include_str!(crate::asset!("templates/reset.html")).into()),
                    },
                    deletion: Template {
                        title: "Confirm account deletion.".into(),
                        text: include_str!(crate::asset!("templates/deletion.txt")).into(),
                        url: format!("{}/delete/", *APP_URL),
                        html: Some(include_str!(crate::asset!("templates/deletion.html")).into()),
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

    if let Some(api_key) = &*AUTHIFIER_SHIELD_KEY {
        config.shield = Shield::Enabled {
            api_key: api_key.to_string(),
            strict: false,
        };
    }

    if std::env::var("TRUST_CLOUDFLARE")
        .map(|x| x == "1")
        .unwrap_or_default()
    {
        config.resolve_ip = ResolveIp::Cloudflare;
    }

    config
}
