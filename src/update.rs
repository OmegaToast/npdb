use std::time::Duration;

use super::*;

pub async fn loop_run(data: Arc<Mutex<Vec<PlayerData>>>) {
    loop {
        match run(data.clone()).await {
            Ok(_) => todo!(),
            Err(x) => println!("ERROR -> {:?}", x)
        }
    }
}

async fn run(data: Arc<Mutex<Vec<PlayerData>>>) -> Result<(), CustomError> {

    let http = Http::new(token::TOKEN);

    loop {
        let games_list_ref;
        match data.lock() {
            Ok(x) => games_list_ref = x.clone(),
            Err(_) => return Err(CustomError::Locking),
        }

        for i in 0..games_list_ref.len() {
            let mut output_string = String::new();

            // check if api should be gotten
            let mut get_api = false;
            let player_data_ref = games_list_ref.get(i).ok_or_else(|| {CustomError::Indexing})?;
            let elapsed = player_data_ref.time.elapsed().or_else(|_| {Err(CustomError::Locking)})?;
            if (elapsed.as_secs() > player_data_ref.next_tick_wait as u64 + 300) | &player_data_ref.update {get_api=true;}
            if player_data_ref.just_started {get_api = false;} // api was gotten already for making the thread

            // gotting the api
            let scanning_data: ScanningData;
            match get_api {
                true => {
                    scanning_data = match api::get(player_data_ref.game_number.clone(), player_data_ref.code.clone()).await {
                        Ok(x) => x,
                        Err(_) => {
                            let _ = http.delete_channel(player_data_ref.thread.id.0).await;
                            data.lock().unwrap().remove(i);
                            return Err(CustomError::API)
                        },
                    }
                },
                false => scanning_data = player_data_ref.api.clone(),
            }

            // main logic
            {
                // lock the mutex
                let mut games_list = data.lock().or_else(|_| {Err(CustomError::Locking)})?;
                let mut player_data = games_list.get_mut(i).ok_or_else(|| {CustomError::Indexing})?;

                if (elapsed.as_secs() > player_data.next_tick_wait as u64 + 300) | &player_data.just_started | &player_data.update {
                    if !player_data.game_started { // Game still not started
                        let result = game_not_started(&mut player_data, scanning_data.clone())?;
                        output_string += &result;
                        
                    }
                    // game checks
                    else {
                        let result = game_started(&mut player_data, scanning_data.clone())?;
                        output_string += &result;
                    }
                }
                
                // reset vars
                player_data.just_started = false;
                player_data.update = false;
                player_data.api = scanning_data;

                // save
                save::save(games_list.clone());
            }
            // send messages
            if !output_string.is_empty() {
                match player_data_ref.thread.say(&http, output_string).await {
                    Ok(_) => (),
                    Err(x) => {
                        data.lock().unwrap().remove(i);
                        return  Err(CustomError::Say(Box::new(x)))
                    },
                }
            }
        }
        match std::io::Write::flush(&mut std::io::stdout()) {
            Ok(_) => (),
            Err(_) => return  Err(CustomError::Flush),
        }
        thread::sleep(Duration::from_secs(5));
    }
}

fn game_not_started(player_data: &mut PlayerData, scanning_data: ScanningData) -> Result<String, CustomError> {
    let mut output_string = String::new();

    // time
    player_data.time = SystemTime::now();
    player_data.next_tick_wait = api::get_next_time(scanning_data.clone());

    // new players
    for (_, s) in scanning_data.players {
        let mut new_player = true;
        if s.alias.is_empty() | player_data.players.clone().contains(&s.alias) {
            new_player = false;
        }
        if new_player {
            output_string += &format!("New player, {}, just joined!\n", s.alias);
            player_data.players.insert(0, s.alias.clone());
        }
    }

    // game start
    if scanning_data.started {
        output_string += &format!("<@{}> **Your game has started!**\n", player_data.user_id);
        player_data.game_started = true;
    }
    
    Ok(output_string)
}

fn game_started(player_data: &mut PlayerData, scanning_data: ScanningData) -> Result<String, CustomError> {
    let mut output_string = String::new();

    // turn based
    if let Some(x) = scanning_data.turn_based {
        if x == 1 {
            if let Some(x) = scanning_data.turn_based_time_out {
                if let Some(y) = player_data.api.turn_based_time_out {
                    if x > y {
                        output_string += &format!("<@{}>A turn has passed!\n", player_data.user_id)
                    }
                }
            }
        }
    }
    
    // time
    player_data.time = SystemTime::now();
    player_data.next_tick_wait = api::get_next_time(scanning_data.clone());

    // FA
    let mut friends: Vec<String> = Vec::new();
    for (_, p) in scanning_data.players.clone() { // major rework needed here
        if let Some(w) = p.war {
            for f in w {
                if f.1 == 0 {
                    friends.insert(0, f.0);
                } else if f.1 == 1 {
                    let friend = match scanning_data.players.get(&f.0) {
                        Some(x) => x,
                        None => todo!(),
                    };
                    output_string += &format!("{} has requested a Formal Alliance\n", friend.alias);
                }
            }
        }
    }

    // fleets
    let puid = scanning_data.player_uid;
    for (_, carrier) in scanning_data.fleets {
        if let Some(x) = carrier.o.get(0) {
            if let Some(star) = scanning_data.stars.get(&x[1].to_string()) {
                let mut new = true;
                for (cuid, suid) in player_data.known_attacks.clone() {
                    if (carrier.uid == cuid) & (star.uid == suid) {
                        new = false;
                    }
                }
                if friends.contains(&carrier.puid.to_string()) {
                    new = false;
                } 
                if (star.puid == puid) & (puid != carrier.puid) & new {
                    let attacker_name = match &scanning_data.players.get(&carrier.puid.to_string()) {
                        Some(x) => &x.alias,
                        None => return Err(CustomError::Indexing),
                    };
                    output_string += &format!("<@{}> Tick:{} **You are under attack by {} on {}**\n", player_data.user_id, scanning_data.tick, attacker_name, star.n);
                    player_data.known_attacks.insert(0, (carrier.uid, star.uid));
                }
            }
        }
    }

    // production cycle

    Ok(output_string)
}