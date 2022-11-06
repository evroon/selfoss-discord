use std::collections::HashMap;

use reqwest::header::ACCEPT;

use crate::config::Config;
use crate::selfoss::models::SelfossItem;

pub async fn get_tree(config: &Config) -> Result<Vec<SelfossItem>, reqwest::Error> {
    let endpoint = config.selfoss_base_url.clone() + "/items?type=unread&items=200";
    let client = reqwest::Client::new();
    let response = client
        .get(endpoint)
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<Vec<SelfossItem>>()
        .await;

    if response.is_err() {
        print!("Response error: {:?}", response);
    }

    response
}

pub async fn mark_items_as_read(config: &Config, item_id: u64) -> Result<String, reqwest::Error> {
    let endpoint = config.selfoss_base_url.clone() + "/mark/" + &item_id.to_string();
    let client = reqwest::Client::new();

    let mut query = HashMap::new();
    query.insert("username", config.selfoss_username.clone());
    query.insert("password", config.selfoss_password.clone());

    let response = client
        .post(endpoint)
        .header(ACCEPT, "application/json")
        .query(&query)
        .send()
        .await?
        .text()
        .await;

    if response.is_err() {
        print!("Response error: {:?}", response);
    }

    response
}
