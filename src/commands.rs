use super::*;

/// This is to check if the bot is active
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // print!("-");
    ctx.send(|u| {
        u.content(format!("Pong")).ephemeral(true)
    }).await?;

    Ok(())
}

#[derive(Debug, poise::Modal)]
#[name = "Start Game"]
struct GameModal {
    #[name = "API Key"]
    api_key: String,
    #[name = "Game ID"]
    game_id: String,
}

/// Start a game
#[poise::command(slash_command)]
pub async fn start(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    use poise::Modal as _; // why?

    // create http referance
    let http = Http::new(token::TOKEN);

    // send form for user
    let data = GameModal::execute(ctx).await?;
    let modal_result: GameModal;
    match data {
        Some(x) => modal_result = x,
        None => {
            return Ok(())
        },
    };
    println!("{:?}", modal_result);
    
    // get api data
    let scanning_data = api::get(modal_result.game_id.clone(), modal_result.api_key.clone()).await.unwrap();
    
    // create new private thread
    let mut json = JsonMap::new();
    json.insert("name".into(), scanning_data.name.clone().into());
    let channel_id = http.create_private_thread(1169300509681786993, &json).await.expect("Unable to make thread");
    
    // add user to thread
    http.add_thread_channel_member(channel_id.id.0, ctx.author().id.0).await.expect("Unable to add user");
    
    {
        let mut games_list = ctx.data().games.lock().unwrap();
        games_list.insert(0,
            PlayerData {
                code: modal_result.api_key.clone(),
                game_number: modal_result.game_id.clone(),
                thread: channel_id.clone(),
                api: scanning_data.clone(),
                time: SystemTime::now(),
                next_tick_wait: api::get_next_time(scanning_data.clone()),
                user_id: ctx.author().id.0,
                just_started: true,
                update: false,
                known_attacks: Vec::new(),
                game_started: scanning_data.started,
                players: {
                    let mut vec = Vec::new();
                    for (_, s) in scanning_data.players.clone() {
                        if !s.alias.is_empty() {vec.insert(0, s.alias);}
                    }
                    vec
                },
            });
    }

    // print data
    channel_id.say(&http, format!("Watching game: **{}**", scanning_data.name.clone())).await?;

    Ok(())
}

/// End a game (must be in the game thread)
#[poise::command(slash_command)]
pub async fn end(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    // create http referance
    let http = Http::new(token::TOKEN);
    let current_thread = ctx.guild_channel().await.unwrap();
    let mut delete: bool = false;
    {
        // check if this is a game thread an not a real channel
        let mut games_list = ctx.data().games.lock().unwrap();
        for i in 0..games_list.len() {
            if games_list.get(i).unwrap().thread.id.0 == current_thread.id.0 {
                games_list.remove(i);
                delete = true;
                break;
            }
        }
    }
    if delete {
        http.delete_channel(current_thread.id.0).await?;
    }
    else {
        ctx.send(|u| {
            u.content(format!("You cannot delete this channel. If you think this is a mistake, please contact the owner of this bot.")).ephemeral(true)
        }).await?;
    }

    Ok(())
}

/// Queries API and refreshes memory
#[poise::command(slash_command)]
pub async fn refresh(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    let mut game = false;
    {
        // check if this is a game thread an not a real channel
        let mut games_list = ctx.data().games.lock().unwrap();
        for i in 0..games_list.len() {
            if games_list.get(i).unwrap().thread.id.0 == ctx.channel_id().0 {
                game = true;
                games_list.get_mut(i).unwrap().update = true;
                games_list.get_mut(i).unwrap().known_attacks = Vec::new();
                break;
            }
        }
    }
    if game {
        ctx.send(|u| {
            u.content(format!("Refreshing")).ephemeral(true)
        }).await?;
    } else {
        ctx.send(|u| {
            u.content(format!("This is not a valid game")).ephemeral(true)
        }).await?;
    }


    Ok(())
}

/// Change an api key when you make a new one
#[poise::command(slash_command)]
pub async fn change_key(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[description = "API Code"]
    api: String,
) -> Result<(), Error> {
    {
        // check if this is a game thread an not a real channel
        let mut games_list = ctx.data().games.lock().unwrap();
        for i in 0..games_list.len() {
            if games_list.get(i).unwrap().thread.id.0 == ctx.channel_id().0 {
                games_list.get_mut(i).unwrap().code = api;
                break;
            }
        }
    }
    ctx.send(|u| {
        u.content(format!("Succesfully changed API key")).ephemeral(true)
    }).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn get_channel(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    println!("{}", serde_json::to_string(&ctx.guild_channel().await.unwrap()).unwrap());
    Ok(())
}