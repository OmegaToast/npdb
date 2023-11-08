use std::{fs::{self, File}, io::Read};
use super::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Raw{
    data: Vec<PlayerData>,
}

pub fn save(data: Vec<PlayerData>) {
    let raw = Raw {data};
    let str = serde_json::to_string::<Raw>(&raw).unwrap();
    fs::write("saves.json", str).unwrap();
}

pub fn load() -> Vec<PlayerData> {    
    let mut file = File::open("saves.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    match serde_json::from_str::<Raw>(&data) {
        Ok(x) => x.data,
        Err(_) => {
            print!("Error reaing file, please try again");
            let _ = std::io::stdin();
            save(Vec::new());
            panic!();
        },
    }
}