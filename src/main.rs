mod config;

use commands::init_framework;
use config::Config;

use serenity::{all::{Gateway, GatewayIntents, StandardFramework}, Client};
use songbird::SerenityInit;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = init_framework();

    let mut client = Client::builder(config.discord_token, intents)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Failed to create client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}
