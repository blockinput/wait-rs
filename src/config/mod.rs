use serde_json::Result;
use serde::Deserialize;
use std::fs;


pub fn load_config(file:impl Into<String>) -> Config {
    let config_file = fs::read_to_string(file.into())
    .expect("Should have been able to read the file");
    
    let value:Result<Config>= serde_json::from_str(config_file.as_str());
    let config = value.unwrap();
    config
}

pub fn load_data(file:impl Into<String>) -> Vec<Dev> {
    let config_file = fs::read_to_string(file.into())
    .expect("Should have been able to read the file");
    
    let value:Result<Vec<Dev>>= serde_json::from_str(config_file.as_str());
    let config = value.unwrap();
    config
}
// Структура для хранения конфигурации из файла config.json
#[derive(Debug, Deserialize)]
pub struct Blockchain {
    pub ethereum: Ethereum,
    palm: Palm,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ethereum {
    pub name: String,
    pub topic: String,
    pub topicnft: String,
    pub wss: String,
    pub http: String,
    pub explorer: String,
}

#[derive(Debug, Deserialize)]
struct Palm {}

 #[derive(Debug, Deserialize)]
pub struct Main {
    pub group: String,
    pub devGroup: String,
    pub BOT_TOKEN: String,
    pub tickers: Vec<String>,
    pub names: Vec<String>,
    pub dev_list: Vec<Developer>,
}

#[derive(Debug, Deserialize)]
pub struct Developer {
    address: String,
    about: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub blockchain: Blockchain,
    pub main: Main,
}


//data.json

#[derive(Debug, Deserialize)]
pub struct Dev {
pub    creator: String,
pub    name: String,
pub    created_at_timestamp: String,
pub    volume_usd: String,
pub    tx_count: String,
pub    symbol: String,
pub    id_pair: String,
pub    id_token: String,
}

