#![allow(non_snake_case, dead_code)]

use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use ethers::prelude::*;
use ethers::providers::{Http, Provider};
//use log::{error, info};
//use log4rs;
use lazy_static::lazy_static;
use regex::Regex;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::*;
use tokio::sync::Mutex;

mod config;
mod telegam;

lazy_static! {
     static  ref CONF: config::Config =config::load_config("src/config.json");
}
lazy_static! {
    static ref CONF1: Arc<Mutex<config::Config>> = Arc::new(Mutex::new(config::load_config("src/config.json")));
}


lazy_static! {
    static ref DATA_UNI_V2: Vec<config::Dev> = config::load_data("src/data.json");
}
lazy_static! {
    static ref BOT: Bot = Bot::new("394446592:AAFhXQksbbdD1eJDtQmyrkqCwxj24ma7TzA");
}

abigen!(Token, "abi/tokenabi.json");

#[tokio::main]
async fn main() -> Result<(), ()> {
    //let config: Mutex<config::Config> = Mutex::new(config::load_config("src/config.json"));

    tokio::spawn(telegam::operate(BOT.clone(), &CONF1));
    let ethereum: config::Ethereum = CONF.blockchain.ethereum.clone();
    // Logging
    // log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    //  let logger = log::logger();

    let provider = Provider::<Http>::try_from(ethereum.http).unwrap();
    let client = Arc::new(provider.clone());
    let mut last_block = client.get_block_number().await.unwrap();
    loop {
        sleep(Duration::from_secs(19));
        let curr_block = provider.get_block_number().await.unwrap();
        for num in last_block.as_u64()..=curr_block.as_u64() {
            // println!("{}", num);
            let client_copy = Arc::clone(&client);
            tokio::spawn(check_block(num, client_copy));
        }
        last_block = curr_block + 1;
    }
}

async fn check_block(block_number: u64, client: Arc<Provider<Http>>) -> Result<(), ()> {
    let result = client.get_block_with_txs(block_number).await.unwrap();
    if let Some(block) = result {
        for element in &block.transactions {
            //show_dev_bydata(symbol.clone(), creates, dev).await;
            if element.to.is_none() {
                let creates = ethers::utils::get_contract_address(element.from, element.nonce);
                //println!("{:?}", creates);
                //let address: Address = "0x5712b6d0ed669eb45ae7c60e5b3ad185bf04ce15".parse().unwrap();
                let client_copy = Arc::clone(&client);
                let new_token = Token::new(creates, client_copy);
                let symbol = new_token.symbol().call().await;
                match symbol {
                    Ok(symbol) => {
                        dbg!(&symbol);
                        let nft = check_nft(new_token.clone(), creates, symbol.clone()).await;
                        if !nft {
                            check_token(new_token, creates, symbol.clone()).await;
                        }
                        for dev in DATA_UNI_V2.iter() {
                            let creator: H160 = dev.creator.parse().unwrap();
                            if element.from == creator {
                                let _ = show_dev_bydata(symbol.clone(), creates, dev).await;
                            }
                        }
                    }
                    Err(_) => {
                        //println!("method not exist");
                    }
                }
            }
        }
    }
    Ok(())
}

async fn check_nft(token_contract: Token<Provider<Http>>, creates: H160, symbol: String) -> bool {
    let original_u32: u32 = 0x1a3f02f4;
    let u32_as_bytes: [u8; 4] = original_u32.to_be_bytes();
    //println!("{:?}", u32_as_bytes);
    //("supportsInterface", (0x80ac58cd,0xc8c6c9f3,0x1a3f02f4))
    let token_interface = token_contract.supports_interface(u32_as_bytes).call().await;
    match token_interface {
        Err(_) => {
            return false;
        }
        _ => {
            println!("{:?}", token_interface);
        }
    }
    let name = token_contract.name().call().await.unwrap_or_default();
    // Escape ticker
    let escaped_symbol = html::escape(&symbol);
    // Escape name
    let escaped_name = html::escape(&name);
    let _ = show_nft(escaped_symbol.clone(), escaped_name.clone(), creates).await;
    true
}

async fn check_token(token_contract: Token<Provider<Http>>, creates: H160, symbol: String) -> bool {
    let name = token_contract.name().call().await.unwrap_or_default();
    dbg!(&name);
    let supply = token_contract
        .total_supply()
        .call()
        .await
        .unwrap_or_default();
    let decimals = token_contract.decimals().call().await.unwrap_or_default();
    // Escape ticker
    let escaped_symbol = html::escape(&symbol);
    // Escape name
    let escaped_name = html::escape(&name);
    let _ = show_token(
        escaped_symbol.clone(),
        escaped_name.clone(),
        supply,
        decimals,
        creates,
    );


    //Сравниваем с именем

    let mut found = false;
    for str in &CONF.main.names {
        dbg!(str);
        dbg!(&name);
        let pattern = Regex::new(&format!(r"(?xmi)({})", str.as_str())).unwrap(); //regex::escape()
        if pattern.is_match(name.as_str()) {
            found = true;
        }
    }

    //Сравниваем с тикером

    //let mut found = false;
    for str in &CONF.main.tickers {
        let pattern = Regex::new(&format!(r"(?xmi)({})", regex::escape(str.as_str()))).unwrap();
        if pattern.is_match(symbol.as_str()) {
            found = true;
        }
    }
    if found {
        show_dev(
            escaped_symbol,
            escaped_name,
            creates,
            "Found by Name or Ticker",
        )
            .await.unwrap()
    }
    dbg!(found);

    true
}

