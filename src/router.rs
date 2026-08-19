use std::time::Duration;

use crate::app_shell::AppShell;
use crate::screen::post_detail::PostDetailScreen;
use crate::screen::settings::SettingsScreen;
use crate::screen::topic_list::TopicListScreen;
use aimer::animation::{AnimatedSwitcher, Curve};
use aimer::router::{Router, Shell};
use aimer::{AnyWidget, BuildContext, Router, Widget};

const ROUTE_TRANSITION_DURATION: Duration = Duration::from_millis(200);
const ROUTE_SWITCHER_KEY: &str = "route-switcher";

#[derive(Clone, Router, PartialEq)]
pub enum AppRouter {
    #[route("/")]
    Latest,
    #[route("/new")]
    New,
    #[route("/hot")]
    Hot,
    #[route("/c/{slug}/{id}")]
    CategoryTopics { slug: String, id: i64 },
    #[route("/t/{slug}/{id}")]
    PostDetail { slug: String, id: i64 },
    #[route("/settings")]
    Settings,
}

fn transitioned_page(key: &'static str, child: AnyWidget) -> AnyWidget {
    AnimatedSwitcher::new(ROUTE_TRANSITION_DURATION, Curve::FastOutSlowIn, child)
        .child_key(key)
        .key(ROUTE_SWITCHER_KEY)
        .boxed()
}

/// A shell whose outlet renders a [`TopicListScreen`] for a fixed Discourse
/// list/category path, wrapped in a route transition keyed by `key`.
fn topic_list_page(
    key: &'static str,
    path: String,
    route: AppRouter,
    title: impl Into<String>,
) -> AnyWidget {
    let title = title.into();
    Shell::boxing(AppShell::new(route), move |_| {
        transitioned_page(
            key,
            TopicListScreen::new(path.clone(), title.clone()).boxed(),
        )
    })
}

impl Router for AppRouter {
    fn build(&self, _ctx: &BuildContext) -> AnyWidget {
        let current_route = self.clone();
        match self {
            AppRouter::Latest => {
                topic_list_page("latest", "latest.json".to_string(), current_route, "Latest")
            }
            AppRouter::New => topic_list_page("new", "new.json".to_string(), current_route, "New"),
            AppRouter::Hot => topic_list_page("hot", "hot.json".to_string(), current_route, "Hot"),
            AppRouter::CategoryTopics { slug, id } => {
                let path = format!("c/{slug}/{}.json", *id);
                topic_list_page("category", path, current_route, slug)
            }
            AppRouter::PostDetail { slug, id } => {
                let slug = slug.clone();
                let id = *id;
                Shell::boxing(AppShell::new(current_route), move |_| {
                    transitioned_page("post", PostDetailScreen::new(slug.clone(), id).boxed())
                })
            }
            AppRouter::Settings => {
                Shell::boxing(AppShell::new(current_route), move |_| {
                    transitioned_page("settings", SettingsScreen.boxed())
                })
            }
        }
    }
}
