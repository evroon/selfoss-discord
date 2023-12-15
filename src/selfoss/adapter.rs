use std::collections::HashMap;

use reqwest::header::ACCEPT;

use crate::config::Config;
use crate::selfoss::models::SelfossItem;

pub async fn get_tree(config: &Config) -> Result<Vec<SelfossItem>, reqwest::Error> {
    reqwest::Client::new()
        .get(config.selfoss_base_url.clone() + "/items")
        .query(&[("type", "unread"), ("items", "200")])
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<Vec<SelfossItem>>()
        .await
}

pub async fn mark_items_as_read(config: &Config, item_id: u64) -> Result<String, reqwest::Error> {
    let endpoint = config.selfoss_base_url.clone() + "/mark/" + &item_id.to_string();
    let client = reqwest::Client::new();

    let mut query = HashMap::new();
    query.insert("username", config.selfoss_username.clone());
    query.insert("password", config.selfoss_password.clone());

    client
        .post(endpoint)
        .header(ACCEPT, "application/json")
        .query(&query)
        .send()
        .await?
        .text()
        .await
}

#[cfg(test)]
mod test {
    use crate::{
        selfoss::adapter::get_tree,
        test::{get_mock_item, start_server},
    };
    use httpmock::Method::GET;

    #[tokio::test]
    async fn test_get_tree() {
        let (server, config) = start_server();

        let get_selfoss_items_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/items")
                .query_param("type", "unread")
                .query_param("items", "200");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("src/assets/selfoss_mock_response.json");
        });

        let item_list = get_tree(&config).await;
        assert_eq!(item_list.unwrap(), vec![get_mock_item(); 1]);

        get_selfoss_items_mock.assert_async().await;
    }
}
