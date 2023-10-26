use std::collections::HashMap;

use cached::proc_macro::cached;
use config::{Config, File, FileFormat};
use futures_locks::RwLock;
use once_cell::sync::Lazy;
use serde::Deserialize;

static CONFIG_BUILDER: Lazy<RwLock<Config>> = Lazy::new(|| {
    RwLock::new({
        let mut builder = Config::builder().add_source(File::from_str(
            include_str!("../Revolt.toml"),
            FileFormat::Toml,
        ));

        if std::env::var("TEST_DB").is_ok() {
            builder = builder.add_source(File::from_str(
                include_str!("../Revolt.test.toml"),
                FileFormat::Toml,
            ));
        } else if std::path::Path::new("Revolt.toml").exists() {
            builder = builder.add_source(File::new("Revolt.toml", FileFormat::Toml));
        }

        builder.build().unwrap()
    })
});

// https://gifbox.me/view/gT5mqxYKCZv-twilight-meow

#[derive(Deserialize, Debug, Clone)]
pub struct Database {
    pub mongodb: String,
    pub redis: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Hosts {
    pub app: String,
    pub api: String,
    pub events: String,
    pub autumn: String,
    pub january: String,
    pub voso_legacy: String,
    pub voso_legacy_ws: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiRegistration {
    pub invite_only: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiSmtp {
    pub host: String,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiVapid {
    pub private_key: String,
    pub public_key: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiFcm {
    pub api_key: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiSecurityCaptcha {
    pub hcaptcha_key: String,
    pub hcaptcha_sitekey: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiSecurity {
    pub authifier_shield_key: String,
    pub voso_legacy_token: String,
    pub captcha: ApiSecurityCaptcha,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiWorkers {
    pub max_concurrent_connections: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Api {
    pub staging: bool,
    pub registration: ApiRegistration,
    pub smtp: ApiSmtp,
    pub vapid: ApiVapid,
    pub fcm: ApiFcm,
    pub security: ApiSecurity,
    pub workers: ApiWorkers,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FeaturesLimits {
    pub group_size: usize,
    pub bots: usize,
    pub message_length: usize,
    pub message_replies: usize,
    pub message_attachments: usize,
    pub message_embeds: usize,
    pub message_reactions: usize,
    pub servers: usize,
    pub server_emoji: usize,
    pub server_roles: usize,
    pub server_channels: usize,

    pub attachment_size: usize,
    pub avatar_size: usize,
    pub background_size: usize,
    pub icon_size: usize,
    pub banner_size: usize,
    pub emoji_size: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FeaturesLimitsCollection {
    pub default: FeaturesLimits,

    #[serde(flatten)]
    pub roles: HashMap<String, FeaturesLimits>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Features {
    pub limits: FeaturesLimitsCollection,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub database: Database,
    pub hosts: Hosts,
    pub api: Api,
    pub features: Features,
}

pub async fn init() {
    println!(
        ":: Revolt Configuration ::\n\x1b[32m{:?}\x1b[0m",
        config().await
    );
}

pub async fn read() -> Config {
    CONFIG_BUILDER.read().await.clone()
}

#[cached(time = 30)]
pub async fn config() -> Settings {
    read().await.try_deserialize::<Settings>().unwrap()
}

#[cfg(feature = "test")]
#[cfg(test)]
mod tests {
    use crate::init;

    #[async_std::test]
    async fn it_works() {
        init().await;
    }
}
