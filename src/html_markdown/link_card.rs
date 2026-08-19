use crate::theme::ForumTheme;
use aimer::style::{
    BorderSlice, BorderStyle, BoxAlignment, BoxBorder, BoxDecoration, BoxFit, BoxShadow,
    LayoutSpacing, TextOverflow, TextStyle, Theme,
};
use aimer::{
    AnyWidget, BuildContext, Button, Color, Column, Container, Dimension, Expanded, FontWeight,
    MarkdownCustomBlock, MarkdownCustomBlockInput, MarkdownError, NetworkImage, Row,
    StatelessWidget, Text, Widget, ZeroSizedBox,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, StatelessWidget)]
pub enum LinkCard {
    GenericLinkCard {
        src: String,
        header: String,
        header_img: String,
        thumbnail_src: String,
        short_intro: String,
        gh_description: String,
    },
    GHRepoCard {
        src: String,
        header: String,
        thumbnail_src: String,
        short_intro: String,
        gh_description: String,
    },
}

impl StatelessWidget for LinkCard {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        self.card_ui(ctx)
    }
}

impl LinkCard {
    fn card_ui(&self, ctx: &BuildContext) -> AnyWidget {
        let theme = *ForumTheme::of(ctx);

        let (header_img, header, thumbnail_src, short_intro, gh_description) = match self {
            LinkCard::GenericLinkCard {
                header,
                header_img,
                thumbnail_src,
                short_intro,
                gh_description,
                ..
            } => (
                Some(header_img.as_str()),
                header.as_str(),
                thumbnail_src.as_str(),
                short_intro.as_str(),
                gh_description.as_str(),
            ),
            LinkCard::GHRepoCard {
                header,
                thumbnail_src,
                short_intro,
                gh_description,
                ..
            } => (
                None,
                header.as_str(),
                thumbnail_src.as_str(),
                short_intro.as_str(),
                gh_description.as_str(),
            ),
        };

        // Header row: an optional site icon (generic cards only) followed by
        // the source/site name.

        let mut header_row = Row::new()
            .gaps(LayoutSpacing::all(8))
            .vertical_alignment(BoxAlignment::Center);
        if let Some(icon) = header_img {
            header_row = header_row.add_child(
                NetworkImage::new(icon.to_string())
                    .error_widget(Self::error_header_icon(16, 16))
                    .width(16)
                    .height(16)
                    .fit(BoxFit::Contain),
            );
        }
        let header_row = header_row.add_child(
            Expanded::new().box_child(
                Text::new(header.to_string()).text_style(
                    TextStyle::new()
                        .font_size(20)
                        .font_weight(FontWeight::Bold)
                        .text_overflow(TextOverflow::Ellipsis)
                        .color(theme.on_surface_color().with_opacity(150)),
                ),
            ),
        );

        const IMAGE_WIDTH: u32 = 130;
        const IMAGE_HEIGHT: u32 = 64;

        let body_row = Row::new().gaps(LayoutSpacing::all(12)).children([
            Container::new()
                .width(IMAGE_WIDTH)
                .height(IMAGE_HEIGHT)
                .box_decoration(BoxDecoration::new().border_radius(4))
                .box_child(
                    NetworkImage::new(thumbnail_src.to_string())
                        .error_widget(Self::error_header_icon(IMAGE_WIDTH, IMAGE_HEIGHT))
                        .height(IMAGE_HEIGHT)
                        .fit(BoxFit::FitHeight),
                ),
            Expanded::new().box_child(
                Column::new()
                    .gaps(LayoutSpacing::new().bottom(8))
                    .children([
                        Text::new(short_intro.to_string()).text_style(
                            TextStyle::new()
                                .font_size(20)
                                .font_weight(FontWeight::Bold)
                                .text_overflow(TextOverflow::Wrap)
                                .color(theme.on_surface_color()),
                        ),
                        Text::new(gh_description.to_string()).text_style(
                            TextStyle::new()
                                .font_size(16)
                                .text_overflow(TextOverflow::Wrap)
                                .color(theme.on_surface_color().with_opacity(150)),
                        ),
                    ]),
            ),
        ]);

        let src = self.src().to_string();

        Button::new()
            .hover_decoration(theme.hover_box_decor())
            .decoration(theme.box_decor())
            .on_press(move || {
                if let Err(e) = webbrowser::open(&src) {
                    eprintln!("Failed to open browser: {}", e);
                }
            })
            .box_child(
                Container::new()
                    .width(Dimension::Auto)
                    .padding(LayoutSpacing::all(16))
                    .child(
                        Column::new()
                            .gaps(LayoutSpacing::all(12))
                            .box_children([header_row, body_row]),
                    ),
            )
    }

    fn src(&self) -> &str {
        match self {
            Self::GenericLinkCard { src, .. } => src,
            Self::GHRepoCard { src, .. } => src,
        }
    }

    fn error_header_icon(width: impl Into<Dimension>, height: impl Into<Dimension>) -> impl Widget {
        Container::new()
            .width(width)
            .height(height)
            .box_decoration(
                BoxDecoration::new()
                    .border(BoxBorder::all(
                        BorderSlice::new()
                            .color(Color::GRAY.with_alpha(0.6))
                            .style(BorderStyle::Solid)
                            .stroke(1),
                    ))
                    .border_radius(4),
            )
            .child(ZeroSizedBox)
    }
}

impl MarkdownCustomBlock for LinkCard {
    const NAME: &'static str = "link-card";
    const OPENING: &'static str = ":::link-card";
    const CLOSING: &'static str = ":::";
    type Props = LinkCard;

    fn parse(input: MarkdownCustomBlockInput<'_>) -> Result<Self::Props, MarkdownError> {
        serde_json::from_str(input.raw).map_err(|e| MarkdownError::new(e.to_string()))
    }

    fn build(props: &Self::Props, ctx: &BuildContext) -> AnyWidget {
        props.card_ui(ctx)
    }
}
