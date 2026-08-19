use std::collections::HashMap;

use aimer::style::ThemeMode;

use crate::api::Api;
use crate::models::categories::{Category, CategoryRoot};
use crate::models::posts::PostRoot;
use crate::models::{Root, Topic};

#[derive(Clone, Default)]
pub struct AppStore {
    pub root: Root,
    pub posts: HashMap<i64, PostRoot>,
    /// The streamed topic list for a path (e.g. `latest.json`), grown
    /// incrementally as topics arrive.
    pub streams: HashMap<String, TopicStream>,
    /// Maps a topic's `category_id` to its display name.
    pub categories: HashMap<i64, String>,
    /// How the app theme is chosen (follows the system by default).
    pub theme_mode: ThemeMode,
}

/// The streaming state of one topic-list request.
#[derive(Clone, PartialEq, Default)]
pub struct TopicStream {
    pub topics: Vec<Topic>,
    pub status: StreamStatus,
}

#[derive(Clone, PartialEq, Default)]
pub enum StreamStatus {
    #[default]
    Loading,
    Done,
    Error(String),
}

impl AppStore {
    pub async fn fetch_latest(&mut self) {
        let store = Api::get("latest.json").await.unwrap();
        self.root = store;
    }

    pub async fn fetch_categories() -> Result<CategoryRoot, String> {
        Api::get::<CategoryRoot>("categories.json")
            .await
            .map_err(|e| e.to_string())
    }

    pub fn cached_post(&self, id: i64) -> Option<PostRoot> {
        self.posts.get(&id).cloned()
    }

    pub fn cache_post(&mut self, id: i64, post: PostRoot) {
        self.posts.insert(id, post);
    }

    pub fn topic_stream(&self, path: &str) -> TopicStream {
        self.streams.get(path).cloned().unwrap_or_default()
    }

    pub fn push_stream_topic(&mut self, path: String, topic: Topic) {
        self.streams.entry(path).or_default().topics.push(topic);
    }

    pub fn finish_stream(&mut self, path: String) {
        self.streams.entry(path).or_default().status = StreamStatus::Done;
    }

    pub fn fail_stream(&mut self, path: String, error: String) {
        self.streams.entry(path).or_default().status = StreamStatus::Error(error);
    }

    pub fn category_name(&self, id: i64) -> Option<String> {
        self.categories.get(&id).cloned()
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
    }

    pub fn cache_categories(&mut self, categories: Vec<Category>) {
        self.categories = categories
            .into_iter()
            .map(|category| (category.id, category.name))
            .collect();
    }

    pub async fn stream_posts(&mut self) {
        let client = reqwest::Client::new();
        let body = match client
            .get("https://users.rust-lang.org/latest.json")
            .send()
            .await
        {
            Ok(res) => res.text().await,
            Err(_) => return,
        };

        let store = match body {
            Ok(body) => serde_json::from_str::<Root>(&body).unwrap(),
            Err(_) => return,
        };

        self.root = store;
    }
}

#[cfg(test)]
mod test_store {
    use crate::models::Root;
    use crate::store::AppStore;

    #[tokio::test]
    async fn test_fetch_latest() {
        let mut store = AppStore::default();
        store.fetch_latest().await;

        println!("{:?}", store.root);
        assert_ne!(store.root, Root::default());
    }
}
