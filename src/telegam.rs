use std::str::FromStr;
use std::sync::Arc;

use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::Mutex;

use crate::config::{self, Config};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Show,
    #[command(description = "handle a username.")]
    Add(String),
    #[command(description = "handle a username and an age.")]//, parse_with = "split")]
    Delete(u8),

}

#[derive(Clone)]
struct DeleteCommand(u8);

impl FromStr for DeleteCommand {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u8>() {
            Ok(num) => Ok(DeleteCommand(num)),
            Err(err) => Err(err),
        }
    }
}

pub async fn operate(bot: Bot, config: &Arc<Mutex<config::Config>>) {
    //let config = Arc::clone(&config);
    teloxide::repl(bot, |bot: Bot, msg: Message, mut config: config::Config| async move {
        let a = config.main.names.remove(1usize);
        dbg! {config.main.names};
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
        .await;
}


async fn answer(bot: Bot, msg: Message, cmd: Command, mut config: &Arc<Mutex<config::Config>>) -> ResponseResult<()> {
    if msg.chat.id.to_string() == config.lock().await.main.devGroup {
        match cmd {
            Command::Show => {
                let mut text = String::new();

                for (index, item) in config.lock().await.main.names.iter().enumerate() {
                    let line = format!(" {}: {}\n", index, item);
                    text.push_str(&line);
                }

                bot.send_message(msg.chat.id, text).await?
            }
            Command::Add(el) => {
                config.lock().await.main.names.push(el.to_string());
                bot.send_message(msg.chat.id, format!("Element {} added", el)).await?
            }
            Command::Delete(num) => {
                let in_usize: usize = num.try_into().unwrap_or_else(|_| 999usize);
                let name = config.lock().await.main.names.remove(in_usize.clone());
                bot.send_message(msg.chat.id, format!("Deleted element: {}", name))
                    .await?
            }
        };
    }
    Ok(())
}












