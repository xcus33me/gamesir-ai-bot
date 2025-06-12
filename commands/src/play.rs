use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use songbird::{
    input::YoutubeDl, 
    Event, EventContext, EventHandler as VoiceEventHandler, 
    TrackEvent
};
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct TrackEndNotifier {
    chan_id: serenity::ChannelId,
    http: Arc<serenity::Http>,
}

#[serenity::async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_track_list) = ctx {
            let _ = self    
                .chan_id
                .say(&self.http, "Track finished!")
                .await;
        }
        None
    }
}

pub struct TrackErrorNotifier {
    chan_id: serenity::ChannelId,
    http: Arc<serenity::Http>,
}

#[serenity::async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_track_list) = ctx {
            let _ = self    
                .chan_id
                .say(&self.http, "Error playing track!")
                .await;
        }
        None
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL YouTube video"] url: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command should be used in server")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird is not initialized")?;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        // Validate URL
        if !url.contains("youtube.com") && !url.contains("youtu.be") {
            ctx.say("Please provide a valid YouTube URL!").await?;
            return Ok(());
        }

        ctx.say("🔄 Loading track...").await?;

        // Create YoutubeDl source and play
        let src = YoutubeDl::new(reqwest::Client::new(), url.clone());
        
        info!("Successfully created YoutubeDl source for: {}", url);

        // Play the source
        let track_handle = handler.play(src.into());

        // Add event handlers
        let _ = track_handle.add_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                chan_id: ctx.channel_id(),
                http: ctx.serenity_context().http.clone(),
            },
        );

        let _ = track_handle.add_event(
            Event::Track(TrackEvent::Error),
            TrackErrorNotifier {
                chan_id: ctx.channel_id(),
                http: ctx.serenity_context().http.clone(),
            },
        );

        ctx.say(format!("✅ Added to queue: {}", url)).await?;
        
        // Log track info
        info!("Track added to queue. Queue length: {}", handler.queue().len());
        
    } else {
        ctx.say("❌ Not connected to voice channel. Use `!join` first").await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command should be used in server")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird is not initialized")?;

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        
        queue.stop();
        ctx.say("⏹️ Playback stopped and queue cleared").await?;
    } else {
        ctx.say("❌ Not connected to voice channel").await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command should be used in server")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird is not initialized")?;

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        
        if queue.len() > 0 {
            let _ = queue.skip();
            ctx.say("⏭️ Track skipped").await?;
        } else {
            ctx.say("❌ Queue is empty").await?;
        }
    } else {
        ctx.say("❌ Not connected to voice channel").await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command should be used in server")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird is not initialized")?;

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        
        let queue_len = queue.len();
        
        if queue_len == 0 {
            ctx.say("📭 Queue is empty").await?;
        } else {
            let current = queue.current();
            let mut response = format!("🎵 **Queue ({} tracks)**\n", queue_len);
            
            if let Some(_track) = current {
                response.push_str("▶️ **Now playing:** Current track\n");
            }
            
            if queue_len > 1 {
                response.push_str(&format!("📄 **Next:** {} track(s) in queue", queue_len - 1));
            }
            
            ctx.say(response).await?;
        }
    } else {
        ctx.say("❌ Not connected to voice channel").await?;
    }

    Ok(())
}