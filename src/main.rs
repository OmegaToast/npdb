use std::{sync::{Mutex, Arc}, thread, time::SystemTime};
use poise::serenity_prelude::{self as serenity, Http, json::JsonMap, GuildChannel};
use serde::{Serialize, Deserialize};

mod scanning_data_format;
use scanning_data_format::*;
mod commands;
mod api;
mod update;
mod save;
mod token;

#[derive(Debug)]
enum CustomError {
    Locking,
    Indexing,
    Time,
    Say(Error),
    Flush,
    AsMut,
    API
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerData {
    code: String,
    game_number: String,
    thread: GuildChannel,
    api: ScanningData,
    time: SystemTime,
    next_tick_wait: u32,
    user_id: u64,
    just_started: bool,
    update: bool,
    known_attacks: Vec<(i32, i32)>,
    game_started: bool,
    players: Vec<String>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
struct Data {
    games: Arc<Mutex<Vec<PlayerData>>>
} // User data, which is stored and accessible in all command invocations


#[tokio::main]
async fn main() {

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::ping(),
                commands::start(),
                commands::end(),
                commands::refresh(),
                commands::get_channel(),
                commands::change_key(),
                ],
            pre_command: |ctx| {
                Box::pin(async move {
                    println!("Executing command {}", ctx.command().qualified_name);
                })
            },
            ..Default::default()
        })
        .token(token::TOKEN)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let data = Data { games:Arc::new(Mutex::new(save::load()))};
                let arc_copy = Arc::clone(&data.games);
                tokio::spawn(update::loop_run(arc_copy));
                Ok(data)
            })
        });
    
    println!("Running");

    framework.run().await.unwrap();
}