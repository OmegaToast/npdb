use std::{fs::{self, File}, io::Read};
use super::*;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Raw{
    data: Vec<PlayerData>,
}

pub fn backup() {
    let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect();
    let mut file = File::open("saves.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    fs::write("backup/".to_owned()+&id, data).unwrap();
    println!("New save backup: {}", id);
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
            println!("Error reaing file, please try again");
            backup();
            save(Vec::new());
            panic!();
        },
    }
}