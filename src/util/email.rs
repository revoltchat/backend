use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use super::variables::{PUBLIC_URL, SMTP_FROM, SMTP_HOST, SMTP_PASSWORD, SMTP_USERNAME};

lazy_static! {
    static ref MAILER: lettre::transport::smtp::SmtpTransport =
        SmtpTransport::relay(SMTP_HOST.as_ref())
            .unwrap()
            .credentials(Credentials::new(
                SMTP_USERNAME.to_string(),
                SMTP_PASSWORD.to_string()
            ))
            .build();
}

fn send(message: Message) -> Result<(), String> {
    MAILER
        .send(&message)
        .map_err(|err| format!("Failed to send email! {}", err.to_string()))?;

    Ok(())
}

fn generate_multipart(text: &str, html: &str) -> MultiPart {
    MultiPart::mixed().multipart(
        MultiPart::alternative()
            .singlepart(
                SinglePart::quoted_printable()
                    .header(header::ContentType(
                        "text/plain; charset=utf8".parse().unwrap(),
                    ))
                    .body(text),
            )
            .multipart(
                MultiPart::related().singlepart(
                    SinglePart::eight_bit()
                        .header(header::ContentType(
                            "text/html; charset=utf8".parse().unwrap(),
                        ))
                        .body(html),
                ),
            ),
    )
}

pub fn send_verification_email(email: String, code: String) -> Result<(), String> {
    let url = format!("{}/api/account/verify/{}", PUBLIC_URL.to_string(), code);
    let email = Message::builder()
        .from(SMTP_FROM.to_string().parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Verify your email!")
        .multipart(generate_multipart(
            &format!("Verify your email here: {}", url),
            &format!("<a href=\"{}\">Click to verify your email!</a>", url),
        ))
        .unwrap();

    send(email)
}

pub fn send_password_reset(email: String, code: String) -> Result<(), String> {
    let url = format!("{}/api/account/reset/{}", PUBLIC_URL.to_string(), code);
    let email = Message::builder()
        .from(SMTP_FROM.to_string().parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Reset your password.")
        .multipart(generate_multipart(
            &format!("Reset your password here: {}", url),
            &format!("<a href=\"{}\">Click to reset your password!</a>", url),
        ))
        .unwrap();

    send(email)
}

pub fn send_welcome_email(email: String, username: String) -> Result<(), String> {
    let email = Message::builder()
        .from(SMTP_FROM.to_string().parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Welcome to REVOLT!")
        .multipart(
        generate_multipart(
            &format!("Welcome, {}! You can now use REVOLT.", username),
            &format!(
                    "<b>Welcome, {}!</b><br/>You can now use REVOLT.<br/><a href=\"{}\">Go to REVOLT</a>",
                    username,
                    PUBLIC_URL.to_string()
                )
            )
        )
        .unwrap();

    send(email)
}
