mod comments;

use crate::app_shell::is_mobile;
use crate::api::Api;
use crate::components::post_action_bar::PostActionBar;
use crate::html_markdown::{forum_markdown, html_to_markdown};
use crate::models::posts::PostRoot;
use crate::screen::post_detail::comments::CommentsSection;
use crate::store::AppStore;
use crate::theme::ForumTheme;
use aimer::animation::{AnimatedBuilder, AnimationController, curves};
use aimer::console::error;
use aimer::decoration::{BackdropFilter, Decoration, DecorationBox};
use aimer::share::ShareRef;
use aimer::style::{BoxDecoration, LayoutSpacing, TextOverflow, TextStyle, Theme};
use aimer::{
    AnyWidget, AsyncBuilder, AsyncSnapshot, BuildContext, Color, Column, Container, Dimension, FontWeight, Positioned, ProviderHandle, ScrollController,
    Scrollable, SizedBox, Stack, State, StateUpdater, StatefulWidget, StatelessWidget, Text, Vec2d,
    Widget, ZeroSizedBox, case,
};
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, StatefulWidget)]
pub struct PostDetailScreen {
    slug: ShareRef<str>,
    id: i64,
}

impl PostDetailScreen {
    pub fn new(slug: ShareRef<str>, id: i64) -> Self {
        Self { slug, id }
    }
}

pub struct PostDetailScreenState {
    parent: PostDetailScreen,
    scroll_controller: ScrollController,
    is_scrolling: bool,
    like_count: Option<i64>,
    comment_count: Option<i64>,
    updater: StateUpdater<Self>,
}

impl StatefulWidget for PostDetailScreen {
    type State = PostDetailScreenState;

    fn create_state(self) -> Self::State {
        PostDetailScreenState {
            parent: self,
            scroll_controller: ScrollController::new(),
            is_scrolling: false,
            like_count: None,
            comment_count: None,
            updater: StateUpdater::new(),
        }
    }
}

impl State<PostDetailScreen> for PostDetailScreenState {
    fn init_state(&mut self, updater: StateUpdater<Self>) {
        // self.scroll_controller.on_scroll(move |offset: Vec2d| {
        //     let current_scroll = updater.read_state().is_scrolling;
        //     let is_scrolled = offset.y > 50.0;
        //     if is_scrolled != current_scroll {
        //         updater.set_state(move |s| s.is_scrolling = is_scrolled)
        //     }
        // });

        self.updater = updater;
    }

    fn adopt_config_from(&mut self, new: Self) {
        let id_changed = self.parent.id != new.parent.id;
        self.parent = new.parent;

        if id_changed {
            self.is_scrolling = false;
            self.like_count = None;
            self.comment_count = None;
        }
    }

    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let store = ProviderHandle::<AppStore>::of(ctx);
        // let navigator = NavigatorController::<AppRouter>::of(ctx);
        let slug = self.parent.slug.clone();
        let id = self.parent.id;
        let known_like_count = self.like_count;
        let known_comment_count = self.comment_count;
        // The card this page opened from already has the title: show it from the
        // first frame instead of bars, so the card visibly becomes the post.
        let known_title = store.read().topic_title(id);
        let updater = self.updater;
        let scroll_ctrler = updater.read_state().scroll_controller.clone();
        let is_scrolled = updater.read_state().is_scrolling;

        let data = AsyncBuilder::new()
            .request_key(id)
            .key(format!("post-{id}"))
            .future(move || {
                let store = store.clone();
                let slug = slug.clone();
                async move { fetch_cached_post(store, slug, id).await }
            })
            .child(move |snapshot| match snapshot {
                AsyncSnapshot::Waiting => SkeletonPostContent::new(known_title.clone()).boxed(),
                AsyncSnapshot::Error(error) => {
                    error!("Error: {}", error);
                    Text::new(error.clone())
                        .text_style(TextStyle::new().color(theme.on_background_color()))
                        .boxed()
                }
                AsyncSnapshot::Data(post) => {
                    if known_like_count != Some(post.like_count)
                        || known_comment_count != Some(post.reply_count)
                    {
                        let counts_updater = updater.clone();
                        let like_count = post.like_count;
                        let comment_count = post.reply_count;
                        counts_updater.set_state(move |state| {
                            state.like_count = Some(like_count);
                            state.comment_count = Some(comment_count);
                        });
                    }

                    let markdown = post
                        .post_stream
                        .posts
                        .first()
                        .map(|p| html_to_markdown(&p.cooked))
                        .unwrap_or_default();

                    Column::new()
                        .children([
                            PostContent {
                                title: post.title.clone(),
                                markdown,
                            }
                            .boxed(),
                            CommentsSection::new(id, post.post_stream.clone()).boxed(),
                        ])
                        .boxed()
                }
            });

        let actions = match (self.like_count, self.comment_count) {
            (Some(like_count), Some(comment_count)) => ActionsBar {
                is_scrolled: false,
                like_count,
                comment_count,
                id,
            }
            .boxed(),
            _ => ActionsBar {
                is_scrolled: false,
                like_count: 0,
                comment_count: 0,
                id,
            }
            .boxed(),
        };

        Container::new().color(theme.background_color()).child(
            Stack::new().children([
                Positioned::new().layer(3).top(case!(is_mobile(ctx), 45)).box_child(actions),
                Scrollable::new()
                    .controller(scroll_ctrler)
                    .box_child(Column::new().children([
                        SizedBox::new().height(case!(is_mobile(ctx), 45) + 86).boxed(),
                        data.boxed(),
                        SizedBox::new().height(90).boxed(),
                    ])),
            ]),
        )
    }
}

