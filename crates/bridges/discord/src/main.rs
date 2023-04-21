use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::channel::Channel;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("Received message: {}", msg.content);

        if msg.content == "deez" {
            if let Ok(Channel::Guild(channel)) = msg.channel(&ctx).await {
                channel.send_message(&ctx, |m| {
                    m.content("this is where a message would usually go")
                }).await.unwrap();
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}