use reqwest::blocking::Client;
use std::collections::HashMap;

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
