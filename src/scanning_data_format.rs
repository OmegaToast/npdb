use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiData {
    pub scanning_data: ScanningData
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanningData {
    pub name: String,
    pub player_uid: i32,
    pub started: bool,
    pub fleets: HashMap<String, Carrier>,
    pub stars: HashMap<String, Star>,
    pub players: HashMap<String, Players>,
    pub tick: u32,
    pub tick_rate: u32,
    pub tick_fragment: f32,
    pub production_rate: u32,
    pub game_over: Option<i32>,
    pub turn_based: Option<i32>,
    pub turn_based_time_out: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Carrier {
    pub uid: i32,
    pub n: String,
    pub puid: i32,
    pub st: i32,
    pub o: Vec<[i32; 4]>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Star {
    pub uid: i32,
    pub n: String,
    pub puid: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Players {
    pub alias: String,
    pub war: Option<HashMap<String, i32>>,
    pub researching: Option<String>,
    pub cash: Option<i32>,
    pub tech: Option<HashMap<String, Tech>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tech {
    pub level: i32,
    pub research: Option<i32>,
}