/// The bar of like and comment counts at the top of a post.
///
/// It is drawn as it is. A fade on hover used to live here, but the glass panel
/// ignores the opacity that animation produced, so crossing the bar started a
/// couple of dozen frames of rebuilding, reconciling and re-recording the page
/// for a picture that did not change — a burst of full CPU each time the pointer
/// went over it, and again when it left. When glass can take an opacity, wrap the
/// panel in an `ImplicitAnimatedBuilder` driven by a hover state again.
#[derive(StatelessWidget)]
struct ActionsBar {
    is_scrolled: bool,
    like_count: i64,
    comment_count: i64,
    id: i64,
}

impl StatelessWidget for ActionsBar {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);

        let decoration = case!(
            self.is_scrolled,
            BackdropFilter::new()
                .blur_radius(case!(self.is_scrolled, 8.0))
                .saturation(0.65)
                .brightness(1.0)
                .contrast(0.9)
                .notype(),
            Color::Transparent.notype()
        );
        DecorationBox::new().decoration(decoration).child(
            SizedBox::new()
                .color(case!(
                    self.is_scrolled,
                    theme.surface_color().with_alpha(0.3),
                    Color::Transparent
                ))
                .child(PostActionBar::new(
                    self.like_count,
                    self.comment_count,
                    self.id,
                )),
        )
    }
}

async fn fetch_cached_post(
    store: ProviderHandle<AppStore>,
    slug: ShareRef<str>,
    id: i64,
) -> Result<PostRoot, String> {
    if let Some(post) = store.read().cached_post(id) {
        return Ok(post);
    }

    let post = Api::get::<PostRoot>(&format!("t/{slug}/{id}.json"))
        .await
        .map_err(|e| format!("Error: {}", e))?;

    store.update(|s| s.cache_post(id, post.clone()));
    Ok(post)
}

/// Renders the post title and markdown. Reads the theme in its own `build` so
/// it re-renders when the theme changes, independent of the `AsyncBuilder`'s
/// snapshot (whose child closure only re-runs when the data changes).
#[derive(Clone, StatelessWidget)]
struct PostContent {
    title: String,
    markdown: Rc<str>,
}

impl StatelessWidget for PostContent {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);

        Column::new().children([
            post_title(theme, self.title.clone()),
            forum_markdown(&self.markdown, &theme, LayoutSpacing::all(24)),
        ])
    }
}

/// The title of a post, as the page shows it both while the post loads and once
/// it has, so the two never differ by a pixel.
fn post_title(theme: ForumTheme, title: String) -> AnyWidget {
    Container::new()
        .padding(LayoutSpacing::horizontal(24.into()))
        .box_child(
            Text::new(title).text_style(
                TextStyle::new()
                    .text_overflow(TextOverflow::Wrap)
                    .font_weight(FontWeight::Bold)
                    .font_size(28)
                    .color(theme.on_background_color()),
            ),
        )
}

/// Pulsing placeholders shown while the post request is loading.
#[derive(StatelessWidget)]
struct SkeletonPostContent {
    controller: AnimationController,
    /// The title, when it is already known; bars stand in for it otherwise.
    title: Option<String>,
}

impl SkeletonPostContent {
    fn new(title: Option<String>) -> Self {
        let controller = AnimationController::new(Duration::from_millis(1000), curves::EASE_IN_OUT);
        controller.set_repeat(true);
        controller.set_auto_reverse(true);
        controller.forward_from_first_tick();
        Self { controller, title }
    }
}

impl StatelessWidget for SkeletonPostContent {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let controller = self.controller.clone();
        let title = self.title.clone();
        AnimatedBuilder::new(controller, move |value| {
            let tone = theme.primary_color().with_brightness(1.1);
            let bar = tone.with_alpha(0.12).lerp(tone.with_alpha(0.3), value);

            Column::new().children([
                match title.clone() {
                    Some(title) => post_title(theme, title),
                    None => Container::new()
                        .padding(LayoutSpacing::horizontal(24.into()))
                        .child(
                            Column::new().gaps(LayoutSpacing::all(10)).children([
                                skeleton_bar(bar, 28.0, 75.0),
                                skeleton_bar(bar, 28.0, 45.0),
                            ]),
                        )
                        .boxed(),
                },
                Container::new().padding(LayoutSpacing::all(24)).child(
                    Column::new().gaps(LayoutSpacing::all(14)).children([
                        skeleton_bar(bar, 16.0, 100.0),
                        skeleton_bar(bar, 16.0, 96.0),
                        skeleton_bar(bar, 16.0, 87.0),
                        skeleton_bar(bar, 16.0, 100.0),
                        skeleton_bar(bar, 16.0, 66.0),
                        SizedBox::new().height(14).boxed(),
                        skeleton_bar(bar, 16.0, 92.0),
                        skeleton_bar(bar, 16.0, 100.0),
                        skeleton_bar(bar, 16.0, 72.0),
                    ]),
                )
                .boxed(),
            ])
        })
    }
}

fn skeleton_bar(color: Color, height: f32, width_percent: f32) -> AnyWidget {
    Container::new()
        .height(Dimension::Px(height))
        .width(Dimension::Percent(width_percent))
        .box_decoration(
            BoxDecoration::new()
                .background_color(color)
                .border_radius(6),
        )
        .child(ZeroSizedBox)
        .boxed()
}
