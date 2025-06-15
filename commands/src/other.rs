use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use songbird::{input::YoutubeDl, Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use std::sync::Arc;
use tracing::{error, info};

#[poise::command(prefix_command, slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("The command must be used in the server")?;
    
    let channel_id = ctx
        .guild()
        .ok_or("Failed to get server info")?
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or("You should be in the voice channel")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird is not initialized")?;

    if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
        ctx.say(format!("Joined the <#{}>", channel_id)).await?;
    } else {
        ctx.say("Error when joining the channel").await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("The command must be used in the server")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird is not initialized")?;

    if manager.get(guild_id).is_some() {
        if let Err(e) = manager.remove(guild_id).await {
            ctx.say(format!("Error when disconnecting: {}", e)).await?;
        } else {
            ctx.say("Left the voice channel").await?;
        }
    } else {
        ctx.say("Not connected to the voice channel").await?;
    }

    Ok(())
}