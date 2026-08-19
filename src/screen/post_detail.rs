use crate::api::Api;
use crate::html_markdown::html_to_markdown;
use crate::html_markdown::light_box::LightBox;
use crate::html_markdown::link_card::LinkCard;
use crate::models::posts::PostRoot;
use crate::router::AppRouter;
use crate::store::AppStore;
use crate::theme::ForumTheme;
use aimer::console::error;
use aimer::router::NavigatorController;
use aimer::style::{BorderRadius, BoxDecoration, LayoutSpacing, TextAlign, TextOverflow, TextStyle, Theme};
use aimer::{
    Align, Alignment, AsyncBuilder, AsyncSnapshot, BuildContext, Button, Column, Container,
    Expanded, FontWeight, MarkdownBlockRule, MarkdownBlockSyntax, MarkdownTheme, MarkdownViewer,
    ProviderHandle, Row, Scrollable, SizedBox, Stack, StatelessWidget, Text, Widget,
};
use std::rc::Rc;

#[derive(Clone, StatelessWidget)]
pub struct PostDetailScreen {
    slug: String,
    id: i64,
}

impl PostDetailScreen {
    pub fn new(slug: String, id: i64) -> Self {
        Self { slug, id }
    }
}

impl StatelessWidget for PostDetailScreen {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let store = ProviderHandle::<AppStore>::of(ctx);
        let navigator = NavigatorController::<AppRouter>::of(ctx);
        let slug = self.slug.clone();
        let id = self.id;

        let data = AsyncBuilder::new()
            .request_key(id)
            .key(format!("post-{id}"))
            .future(move || {
                let store = store.clone();
                let slug = slug.clone();
                async move { fetch_cached_post(store, slug, id).await }
            })
            .child(move |snapshot| match snapshot {
                AsyncSnapshot::Waiting => Text::new("Loading")
                    .text_style(TextStyle::new().color(theme.on_background_color()))
                    .boxed(),
                AsyncSnapshot::Error(error) => {
                    error!("Error: {}", error);
                    Text::new(error.clone())
                        .text_style(TextStyle::new().color(theme.on_background_color()))
                        .boxed()
                }
                AsyncSnapshot::Data(post) => {
                    let markdown = post
                        .post_stream
                        .posts
                        .first()
                        .map(|p| html_to_markdown(&p.cooked))
                        .unwrap_or_default();

                    PostContent {
                        title: post.title.clone(),
                        markdown,
                    }
                    .boxed()
                }
            });

        Container::new().color(theme.background_color()).child(
            Stack::new().children([
                Align::new()
                    .layer(3)
                    .alignment(Alignment::TopLeft)
                    .box_child(
                        Container::new()
                            .height(60)
                            .width(100)
                            .padding(LayoutSpacing::all(10))
                            .child(
                                Button::new()
                                    .decoration(theme.box_decor().border_radius(50))
                                    .hover_decoration(theme.hover_box_decor().border_radius(50).background_color(theme.background_color()))
                                    .on_press(move || {
                                        if navigator.can_pop() {
                                            navigator.pop();
                                        } else {
                                            navigator.push(AppRouter::Latest);
                                        }
                                    })
                                    .child(
                                        Text::new("< Back")
                                            .text_align(TextAlign::MidCenter)
                                            .text_style(
                                                TextStyle::new()
                                                    .font_size(16)
                                                    .color(theme.on_background_color()),
                                            ),
                                    ),
                            ),
                    ),
                Scrollable::new().box_child(
                    Column::new().children([SizedBox::new().height(66).boxed(), data.boxed()]),
                ),
            ]),
        )
    }
}

async fn fetch_cached_post(
    store: ProviderHandle<AppStore>,
    slug: String,
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
            Container::new()
                .padding(LayoutSpacing::new().left(12))
                .box_child(
                    Text::new(self.title.clone()).text_style(
                        TextStyle::new()
                            .text_overflow(TextOverflow::Wrap)
                            .font_weight(FontWeight::Bold)
                            .font_size(28)
                            .color(theme.on_background_color()),
                    ),
                ),
            MarkdownViewer::new()
                .typed_block::<LinkCard>()
                .typed_block::<LightBox>()
                .padding(LayoutSpacing::all(16))
                .markdown(self.markdown.clone())
                .theme(themed_markdown(&theme))
                .scrollable(false)
                .boxed(),
        ])
    }
}

fn themed_markdown(theme: &ForumTheme) -> MarkdownTheme {
    let mut markdown = MarkdownTheme::default();
    markdown.body = markdown.body.color(theme.font_color);
    markdown.headings = markdown.headings.map(|style| style.color(theme.font_color));
    markdown.blockquote = markdown
        .blockquote
        .color(theme.on_background_color().with_opacity(180));
    markdown.code_block = markdown.code_block.color(theme.on_surface_color());
    markdown.inline_code = markdown
        .inline_code
        .color(theme.on_surface_color())
        .background_color(theme.surface_color().darken(0.1).with_alpha(0.4));
    markdown.link = markdown.link.color(theme.primary_color());
    markdown.link_hover_color = theme.primary_color().lighten(0.2);
    markdown.code_background = theme.surface_color().darken(0.1).with_alpha(0.4);
    markdown.quote_background = theme.surface_color().darken(0.1).with_alpha(0.4);
    markdown.rule_color = theme.on_background_color().with_opacity(72);
    markdown.table_header_background = theme.surface_color();
    markdown.table_cell_background = theme.background_color();
    markdown
}
