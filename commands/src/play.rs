use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use songbird::{
    input::{YoutubeDl, Input},
    Event, EventContext, EventHandler as VoiceEventHandler,
    TrackEvent,
};
use std::sync::Arc;
use tracing::{error, info, warn};

async fn get_direct_soundcloud_url(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let output = tokio::process::Command::new("yt-dlp")
        .args([
            "--no-playlist",
            "-f", "http_mp3_1_0/http_mp3/best[protocol=http]",
            "--get-url",
            "--extractor-args", "soundcloud:force_api_v2",
            url
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    let direct_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    info!("Got direct URL from yt-dlp: {}", direct_url);
    Ok(direct_url)
}

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
        if let EventContext::Track(track_list) = ctx {
            if let Some(track) = track_list.first() {
                let track_handle = track.1;
                let track_info = track_handle.get_info().await;
                error!("Track error occurred: {:?}", track_info);
                
                let error_msg = match track_info {
                    Ok(info) => format!("❌ Error playing track: {:?}", info.playing),
                    Err(e) => format!("❌ Error playing track: {}", e),
                };
                
                let _ = self
                    .chan_id
                    .say(&self.http, &error_msg)
                    .await;
            } else {
                error!("Track error occurred - no track info available");
                let _ = self
                    .chan_id
                    .say(&self.http, "❌ Error playing track!")
                    .await;
            }
        }
        None
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Music URL (Youtube, Soundcloud and etc...)"] url: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command should be used in server")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird is not initialized")?;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let is_url = url.starts_with("http://") || url.starts_with("https://");

        ctx.say("🔄 Loading track...").await?;

        let request = if is_url {
            info!("Processing URL: {}", url);
            url.clone()
        } else {
            info!("Searching for: {}", url);
            format!("ytsearch:{}", url)
        };

        let is_soundcloud = url.contains("soundcloud.com");
        
        let track_handle = if is_soundcloud {
            info!("Detected SoundCloud URL, using special handling");
            
            match get_direct_soundcloud_url(&url).await {
                Ok(direct_url) => {
                    info!("Using direct URL for SoundCloud: {}", direct_url);
                    let yt_source = YoutubeDl::new(reqwest::Client::new(), direct_url)
                        .user_args(vec![
                            "--no-playlist".into(),
                            "--socket-timeout".into(),
                            "60".into(),
                            "--retries".into(),
                            "5".into(),
                        ]);
                    handler.play_input(yt_source.into())
                }
                Err(e) => {
                    warn!("Failed to get direct URL, falling back to normal method: {}", e);
                    let yt_source = YoutubeDl::new(reqwest::Client::new(), request)
                        .user_args(vec![
                            "--no-playlist".into(),
                            "-f".into(),
                            "http_mp3_1_0/http_mp3/best[protocol=http][protocol!~m3u8][protocol!~hls]".into(),
                            "--extractor-args".into(),
                            "soundcloud:force_api_v2".into(),
                            "--no-check-certificates".into(),
                            "--socket-timeout".into(),
                            "60".into(),
                            "--retries".into(),
                            "5".into(),
                            "--fragment-retries".into(),
                            "5".into(),
                            "--abort-on-unavailable-fragment".into(),
                        ]);
                    handler.play_input(yt_source.into())
                }
            }
        } else {
            let yt_source = YoutubeDl::new(reqwest::Client::new(), request)
                .user_args(vec![
                    "--no-playlist".into(),
                    "-f".into(),
                    "http_mp3/best[protocol!=m3u8][protocol!=hls]/bestaudio/best".into(),
                    "--extractor-args".into(),
                    "soundcloud:force_api_v2".into(),
                    "--no-check-certificates".into(),
                    "--prefer-free-formats".into(),
                    "--socket-timeout".into(),
                    "30".into(),
                    "--retries".into(),
                    "3".into(),
                ]);
            handler.play_input(yt_source.into())
        };
        
        match track_handle.get_info().await {
            Ok(info) => {
                info!("Track created successfully: {:?}", info.playing);
            }
            Err(e) => {
                error!("Failed to get track info: {}", e);
                ctx.say("❌ Failed to create track").await?;
                return Ok(());
            }
        }

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

        let queue_info = handler.queue().current_queue();
        info!("Queue info: {} tracks", queue_info.len());

        if is_url {
            ctx.say(format!("✅ Added to queue: {}", url)).await?;
        } else {
            ctx.say(format!("✅ Searching and adding: {}", url)).await?;
        }

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
