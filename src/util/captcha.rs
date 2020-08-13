use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::env;

#[derive(Serialize, Deserialize)]
struct CaptchaResponse {
    success: bool
}

pub fn verify(user_token: &Option<String>) -> Result<(), String> {
    if let Ok(key) = env::var("HCAPTCHA_KEY") {
        if let Some(token) = user_token {
            let mut map = HashMap::new();
            map.insert("secret", key);
            map.insert("response", token.to_string());

            let client = Client::new();
            if let Ok(response) = client
                .post("https://hcaptcha.com/siteverify")
                .json(&map)
                .send()
            {
                let result: CaptchaResponse = response
                    .json()
                    .map_err(|_| "Failed to deserialise captcha result.".to_string())?;

                if result.success {
                    Ok(())
                } else {
                    Err("Unsuccessful captcha verification".to_string())
                }
            } else {
                Err("Failed to verify with hCaptcha".to_string())
            }
        } else {
            Err("Missing hCaptcha token!".to_string())
        }
    } else {
        Ok(())
    }
}
