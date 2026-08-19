use std::time::Duration;

use crate::components::skeleton_topic_card::SkeletonTopicCard;
use crate::components::topic_card::TopicCard;
use crate::models::Topic;
use crate::store::{AppStore, StreamStatus, TopicStream};
use crate::theme::ForumTheme;
use crate::topic_stream::stream_topics;
use aimer::animation::{AnimatedSwitcher, Curve};
use aimer::style::{LayoutSpacing, TextStyle, Theme};
use aimer::{
    AnyWidget, AsyncBuilder, BuildContext, Column, Container, Expanded, FontWeight,
    ProviderContext, ProviderHandle, Scrollable, SizedBox, StatelessWidget, Text, Widget,
    ZeroSizedBox,
};
use crate::components::page_title::PageTitle;

const TRANSITION_DURATION: Duration = Duration::from_millis(250);

/// A scrollable list of topics streamed from a Discourse API path such as
/// `latest.json` (home) or `c/{slug}/{id}.json` (a category). Topics render
/// one-by-one as they arrive instead of waiting for the whole document.
#[derive(StatelessWidget, Clone)]
pub struct TopicListScreen {
    path: String,
    title: String,
}

impl TopicListScreen {
    pub fn new(path: String, title: String) -> Self {
        Self { path, title }
    }
}

impl StatelessWidget for TopicListScreen {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let store = ProviderHandle::<AppStore>::of(ctx);
        let path = self.path.clone();
        let stream = ctx.select::<AppStore, TopicStream>({
            let path = path.clone();
            move |s| s.topic_stream(&path)
        });

        let phase = if matches!(&stream.status, StreamStatus::Loading) && stream.topics.is_empty() {
            "loading"
        } else {
            "loaded"
        };
        let content = AnimatedSwitcher::new(
            TRANSITION_DURATION,
            Curve::EaseInOut,
            Column::new().children([
                PageTitle::new(self.title.clone()).boxed(),
                SizedBox::new().height(16).boxed(),
                stream_content(stream, theme),
            ]),
        )
        .child_key(phase)
        .key(format!("topic-stream-switcher-{path}"))
        .boxed();

        let driver = AsyncBuilder::new()
            .request_key(path.clone())
            .key(format!("topics-stream-{path}"))
            .future(move || {
                let store = store.clone();
                let path = path.clone();
                async move { stream_topics_task(store, path).await }
            })
            .child(|_| ZeroSizedBox.boxed());

        // Text::new(self.title.clone())
        //     .text_style(TextStyle::new().font_size(32).font_weight(FontWeight::Bold))
        //     .boxed(),

        Container::new().color(theme.background_color()).child(
            Column::new().children([
                driver.boxed(),
                Expanded::new()
                    .child(
                        Scrollable::new()
                            .key(format!("scrollable-{}", self.path))
                            .child(content),
                    )
                    .boxed(),
            ]),
        )
    }
}

async fn stream_topics_task(store: ProviderHandle<AppStore>, path: String) -> Result<(), String> {
    // A path that finished streaming before is already in the store; don't
    // re-fetch it.
    if store.read().topic_stream(&path).status == StreamStatus::Done {
        return Ok(());
    }

    let store_for_stream = store.clone();
    let path_for_push = path.clone();
    let result = stream_topics(&path, move |topic| {
        let path = path_for_push.clone();
        store_for_stream.update(move |s| s.push_stream_topic(path, topic));
    })
    .await;

    match result {
        Ok(()) => store.update(|s| s.finish_stream(path.clone())),
        Err(error) => store.update(|s| s.fail_stream(path.clone(), error)),
    }

    Ok(())
}

fn stream_content(stream: TopicStream, theme: ForumTheme) -> AnyWidget {
    match &stream.status {
        StreamStatus::Loading if stream.topics.is_empty() => skeleton_list(),
        StreamStatus::Error(error) if stream.topics.is_empty() => Text::new(error.clone())
            .text_style(TextStyle::new().color(theme.on_background_color()))
            .boxed(),
        _ => topics_list(stream.topics),
    }
}

fn skeleton_list() -> AnyWidget {
    Column::new()
        .list((0..6).collect::<Vec<usize>>())
        .builder(|_| {
            Container::new()
                .margin(LayoutSpacing::all(12))
                .child(SkeletonTopicCard::new())
        })
        .boxed()
}

fn topics_list(topics: Vec<Topic>) -> AnyWidget {
    Column::new()
        .list(topics)
        .builder(|topic| {
            Container::new()
                .margin(LayoutSpacing::all(12))
                .child(TopicCard::new(topic.clone()))
        })
        .boxed()
}
