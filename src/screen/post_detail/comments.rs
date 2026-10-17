use crate::api::Api;
use crate::components::profile_link::profile_link;
use crate::router::AppRouter;
use aimer::router::NavigatorController;
use crate::components::topic_card::relative_time;
use crate::html_markdown::forum_markdown;
use crate::models::comments::{Comment, CommentNode, build_comment_tree};
use crate::models::posts::{Post, PostStream, PostsPage};
use crate::store::{AppStore, CommentThread, StreamStatus};
use crate::theme::ForumTheme;
use aimer::input::button::Button;
use aimer::style::{
    BorderSlice, BorderStyle, BoxBorder, BoxDecoration, BoxFit, LayoutSpacing, TextOverflow, TextStyle,
    Theme,
};
use aimer::{
    AnyWidget, AsyncBuilder, BuildContext, Column, Container, Expanded, FontWeight,
    NetworkImage, ProviderContext, ProviderHandle, Row, SizedBox, State,
    StateUpdater, StatefulWidget, StatelessWidget, Text, Widget, ZeroSizedBox,
};
use std::collections::HashSet;
use std::rc::Rc;

/// How many posts one `posts.json` request asks for.
const POSTS_PER_REQUEST: usize = 20;
const AVATAR_DIAMETER: f32 = 36.0;
/// How many levels of replies indent. Deeper ones stay at the last level's
/// indent so a long chain keeps a readable width.
const MAX_INDENT_DEPTH: usize = 4;

/// The comments under a topic's first post, nested by who replied to whom.
///
/// Replies start collapsed beneath the comment they answer. The topic document
/// only carries the first batch of posts, so the rest are fetched in the
/// background and appear as they arrive.
#[derive(StatelessWidget)]
pub struct CommentsSection {
    topic_id: i64,
    /// The first batch of posts plus the ids of every post in the topic.
    initial: PostStream,
}

impl CommentsSection {
    pub fn new(topic_id: i64, initial: PostStream) -> Self {
        Self { topic_id, initial }
    }
}

impl StatelessWidget for CommentsSection {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let store = ProviderHandle::<AppStore>::of(ctx);
        let topic_id = self.topic_id;
        let initial = self.initial.clone();
        let thread = ctx.select::<AppStore, CommentThread>(move |s| s.comment_thread(topic_id));

        let driver = AsyncBuilder::new()
            .request_key(topic_id)
            .key(format!("comments-{topic_id}"))
            .future(move || {
                let store = store.clone();
                let initial = initial.clone();
                async move { load_comments(store, topic_id, initial).await }
            })
            .child(|_| ZeroSizedBox.boxed());

        let tree = build_comment_tree(&thread.comments);
        let mut sections: Vec<AnyWidget> = vec![
            driver.boxed(),
            section_text(heading(thread.comments.len()), &theme, 20, FontWeight::Bold),
        ];
        sections.push(
            Column::new()
                .gaps(LayoutSpacing::all(6))
                .list(tree)
                .builder(|node| CommentThreadView::new(node.clone(), 0))
                .boxed(),
        );
        match &thread.status {
            StreamStatus::Loading => {
                sections.push(section_text("Loading comments…".into(), &theme, 14, FontWeight::Normal))
            }
            StreamStatus::Error(error) => sections.push(section_text(
                format!("Couldn't load more comments: {error}"),
                &theme,
                14,
                FontWeight::Normal,
            )),
            StreamStatus::Done => {}
        }

        Container::new()
            .padding(LayoutSpacing::all(24).top(8))
            .child(Column::new().gaps(LayoutSpacing::all(12)).children(sections))
    }
}

fn heading(loaded: usize) -> String {
    match loaded {
        0 => "Comments".to_string(),
        1 => "1 comment".to_string(),
        n => format!("{n} comments"),
    }
}

fn section_text(text: String, theme: &ForumTheme, size: u32, weight: FontWeight) -> AnyWidget {
    Text::new(text)
        .text_style(
            TextStyle::new()
                .font_size(size)
                .font_weight(weight)
                .color(theme.on_background_color().with_opacity(if size > 16 { 255 } else { 160 })),
        )
        .boxed()
}

/// Streams a topic's comments into the store: the first batch comes straight
/// from the topic document, the remainder from `posts.json` in chunks.
async fn load_comments(
    store: ProviderHandle<AppStore>,
    topic_id: i64,
    initial: PostStream,
) -> Result<(), String> {
    if store.read().comment_thread(topic_id).status == StreamStatus::Done {
        return Ok(());
    }

    let loaded: HashSet<i64> = initial.posts.iter().map(|post| post.id).collect();
    store.update(|s| s.push_comments(topic_id, comments_from(&initial.posts)));

    let missing: Vec<i64> = initial
        .stream
        .iter()
        .copied()
        .filter(|id| !loaded.contains(id))
        .collect();
    for chunk in missing.chunks(POSTS_PER_REQUEST) {
        let query = chunk
            .iter()
            .map(|id| format!("post_ids[]={id}"))
            .collect::<Vec<_>>()
            .join("&");
        match Api::get::<PostsPage>(&format!("t/{topic_id}/posts.json?{query}")).await {
            Ok(page) => {
                let comments = comments_from(&page.post_stream.posts);
                store.update(|s| s.push_comments(topic_id, comments));
            }
            Err(error) => {
                store.update(|s| s.fail_comments(topic_id, error.to_string()));
                return Ok(());
            }
        }
    }

    store.update(|s| s.finish_comments(topic_id));
    Ok(())
}

