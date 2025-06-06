mod config;
mod ai;

use crate::{ai::client::{AIClient, Model}, config::Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    let client = AIClient::new(Model::Gemini);
    
    let resp = client.ask("Why is kokolang?").await?;
    println!("{}", resp);

    Ok(())
}