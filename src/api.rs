use std::collections::HashMap;
use super::*;

pub fn to_api_data(data: String) -> Result<ScanningData, CustomError> {
    let api_data: ApiData = match serde_json::from_str(&data) {
        Ok(x) => x,
        Err(_) => return Err(CustomError::API),
    };
    Ok(api_data.scanning_data)
}

pub async fn get(game: String, code: String) -> Result<ScanningData, CustomError> {
    let mut map = HashMap::new();
    map.insert("game_number", game);
    map.insert("code", code);
    map.insert("api_version", "0.1".into());

    let client = reqwest::Client::new();
    let resp = match client.post("https://np.ironhelmet.com/api").form(&map).send().await {
        Ok(resp) => resp.text().await.unwrap(),
        Err(err) => panic!("Error: {}", err)
    };

    let api_data = to_api_data(resp);
    api_data
}

pub fn get_next_time(api_data: ScanningData) -> u32 {
    let mut precent: f32 = api_data.tick_fragment;
    while precent >= 1.0 {
        precent -= 1.0;
    }
    precent = 1.0-precent;
    let out = (api_data.tick_rate as f32*60.0*precent) as u32;
    out
}