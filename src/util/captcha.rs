use crate::util::variables::{HCAPTCHA_KEY, USE_HCAPTCHA};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct CaptchaResponse {
    success: bool,
}

pub async fn verify(user_token: &Option<String>) -> Result<(), String> {
    if *USE_HCAPTCHA {
        if let Some(token) = user_token {
            let mut map = HashMap::new();
            map.insert("secret", HCAPTCHA_KEY.to_string());
            map.insert("response", token.to_string());

            let client = Client::new();
            if let Ok(response) = client
                .post("https://hcaptcha.com/siteverify")
                .form(&map)
                .send()
                .await
            {
                let result: CaptchaResponse = response
                    .json()
                    .await
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
