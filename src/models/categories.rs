use serde::{Deserialize, Serialize};

/// Response from `categories.json`: the forum's category list.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CategoryRoot {
    pub category_list: CategoryList,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CategoryList {
    pub can_create_category: bool,
    pub can_create_topic: bool,
    pub categories: Vec<Category>,
}

/// One forum category. Discourse's `categories.json` uses snake_case keys, so
/// the field names map to the JSON as-is.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Category {
    pub id: i64,
    pub name: String,
    /// Hex color without the leading `#`, e.g. `F7941D`.
    pub color: String,
    pub text_color: String,
    pub slug: String,
    pub topic_count: i64,
    pub post_count: i64,
    pub position: i64,
    pub description: Option<String>,
    pub description_excerpt: Option<String>,
    pub topic_url: String,
    pub has_children: bool,
    pub icon: String,
    pub emoji: Option<String>,
    pub is_uncategorized: bool,
    pub read_restricted: bool,
}