async fn show_nft(symbol: String, name: String, creates: H160) -> Result<(), ()> {
    let msg = format!(
        "
ticker: <code>{}</code>
name: <code>{}</code>
exp: <a href=\"{}{:?}\">{:?}</a>",
        symbol, name, CONF.blockchain.ethereum.explorer, creates, creates
    );
    //let escaped_msg = teloxide::utils::markdown::escape(&msg);
    BOT.send_message(CONF.main.group.clone(), msg)
        .message_thread_id(CONF.blockchain.ethereum.topicnft.parse().unwrap())
        .disable_web_page_preview(true)
        .disable_notification(true)
        .parse_mode(ParseMode::Html)
        .await
        .unwrap();
    Ok(())
}

async fn show_token(symbol: String, name: String, supply: U256, decimals: u8, creates: H160) -> Result<(), ()> {
    //println!("{:?}", creates.clone());
    let format_supply = supply
        .checked_div(U256::from(10).pow(decimals.into()))
        .unwrap();
    let msg = format!(
        "
ticker: <code>{symbol}</code>
name: <code>{name}</code>
supply: <code>{format_supply:.1}</code> dec: <code>{decimals}</code>
exp: <a href=\"{}{:?}\">{:?}</a>",
        CONF.blockchain.ethereum.explorer, creates, creates
    );
    //let escaped_msg = teloxide::utils::markdown::escape(&msg);
    BOT.send_message(CONF.main.group.clone(), msg)
        .message_thread_id(CONF.blockchain.ethereum.topic.parse().unwrap())
        .disable_web_page_preview(true)
        .disable_notification(true)
        .parse_mode(ParseMode::Html)
        .await
        .unwrap();
    Ok(())
}

async fn show_dev(symbol: String, name: String, creates: H160, reason: &str) -> Result<(), ()> {
    let msg = format!(
        "
<b>{reason}</b>

ticker: <code>{symbol}</code>
name: <code>{name}</code>

exp: <a href=\"{}{:?}\">{:?}</a>",
        CONF.blockchain.ethereum.explorer, creates, creates
    );

    BOT.send_message(CONF.main.devGroup.clone(), msg)
        .disable_web_page_preview(true)
        .disable_notification(true)
        .parse_mode(ParseMode::Html)
        .await
        .unwrap();
    Ok(())
}

async fn show_dev_bydata(symbol: String, creates: H160, dev: &config::Dev) -> Result<(), ()> {
    let dev_creator = html::escape(&dev.creator);
    let dev_name = &dev.name;
    let dev_created_at_timestamp = html::escape(&dev.created_at_timestamp);
    let dev_volume_usd = html::escape(&dev.volume_usd);
    let dev_tx_count = html::escape(&dev.tx_count);
    let dev_id_pair = html::escape(&dev.id_pair);
    let dev_id_token = html::escape(&dev.id_token);
    let dex = html::escape("https://dexscreener.com/ethereum/");
    let exp = &CONF.blockchain.ethereum.explorer.clone();
    let msg = format!(
        "
        <b>Dev Found by Data</b>
        creator: <a href=\"{exp}{dev_creator}\">{dev_creator}</a>
        
        
        New token
        symbol: {symbol}
        etherscan: <a href=\"{exp}{:?}\">{:?}</a>
        
        Old token
        name: {dev_name}
        date created: {dev_created_at_timestamp}
        volume usd: {dev_volume_usd}
        tx count: {dev_tx_count}
        dexscreener: <a href=\"{dex}{dev_id_pair}\">{dev_id_pair}</a>
        etherscan: <a href=\"{exp}{dev_id_token}\">{dev_id_token}</a>

",
        creates, creates
    );
    //println!("{:?}",msg);

    //let escaped_msg =  teloxide::utils::html::escape(&msg);
    //println!("{:?}",escaped_msg);
    BOT.send_message(CONF.main.devGroup.clone(), msg)
        .parse_mode(ParseMode::Html)
        .disable_web_page_preview(true)
        .await
        .unwrap();
    Ok(())
}

/* async fn show_token(
    symbol_v: String,
    name: String,
    supply: String,
    decimals: String,
    id: String,
    token_explorer: String,
    token_group: String,
    text_settings: String,
    api: Api,
) {
    let mut send_msg = SendMessage::new(token_group, "ticker: `")
        .parse_mode("MarkdownV2")
        .disable_web_page_preview(true);
    let msg_text = format!(
        "{}\`
name: \`
{}
\`
supply: \`
{}
\` dec: \`
{}
\`
link: [{}]({})",
        symbol_v, name, supply, decimals, id, token_explorer
    );
    send_msg.text(msg_text).disable_notification();

        info!("Ticker: {}, Name: {}, Supply: {}, Decimals: {}, Link: {}{}", symbol_v, name, supply, decimals, token_explorer, id);
    });
} */


/*
fn escape_all() -> String {
    let str = r#"Hello ? /world"#;
/*
    let escaped_string = String::from_utf8(
        str.chars()
            .flat_map(|c| {
                escape_default(c as u8);
                if c == "/".chars().nth(0).unwrap() {}
            })
            .collect::<Vec<u8>>(),
    );*/
    dbg!(escaped_string);
    //println!("{:?}", escaped_string.clone());
    let new_string = escaped_string.unwrap();
    /*      new_string = new_string.replace(".", r#"\."#);
    println!("{:?}",new_string.clone());
    new_string = new_string.replace("-", r#"\-"#);
    println!("{:?}",new_string.clone()); */
    new_string
}*/
