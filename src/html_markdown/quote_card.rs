use crate::html_markdown::forum_markdown;
use crate::models::comments::avatar_url;
use crate::theme::ForumTheme;
use aimer::style::{
    BorderSlice, BorderStyle, BoxBorder, BoxDecoration, BoxFit, LayoutSpacing, TextStyle, Theme,
};
use aimer::{
    AnyWidget, BuildContext, Column, Container, FontWeight, MarkdownCustomBlock,
    MarkdownCustomBlockInput, MarkdownError, NetworkImage, Row, SizedBox, StatelessWidget, Text,
    Widget,
};
use serde::{Deserialize, Serialize};

const AVATAR_DIAMETER: f32 = 24.0;

/// A Discourse quote: who is being quoted, and what they said.
///
/// The quoted text is Markdown and may itself hold quotes, lightboxes or link
/// cards, so it is rendered with the full forum Markdown viewer.
#[derive(Serialize, Deserialize, StatelessWidget)]
pub struct QuoteCard {
    username: String,
    avatar_url: String,
    markdown: String,
}

impl QuoteCard {
    pub fn new(username: String, avatar_src: &str, markdown: String) -> Self {
        Self {
            username,
            avatar_url: if avatar_src.is_empty() {
                String::new()
            } else {
                avatar_url(avatar_src, 48)
            },
            markdown,
        }
    }
}

impl StatelessWidget for QuoteCard {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);

        let avatar: AnyWidget = if self.avatar_url.is_empty() {
            SizedBox::new().width(0).height(0).boxed()
        } else {
            Container::new()
                .width(AVATAR_DIAMETER)
                .height(AVATAR_DIAMETER)
                .box_decoration(BoxDecoration::new().border_radius(AVATAR_DIAMETER / 2.0))
                .child(
                    NetworkImage::new(self.avatar_url.clone())
                        .width(AVATAR_DIAMETER)
                        .height(AVATAR_DIAMETER)
                        .fit(BoxFit::Cover),
                )
                .boxed()
        };
        let header = Row::new().gaps(LayoutSpacing::all(8)).children([
            avatar,
            Text::new(format!("{}:", self.username))
                .text_style(
                    TextStyle::new()
                        .font_size(15)
                        .font_weight(FontWeight::Bold)
                        .color(theme.on_background_color().with_opacity(190)),
                )
                .boxed(),
        ]);

        let accent = BoxBorder {
            left: BorderSlice::new()
                .style(BorderStyle::Solid)
                .stroke(4)
                .color(theme.on_background_color().with_opacity(70)),
            ..Default::default()
        };
        Container::new()
            .margin(LayoutSpacing::new().top(6).bottom(6))
            .padding(LayoutSpacing::all(12))
            .box_decoration(
                BoxDecoration::new()
                    .background_color(theme.surface_color().darken(0.1).with_alpha(0.4))
                    .border(accent),
            )
            .child(Column::new().gaps(LayoutSpacing::all(6)).children([
                header.boxed(),
                forum_markdown(&self.markdown, &theme, LayoutSpacing::new()),
            ]))
    }
}

impl MarkdownCustomBlock for QuoteCard {
    const NAME: &'static str = "forum-quote";
    const OPENING: &'static str = ":::forum-quote";
    const CLOSING: &'static str = ":::";
    type Props = QuoteCard;

    fn parse(input: MarkdownCustomBlockInput<'_>) -> Result<Self::Props, MarkdownError> {
        serde_json::from_str(input.raw).map_err(|e| MarkdownError::new(e.to_string()))
    }

    fn build(props: &Self::Props, ctx: &BuildContext) -> AnyWidget {
        QuoteCard {
            username: props.username.clone(),
            avatar_url: props.avatar_url.clone(),
            markdown: props.markdown.clone(),
        }
        .boxed()
    }
}
