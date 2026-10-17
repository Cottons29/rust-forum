use aimer::share::{RcProjectExt, ShareRef, Shared};
use aimer::style::ThemeMode;
use std::collections::HashMap;
use std::rc::Rc;
use crate::api::Api;
use crate::models::categories::{Category, CategoryRoot};
use crate::models::comments::Comment;
use crate::models::posts::PostRoot;
use crate::models::user::UserProfile;
use crate::models::{Root, Topic};

#[derive(Clone, Default)]
pub struct AppStore {
    pub root: Root,
    pub posts: HashMap<i64, PostRoot>,
    /// Loaded user profiles, by username.
    pub profiles: HashMap<String, UserProfile>,
    /// The comments of each topic (by topic id), grown as batches arrive.
    pub threads: HashMap<i64, CommentThread>,
    /// The streamed topic list for a path (e.g. `latest.json`), grown
    /// incrementally as topics arrive.
    pub streams: HashMap<ShareRef<str>, TopicStream>,
    /// Maps a topic's `category_id` to its display name.
    pub categories: HashMap<i64, ShareRef<str>>,
    /// The full category (color, slug, ...) by id, for drawing its pill.
    pub category_details: HashMap<i64, Rc<Category>>,
    /// How the app theme is chosen (follows the system by default).
    pub theme_mode: ThemeMode,
}

/// The comments loaded so far for one topic, in post order.
#[derive(Clone, PartialEq, Default)]
pub struct CommentThread {
    pub comments: Vec<Rc<Comment>>,
    pub status: StreamStatus,
}

/// The streaming state of one topic-list request.
#[derive(Clone, PartialEq, Default)]
pub struct TopicStream {
    pub topics: Vec<Shared<Topic>>,
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

    pub fn cached_profile(&self, username: &str) -> Option<UserProfile> {
        self.profiles.get(username).cloned()
    }

    pub fn cache_profile(&mut self, username: &str, profile: UserProfile) {
        self.profiles.insert(username.to_string(), profile);
    }

    pub fn comment_thread(&self, topic_id: i64) -> CommentThread {
        self.threads.get(&topic_id).cloned().unwrap_or_default()
    }

    /// Adds `comments` to a topic's thread, skipping any already present so a
    /// repeated load cannot duplicate them.
    pub fn push_comments(&mut self, topic_id: i64, comments: Vec<Comment>) {
        let thread = self.threads.entry(topic_id).or_default();
        for comment in comments {
            if !thread
                .comments
                .iter()
                .any(|known| known.post_number == comment.post_number)
            {
                thread.comments.push(Rc::new(comment));
            }
        }
        thread.comments.sort_by_key(|comment| comment.post_number);
    }

    pub fn finish_comments(&mut self, topic_id: i64) {
        self.threads.entry(topic_id).or_default().status = StreamStatus::Done;
    }

    pub fn fail_comments(&mut self, topic_id: i64, error: String) {
        self.threads.entry(topic_id).or_default().status = StreamStatus::Error(error);
    }

    pub fn topic_stream(&self, path: &str) -> TopicStream {
        self.streams.get(path).cloned().unwrap_or_default()
    }

    /// The title of a topic that has been listed, whichever list it came from.
    ///
    /// A post page uses it to show the title before the post itself has loaded,
    /// so the page opening out of a card is recognisably that card's topic.
    pub fn topic_title(&self, id: i64) -> Option<String> {
        self.streams
            .values()
            .flat_map(|stream| stream.topics.iter())
            .find(|topic| topic.id == id)
            .map(|topic| topic.title.clone())
    }

    pub fn push_stream_topic(&mut self, path: ShareRef<str>, topic: Topic) {
        self.streams.entry(path).or_default().topics.push(Shared::new(topic));
    }

    pub fn finish_stream(&mut self, path: ShareRef<str>) {
        self.streams.entry(path).or_default().status = StreamStatus::Done;
    }

    pub fn fail_stream(&mut self, path: ShareRef<str>, error: String) {
        self.streams.entry(path).or_default().status = StreamStatus::Error(error);
    }

    pub fn category_name(&self, id: i64) -> Option<ShareRef<str>> {
        self.categories.get(&id).cloned()
    }

    pub fn category(&self, id: i64) -> Option<Rc<Category>> {
        self.category_details.get(&id).cloned()
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
    }

    pub fn cache_categories(&mut self, categories: &[Rc<Category>]) {
        self.categories = categories
            .into_iter()
            .map(|category| (category.id, category.project(|item|item.name.as_str())))
            .collect();
        self.category_details = categories
            .iter()
            .map(|category| (category.id, category.clone()))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn topic(id: i64, title: &str) -> Topic {
        Topic { id, title: title.to_owned(), ..Default::default() }
    }

    #[test]
    fn a_topic_title_is_found_in_any_stream() {
        let mut store = AppStore::default();
        store.push_stream_topic(ShareRef::from_static("latest.json"), topic(1, "First"));
        store.push_stream_topic(ShareRef::from_static("hot.json"), topic(2, "Second"));

        assert_eq!(store.topic_title(1).as_deref(), Some("First"));
        assert_eq!(store.topic_title(2).as_deref(), Some("Second"));
    }

    #[test]
    fn a_topic_that_was_never_listed_has_no_title() {
        let mut store = AppStore::default();
        assert_eq!(store.topic_title(7), None);

        store.push_stream_topic(ShareRef::from_static("latest.json"), topic(1, "First"));
        assert_eq!(store.topic_title(7), None);
    }
}
