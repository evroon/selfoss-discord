extern crate dotenv;

use std::collections::HashMap;

use config::Config;
use dotenv::dotenv;

mod config;
mod discord;
mod selfoss;
mod utils;

use discord::adapter::{get_channels, post_message};
use selfoss::{
    adapter::{get_tree, mark_items_as_read},
    models::SelfossItem,
};

use crate::{discord::adapter::create_channel, utils::deserialize_string_from_env};

async fn send_messages(
    config: &Config,
    item_list: Vec<SelfossItem>,
    mut channel_map: HashMap<String, String>,
) {
    println!("Found max {} messages to send", item_list.len());

    for item in &item_list {
        let name = item.clone().get_discord_channel_name();
        let channel = channel_map.get(name.clone().as_str());
        let content = item.clone().get_discord_message_content();

        if channel.is_none() {
            let c = create_channel(&config, name.clone().as_str()).await;
            if c.is_ok() {
                channel_map.insert(name.clone(), c.unwrap().id);
            }
        }

        if item.content.len() > 0 && content.len() > 0 {
            let message_result = post_message(
                &config,
                channel_map.get(name.clone().as_str()).unwrap(),
                &content,
            )
            .await;

            if message_result.is_err() {
                println!("{:?}", message_result);
                break;
            }
        }

        let mark_items_as_read_result = mark_items_as_read(&config, item.id).await;
        if mark_items_as_read_result.is_err() {
            println!("{:?}", mark_items_as_read_result);
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config {
        discord_token: deserialize_string_from_env("DISCORD_TOKEN"),
        discord_server_id: deserialize_string_from_env("DISCORD_SERVER_ID"),
        selfoss_base_url: deserialize_string_from_env("SELFOSS_BASE_URL"),
        selfoss_username: deserialize_string_from_env("SELFOSS_USERNAME"),
        selfoss_password: deserialize_string_from_env("SELFOSS_PASSWORD"),
    };

    let item_list = get_tree(&config)
        .await
        .expect("Could not fetch Selfoss items");

    let channels = get_channels(&config).await;

    let channel_map = channels
        .unwrap()
        .into_iter()
        .map(|x| (x.name, x.id))
        .collect();

    send_messages(&config, item_list, channel_map).await;
}
