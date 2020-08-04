use reqwest::blocking::Client;
use std::collections::HashMap;
use std::env;

fn public_uri() -> String {
    env::var("PUBLIC_URI").expect("PUBLIC_URI not in environment variables!")
}

fn portal() -> String {
    env::var("PORTAL_URL").expect("PORTAL_URL not in environment variables!")
}

pub fn send_email(target: String, subject: String, body: String, html: String) -> Result<(), ()> {
    let mut map = HashMap::new();
    map.insert("target", target.clone());
    map.insert("subject", subject);
    map.insert("body", body);
    map.insert("html", html);

    let client = Client::new();
    match client.post(&portal()).json(&map).send() {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub fn send_verification_email(email: String, code: String) -> bool {
    let url = format!("{}/api/account/verify/{}", public_uri(), code);
    send_email(
        email,
        "Verify your email!".to_string(),
        format!("Verify your email here: {}", url),
        format!("<a href=\"{}\">Click to verify your email!</a>", url),
    )
    .is_ok()
}

pub fn send_password_reset(email: String, code: String) -> bool {
    let url = format!("{}/api/account/reset/{}", public_uri(), code);
    send_email(
        email,
        "Reset your password.".to_string(),
        format!("Reset your password here: {}", url),
        format!("<a href=\"{}\">Click to reset your password!</a>", url),
    )
    .is_ok()
}

pub fn send_welcome_email(email: String, username: String) -> bool {
    send_email(
        email,
        "Welcome to REVOLT!".to_string(),
        format!("Welcome, {}! You can now use REVOLT.", username.clone()),
        format!(
            "<b>Welcome, {}!</b><br/>You can now use REVOLT.<br/><a href=\"{}\">Go to REVOLT</a>",
            username.clone(),
            public_uri()
        ),
    )
    .is_ok()
}
