use std::error::Error;

use serde::{Serialize, Deserialize};
use serde_json::Value;
use ureq;

pub struct ServerState {
    pub ip: String,
    pub status: bool,
    pub player_count: u64,
    pub max_players: u64,
    pub players: Vec<String>,
}

impl ServerState {
    pub fn new(ip: &str) -> Self {
        Self {
            ip: String::from(ip),
            status: false,
            player_count: 0,
            max_players: 0,
            players: Vec::new(),
        }
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let ip: &str = &self.ip;
        let response = ureq::get(
                format!("https://api.mcstatus.io/v2/status/java/{ip}")
            )
            .call()?
            .body_mut()
            .read_json::<Value>()?;
   
        self.status = response["online"].as_bool().unwrap_or(false);
        self.player_count = response["players"]["online"].as_u64().unwrap_or(0);
        self.max_players = response["players"]["max"].as_u64().unwrap_or(0);

        self.players = Vec::new();
        if let Some(players) = response["players"]["list"].as_array() {
            for player in players {
                self.players.push(player["name_clean"].to_string());
            }
        }

        Ok(())
    }
}
