use std::env;
use dotenvy::dotenv;

const DISCORD_TOKEN: &str = "DISCORD_TOKEN";
const AI_API_KEY: &str = "AI_API_TOKEN";


pub struct Config {
    pub discord_token: String,
    pub api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Config, Box<dyn std::error::Error>> {
        dotenv().expect(".env file not found");
        Ok(Config {
            discord_token: env::var(DISCORD_TOKEN).expect("Expected a discord token in the environment"),
            api_key: env::var(AI_API_KEY).expect("Expected an api token in the environment"),
        })
    }
}