/// The replies among `posts`: everything but the topic's first post, whispers
/// and small system actions ("closed this topic"), which carry no text.
fn comments_from(posts: &[Post]) -> Vec<Comment> {
    const REGULAR: i64 = 1;
    const MODERATOR_ACTION: i64 = 2;
    posts
        .iter()
        .filter(|post| post.post_number > 1)
        .filter(|post| matches!(post.post_type, REGULAR | MODERATOR_ACTION))
        .map(Comment::from_post)
        .collect()
}

/// A comment and, beneath it, a toggle for the replies to it.
#[derive(StatefulWidget)]
struct CommentThreadView {
    node: Rc<CommentNode>,
    depth: usize,
}

impl CommentThreadView {
    fn new(node: Rc<CommentNode>, depth: usize) -> Self {
        Self { node, depth }
    }
}

struct CommentThreadViewState {
    node: Rc<CommentNode>,
    depth: usize,
    /// Whether the replies beneath this comment are showing. Kept across
    /// rebuilds, which happen whenever another batch of comments arrives.
    expanded: bool,
    updater: StateUpdater<Self>,
}

impl StatefulWidget for CommentThreadView {
    type State = CommentThreadViewState;

    fn create_state(self) -> Self::State {
        CommentThreadViewState {
            node: self.node,
            depth: self.depth,
            expanded: false,
            updater: StateUpdater::new(),
        }
    }
}

impl State<CommentThreadView> for CommentThreadViewState {
    fn init_state(&mut self, updater: StateUpdater<Self>) {
        self.updater = updater;
    }

    fn adopt_config_from(&mut self, new: Self) {
        self.node = new.node;
        self.depth = new.depth;
    }

    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let mut parts: Vec<AnyWidget> = vec![CommentCard::new(self.node.comment.clone()).boxed()];

        let reply_count = self.node.descendant_count();
        if reply_count > 0 {
            let updater = self.updater.clone();
            let label = match (self.expanded, reply_count) {
                (true, _) => "Hide replies".to_string(),
                (false, 1) => "Show 1 reply".to_string(),
                (false, n) => format!("Show {n} replies"),
            };
            parts.push(
                Container::new()
                    .padding(LayoutSpacing::new().left(AVATAR_DIAMETER as u32 + 12))
                    .child(
                        Button::new()
                            .decoration(BoxDecoration::new().border_radius(6))
                            .hover_decoration(
                                BoxDecoration::new()
                                    .border_radius(6)
                                    .background_color(theme.primary_color().with_alpha(0.12)),
                            )
                            .on_press(move || updater.set_state(|s| s.expanded = !s.expanded))
                            .child(
                                Container::new()
                                    .padding(LayoutSpacing::all(6).left(8).right(8))
                                    .child(Text::new(label).text_style(
                                        TextStyle::new()
                                            .font_size(14)
                                            .font_weight(FontWeight::Bold)
                                            .color(theme.primary_color()),
                                    )),
                            ),
                    )
                    .boxed(),
            );

            if self.expanded {
                let depth = self.depth + 1;
                // Replies hang from the middle of their parent's avatar.
                let step = if self.depth < MAX_INDENT_DEPTH {
                    AVATAR_DIAMETER as u32 / 2
                } else {
                    0
                };
                let guide = BoxBorder {
                    left: BorderSlice::new()
                        .style(BorderStyle::Solid)
                        .stroke(2)
                        .color(theme.on_background_color().with_opacity(40)),
                    ..Default::default()
                };
                parts.push(
                    Container::new()
                        .margin(LayoutSpacing::new().left(step))
                        .padding(LayoutSpacing::new().left(step))
                        .box_decoration(BoxDecoration::new().border(guide))
                        .child(
                            Column::new()
                                .gaps(LayoutSpacing::all(6))
                                .list(self.node.replies.clone())
                                .builder(move |reply| CommentThreadView::new(reply.clone(), depth)),
                        )
                        .boxed(),
                );
            }
        }

        Column::new().gaps(LayoutSpacing::all(4)).children(parts)
    }
}

/// One comment: avatar, author and age, and the body.
#[derive(StatelessWidget)]
struct CommentCard {
    comment: Rc<Comment>,
}

impl CommentCard {
    fn new(comment: Rc<Comment>) -> Self {
        Self { comment }
    }
}

impl StatelessWidget for CommentCard {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let comment = &self.comment;
        let navigator = NavigatorController::<AppRouter>::of(ctx);

        let avatar = Container::new()
            .width(AVATAR_DIAMETER)
            .height(AVATAR_DIAMETER)
            .box_decoration(BoxDecoration::new().border_radius(AVATAR_DIAMETER / 2.0))
            .child(
                NetworkImage::new(comment.avatar_url.clone())
                    .width(AVATAR_DIAMETER)
                    .height(AVATAR_DIAMETER)
                    .fit(BoxFit::Cover)
                    .loading_widget(
                        SizedBox::new()
                            .color(theme.primary_color().with_alpha(0.2)),
                    ),
            );

        let header = Row::new().gaps(LayoutSpacing::all(8)).children([
            profile_link(
                navigator.clone(),
                &comment.username,
                Text::new(comment.username.clone()).text_style(
                    TextStyle::new()
                        .font_size(15)
                        .font_weight(FontWeight::Bold)
                        .color(theme.on_background_color()),
                ),
            ),
            Text::new(relative_time(&comment.created_at))
                .text_style(
                    TextStyle::new()
                        .font_size(13)
                        .text_overflow(TextOverflow::Wrap)
                        .color(theme.on_background_color().with_opacity(150)),
                )
                .boxed(),
        ]);

        let body = forum_markdown(&comment.markdown, &theme, LayoutSpacing::new().top(2));

        Row::new().gaps(LayoutSpacing::all(12)).children([
            profile_link(navigator, &comment.username, avatar),
            Expanded::new()
                .box_child(Column::new().children([header.boxed(), body])),
        ])
    }
}
