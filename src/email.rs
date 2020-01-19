/*use lettre::smtp::authentication::{ Credentials, Mechanism };
use lettre::{ SmtpClient, SmtpTransport, Transport, SendableEmail, Envelope, EmailAddress, SendmailTransport };
use lettre_email::EmailBuilder;
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;

use std::env;
use std::sync::Mutex;

use once_cell::sync::OnceCell;
static mut MAILER: OnceCell<Mutex<SmtpTransport>> = OnceCell::new();

pub fn connect() {
	let host      = env::var("SMTP_HOST").expect("SMTP_HOST not in environment variables!");
	let port: u32 = env::var("SMTP_PORT").expect("SMTP_PORT not in environment variables!").parse().unwrap();
	let domain    = env::var("SMTP_DOMAIN").expect("SMTP_DOMAIN not in environment variables!");
	let username  = env::var("SMTP_USERNAME").expect("SMTP_USERNAME not in environment variables!");

	let mailer = SmtpClient::new_simple(&host).unwrap()
        .hello_name(ClientId::Domain(domain))
        .credentials(Credentials::new(username, env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not in environment variables!")))
        .smtp_utf8(true)
        .authentication_mechanism(Mechanism::Plain)
        .connection_reuse(ConnectionReuseParameters::ReuseUnlimited).transport();

	unsafe {
		if let Err(_) = MAILER.set(Mutex::new(mailer)) {
			panic!("Failed to set global mailer!");
		}
	}
}

pub fn send(recipient: String, title: String, contents: String) {
	let username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME not in environment variables!");
	let email = EmailBuilder::new()
		.to(recipient)
		.from(username)
		.subject(title)
		.text(contents)
		.build()
		.unwrap();

	let email = SendableEmail::new(
		Envelope::new(
			Some(EmailAddress::new(username).unwrap()),
			vec![EmailAddress::new(recipient).unwrap()],
		)
		.unwrap(),
		title,
		contents.into_bytes(),
	);

	let mut sender = SendmailTransport::new();
    let result = sender.send(email);

	unsafe {
		MAILER.get_mut().unwrap().lock().unwrap().send(email).expect("Failed to send email!");
	}
}*/

pub fn connect() {
	//
}

use sendmail;
use std::env;

pub fn send(recipient: &str, title: &str, contents: &str) {
	sendmail::email::send(
		&env::var("SMTP_USERNAME").expect("SMTP_USERNAME not in environment variables!"),
		&vec![ recipient ][..],
		title,
		contents
	).unwrap();
}