use reqwest::blocking::Client;
use std::collections::HashMap;
use std::env;

pub fn send_email(target: String, subject: String, body: String, html: String) -> Result<(), ()> {
	let mut map = HashMap::new();
	map.insert("target", target.clone());
	map.insert("subject", subject);
	map.insert("body", body);
	map.insert("html", html);

	let client = Client::new();
	match client.post("http://192.168.0.26:3838/send")
		.json(&map)
		.send() {
			Ok(_) => Ok(()),
			Err(_) => Err(())
		}
}

fn public_uri() -> String {
	env::var("PUBLIC_URI").expect("PUBLIC_URI not in environment variables!")
}

pub fn send_verification_email(email: String, code: String) -> bool {
	let url = format!("{}/api/account/verify/{}", public_uri(), code);
	match send_email(
		email,
		"Verify your email!".to_string(),
		format!("Verify your email here: {}", url).to_string(),
		format!("<a href=\"{}\">Click to verify your email!</a>", url).to_string()
	) {
		Ok(_) => true,
		Err(_) => false,
	}
}

pub fn send_welcome_email(email: String, username: String) -> bool {
	match send_email(
		email,
		"Welcome to REVOLT!".to_string(),
		format!("Welcome, {}! You can now use REVOLT.", username.clone()).to_string(),
		format!("<b>Welcome, {}!</b><br/>You can now use REVOLT.<br/><a href=\"{}\">Go to REVOLT</a>", username.clone(), public_uri()).to_string()
	) {
		Ok(_) => true,
		Err(_) => false,
	}
}
