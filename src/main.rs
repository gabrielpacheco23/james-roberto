use std::error::Error;
use std::fmt;
//use std::fs::File;
//use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use songbird::SerenityInit;
use songbird::Songbird;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    Result as SerenityResult,
};

mod channels_joined_cache;
mod download;
mod my_instants;

use channels_joined_cache::GuildsJoinedCache;

const TOKEN: &str = "ODM5Njk5MjQ2MTk3ODk5Mjk0.YJNc3Q.jAF51HAxlNTac43oIc7KRvVo0p4";

struct Handler {
    guilds_cache: Arc<Mutex<GuildsJoinedCache>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut cache = self.guilds_cache.lock().await;
        let guild_id = msg.guild_id.unwrap();

        if msg.content.starts_with('.') {
            let path = &msg.content[1..];

            if !cache.check_if_present(guild_id) {
                let joined = join(&ctx, &msg).await;

                if let Ok(manager) = joined {
                    let manager = manager.clone();
                    let _ = play_with_manager(&ctx, &msg, path, &manager).await;

                    cache.insert(guild_id);
                }
                // spawn timer to leave voice channel
                let cache_clone = Arc::clone(&self.guilds_cache);
                tokio::spawn(async move {
                    let shutdown_time = 10 * 60; // 10 minutes
                    tokio::time::sleep(Duration::from_secs(shutdown_time)).await;
                    leave(&ctx, &msg).await.unwrap();

                    let mut cache = cache_clone.lock().await;
                    cache.remove(guild_id);
                });
                return;
            }

            let _ = play(&ctx, &msg, path).await;
        } else if msg.content.starts_with("!leave") {
            leave(&ctx, &msg).await.unwrap();

            cache.remove(guild_id);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(ping, instant, audios)]
struct General;

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    // let channels_cached = Arc::new(Mutex::new(ChannelsJoinedCache::new()));

    let mut client = Client::builder(&TOKEN)
        .event_handler(Handler {
            guilds_cache: Arc::new(Mutex::new(GuildsJoinedCache::new())),
            // joined_voice: Arc::new(AtomicBool::new(false)),
        })
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
#[only_in(guilds)]
async fn instant(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Must provide a valid MyInstant url")
                    .await,
            );
            return Ok(());
        }
    };

    let name = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Must provide a single command")
                    .await,
            );
            return Ok(());
        }
    };

    // check if name already exists
    if check_name_exists(&name) {
        check_msg(
            msg.channel_id
                .say(&ctx.http, &format!("Audio named {} already exists", name))
                .await,
        );
        return Ok(());
    }

    let inst = my_instants::MyInstant::from_url(&url).with_name(&name);
    let inst_clone = inst.clone();
    let did_download = tokio::task::spawn_blocking(move || download::download_instant(&inst_clone))
        .await
        .unwrap();

    match did_download {
        true => {
            let d_msg = &format!("Downloaded {}", inst.url);
            check_msg(msg.channel_id.say(&ctx.http, d_msg).await);
            // println!("Downloaded!");
        }
        false => {
            let d_msg = &format!("Failed to download {}", inst.url);
            check_msg(msg.channel_id.say(&ctx.http, d_msg).await);
            // println!("Failed to download!");
        }
    }
    Ok(())
}

// #[command]
// #[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> Result<Arc<Songbird>, impl Error> {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);
            return Err(Box::new(VoiceError));
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client placed in at initialisation.")
        .clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(manager)
}

// just to play faster on first join (not really an improvement)
async fn play_with_manager(
    ctx: &Context,
    msg: &Message,
    url: &str,
    manager: &Songbird,
) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let url = &format!("./audios/{}.mp3", url);
        let path = Path::new(url);
        let source = match songbird::ffmpeg(path).await {
            Ok(source) => source,
            Err(why) => {
                println!("Error starting audio source: {:?}", why);
                check_msg(msg.channel_id.say(&ctx.http, "").await);
                return Err(Box::new(VoiceError));
            }
        };

        handler.play_source(source);

        // check_msg(
        //     msg.channel_id
        //         .say(&ctx.http, &format!("Playing song: {}", path.display()))
        //         .await,
        // );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play audio")
                .await,
        );
    }

    Ok(())
}

// #[command]
// #[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, url: &str) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let url = &format!("./audios/{}.mp3", url);
        let path = Path::new(url);
        let source = match songbird::ffmpeg(path).await {
            Ok(source) => source,
            Err(why) => {
                println!("Error starting audio source: {:?}", why);
                check_msg(msg.channel_id.say(&ctx.http, "").await);
                return Err(Box::new(VoiceError));
            }
        };

        handler.play_source(source);

        // check_msg(
        //     msg.channel_id
        //         .say(&ctx.http, &format!("Playing song: {}", path.display()))
        //         .await,
        // );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play audio")
                .await,
        );
    }

    Ok(())
}

// #[command]
// #[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird voice client placed in at initialisation")
        .clone();

    let has_handler = manager.get(guild_id).is_some();
    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }

        // check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

use serenity::utils::Color;
use std::fs;

#[command]
async fn audios(ctx: &Context, msg: &Message) -> CommandResult {
    let mut audio_names = Vec::new();
    let paths = fs::read_dir("./audios").unwrap();

    for path in paths {
        let name = path.unwrap().path().to_string_lossy().into_owned();
        let (_, name) = name.split_at(9);
        let name = name.split_once('.').unwrap().0.to_owned();

        audio_names.push((name, "\u{2611}", true));
    }

    //let audio_cmds = audio_names.join("\n");

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|embed| {
                embed
                    .title("AVAILABLE AUDIOS \u{1F44D}")
                    .description("")
                    .color(Color::new(0x00FFFF))
                    .fields(audio_names);

                embed
            });
            m
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Pong!").await);
    Ok(())
}

// helpers
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

#[allow(clippy::ptr_arg)]
fn check_name_exists(name: &String) -> bool {
    let mut audio_names = Vec::new();
    let paths = fs::read_dir("./audios").unwrap();

    for path in paths {
        let name = path.unwrap().path().to_string_lossy().into_owned();
        let (_, name) = name.split_at(9);
        let name = name.split_once('.').unwrap().0.to_owned();

        audio_names.push(name);
    }

    audio_names.contains(name)
}

#[derive(Debug)]
struct VoiceError;
impl fmt::Display for VoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "voice error")
    }
}
impl Error for VoiceError {}
