extern crate dotenv;

use std::collections::HashMap;

use config::Config;
use dotenv::dotenv;

mod config;
mod discord;
mod selfoss;
mod utils;

use discord::{
    adapter::{get_channels, post_message},
    errors::RequestError,
};
use selfoss::{
    adapter::{get_tree, mark_items_as_read},
    models::SelfossItem,
};

use crate::{discord::adapter::create_channel, utils::deserialize_string_from_env};

async fn send_messages(
    config: &Config,
    item_list: Vec<SelfossItem>,
    mut channel_map: HashMap<String, String>,
) -> Result<(), RequestError> {
    println!("Found max {} messages to send", item_list.len());

    for item in &item_list {
        let name = item.clone().get_discord_channel_name();
        let channel = channel_map.get(name.clone().as_str());
        let content = item.clone().get_discord_message_content();

        if channel.is_none() {
            let c = create_channel(config, name.clone().as_str()).await?;
            channel_map.insert(name.clone(), c.id);
        }

        if !item.content.is_empty() && !content.is_empty() {
            post_message(
                config,
                channel_map.get(name.clone().as_str()).expect("msg"),
                &content,
            )
            .await?;
        }

        mark_items_as_read(config, item.id).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config {
        discord_base_url: String::from("https://discord.com/api/v10"),
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
        .expect("Could not get channels")
        .into_iter()
        .map(|x| (x.name, x.id))
        .collect();

    let result = send_messages(&config, item_list, channel_map).await;
    result.expect("Could not send a message");
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{config::Config, selfoss::models::SelfossItem, send_messages};
    use chrono::DateTime;
    use httpmock::{Method::POST, MockServer};

    pub fn start_server() -> (MockServer, Config) {
        let server = MockServer::start();
        let config = Config {
            discord_base_url: server.base_url(),
            discord_token: String::from("test token"),
            discord_server_id: String::from("123"),
            selfoss_base_url: server.base_url(),
            selfoss_username: String::from("test username"),
            selfoss_password: String::from("test password"),
        };
        (server, config)
    }

    pub fn get_mock_item() -> SelfossItem {
        SelfossItem {
            title: String::from("My title"),
            sourcetitle: String::from("my_channel"),
            content: String::from("My content"),
            datetime: DateTime::parse_from_rfc3339("2023-12-15T17:40:36Z")
                .unwrap()
                .into(),
            id: 187204,
        }
    }

    #[tokio::test]
    async fn test_send_messages_and_mark_read() {
        let (server, config) = start_server();

        let send_message_mock = server.mock(|when, then| {
            when.method(POST).path("/channels/my_channel_id/messages");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("src/assets/discord_send_message_mock_response.json");
        });

        let mark_item_read_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/mark/187204")
                .query_param("username", "test username")
                .query_param("password", "test password");
            then.status(200)
                .header("content-type", "application/json")
                .body("");
        });

        let channel_map =
            HashMap::from([(String::from("my_channel"), String::from("my_channel_id"))]);
        let item_list = vec![get_mock_item()];
        let result = send_messages(&config, item_list, channel_map).await;

        result.expect("Did not send messages correctly");
        send_message_mock.assert_async().await;
        mark_item_read_mock.assert_async().await;
    }
}
