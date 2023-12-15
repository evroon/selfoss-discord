use crate::config::Config;
use reqwest::{header::AUTHORIZATION, Method};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, fmt::Debug};

use super::errors::RequestError;
use super::middleware::RetryAfterMiddleware;
use super::models::{DiscordChannel, DiscordMessage};

async fn discord_request<D>(
    config: Config,
    method: Method,
    endpoint: &str,
    json: Option<HashMap<&str, &str>>,
) -> Result<D, RequestError>
where
    D: DeserializeOwned + Debug,
{
    let endpoint = format!("{}/{}", config.discord_base_url, endpoint);

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client: ClientWithMiddleware = ClientBuilder::new(reqwest::Client::new())
        .with(RetryAfterMiddleware::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let base_request = client.request(method, endpoint);

    let request = match json {
        Some(x) => base_request.json(&x),
        None => base_request,
    };

    request
        .header(AUTHORIZATION, format!("Bot {}", config.discord_token))
        .send()
        .await?
        .error_for_status()?
        .json::<D>()
        .await
        .map_err(RequestError::Reqwest)
}

pub async fn get_channels(config: &Config) -> Result<Vec<DiscordChannel>, RequestError> {
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
) -> Result<DiscordChannel, RequestError> {
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
) -> Result<DiscordMessage, RequestError> {
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        discord::{
            adapter::{create_channel, get_channels},
            errors::RequestError,
            models::DiscordChannel,
        },
        send_messages,
        test::{get_mock_item, start_server},
    };
    use httpmock::Method::{GET, POST};
    use reqwest::StatusCode;

    fn get_mock_channel() -> DiscordChannel {
        DiscordChannel {
            name: String::from("my_channel"),
            id: String::from("my_channel_id"),
        }
    }

    #[tokio::test]
    async fn test_create_channels() {
        let (server, config) = start_server();

        let get_discord_channels_mock = server.mock(|when, then| {
            when.method(POST).path("/guilds/123/channels");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("src/assets/discord_create_channel_mock_response.json");
        });

        let item_list = create_channel(&config, "my_channel").await;

        assert_eq!(item_list.unwrap(), get_mock_channel());
        get_discord_channels_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_channels() {
        let (server, config) = start_server();

        let get_discord_channels_mock = server.mock(|when, then| {
            when.method(GET).path("/guilds/123/channels");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("src/assets/discord_items_mock_response.json");
        });

        let item_list = get_channels(&config).await;

        assert_eq!(item_list.unwrap(), vec![get_mock_channel()]);
        get_discord_channels_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_send_messages_ratelimited() {
        let (server, config) = start_server();

        let send_message_mock = server.mock(|when, then| {
            when.method(POST).path("/channels/my_channel_id/messages");
            then.status(429)
                .header("content-type", "application/json")
                .header("Retry-After", "1")
                .body_from_file("src/assets/discord_ratelimit_mock_response.json");
        });

        let channel_map =
            HashMap::from([(String::from("my_channel"), String::from("my_channel_id"))]);
        let item_list = vec![get_mock_item()];
        let result = send_messages(&config, item_list, channel_map).await;

        let error = result.expect_err("Did somehow send messages without being ratelimited");
        match error {
            RequestError::Reqwest(x) => assert_eq!(x.status(), Some(StatusCode::TOO_MANY_REQUESTS)),
            _ => panic!("expected Reqwest error"),
        }
        // We do three retries with exponential backoff.
        send_message_mock.assert_hits(4);
    }
}
