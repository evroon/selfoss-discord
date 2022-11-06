use crate::config::Config;
use reqwest::{header::AUTHORIZATION, Method};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, fmt::Debug};

use super::models::{DiscordChannel, DiscordMessage};

async fn discord_request<D>(
    config: Config,
    method: Method,
    endpoint: &str,
    json: Option<HashMap<&str, &str>>,
) -> Result<D, reqwest::Error>
where
    D: DeserializeOwned + Debug,
{
    let endpoint = format!("https://discord.com/api/v10/{}", endpoint);
    let client = reqwest::Client::new();
    let base_request = client.request(method, endpoint);

    let request = match json {
        Some(x) => base_request.json(&x),
        None => base_request,
    };

    let response = request
        .header(AUTHORIZATION, format!("Bot {}", config.discord_token))
        .send()
        .await?
        .text()
        .await;

    match response {
        Err(x) => {
            print!("Response error: {:?}\n", x);
            Err(x)
        }
        Ok(x) => match serde_json::from_str::<D>(&x) {
            Err(y) => panic!("Response: {}\n\n\nCould not deserialize: {}", x, y),
            Ok(y) => Ok(y),
        },
    }
}

pub async fn get_channels(config: &Config) -> Result<Vec<DiscordChannel>, reqwest::Error> {
    discord_request::<Vec<DiscordChannel>>(
        config.clone(),
        Method::GET,
        format!("guilds/{}/channels", config.discord_server_id).as_str(),
        None,
    )
    .await
}

pub async fn create_channel(
    config: &Config,
    channel_name: &str,
) -> Result<DiscordChannel, reqwest::Error> {
    let mut payload = HashMap::new();
    payload.insert("name", channel_name);
    discord_request::<DiscordChannel>(
        config.clone(),
        Method::POST,
        format!("guilds/{}/channels", config.discord_server_id).as_str(),
        Some(payload),
    )
    .await
}

pub async fn post_message(
    config: &Config,
    channel_id: &str,
    content: &str,
) -> Result<DiscordMessage, reqwest::Error> {
    let mut payload = HashMap::new();
    payload.insert("content", content);
    discord_request::<DiscordMessage>(
        config.clone(),
        Method::POST,
        format!("channels/{}/messages", channel_id).as_str(),
        Some(payload),
    )
    .await
}
