use std::option;

use poise::FrameworkBuilder;
use tracing::info;

mod play;
mod other;

pub struct Data; // glob data

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub fn init_framework() -> poise::Framework<Data, Error> {
    let options = poise::FrameworkOptions{
        commands: vec![
            other::join(),
            other::leave(),
            play::play(),
            play::skip(),
            play::queue(),
        ],
        prefix_options: poise::PrefixFrameworkOptions { 
            prefix: Some("!".into()),
            ..Default::default()
        },
        ..Default::default()
    };    
    
    poise::Framework::builder()
        .options(options)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Bot {} is ready!", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build()
}