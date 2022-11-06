use chrono::{DateTime, Utc};
use scraper::Html;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelfossItemMarkedReadResponse {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelfossItem {
    pub title: String,
    pub sourcetitle: String,
    pub content: String,
    pub datetime: DateTime<Utc>,
    pub id: u64,
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

impl SelfossItem {
    pub fn get_discord_channel_name(self) -> String {
        self.sourcetitle
            .replace(' ', "-")
            .replace('.', "-")
            .to_lowercase()
    }
    pub fn get_discord_message_content(self) -> String {
        let truncated_string = truncate(&self.content, 2000).to_string();
        Html::parse_fragment(&truncated_string)
            .root_element()
            .text()
            .collect()
    }
}
