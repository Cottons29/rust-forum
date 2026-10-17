use crate::components::general_button::GeneralButton;
use crate::router::AppRouter;
use crate::theme::ForumTheme;
use aimer::console::info;
use aimer::router::NavigatorController;
use aimer::style::{LayoutSpacing, TextStyle, Theme};
use aimer::{
    Align, Alignment, AnyWidget, BoxAlignment, BuildContext, Container, Dimension, Expanded, Icons, Row,
    SizedBox, StatelessWidget, StaticSvg, Svg, SvgStyle, Text, Widget,
};

/// Diameter of the round back button, and the height of the like/comment buttons.
const BUTTON_SIZE: f32 = 40.0;
const ICON_SIZE: u32 = 20;
/// The like and comment buttons are wide enough for an icon and a short count.
const COUNT_BUTTON_WIDTH: f32 = 72.0;

#[derive(Clone, StatelessWidget)]
pub struct PostActionBar {
    like_count: i64,
    comment_count: i64,
    post_id: i64,
}

impl PostActionBar {
    pub(crate) fn new(like_count: i64, comment_count: i64, post_id: i64) -> Self {
        Self {
            like_count,
            comment_count,
            post_id,
        }
    }
}

impl StatelessWidget for PostActionBar {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let like_post_id = self.post_id;
        let comment_post_id = self.post_id;
        Container::new()
            .height(60)
            .padding(LayoutSpacing::all(10))
            .child(
                Row::new()
                    .vertical_alignment(BoxAlignment::Center)
                    .horizontal_alignment(BoxAlignment::Center)
                    .gaps(LayoutSpacing::horizontal(4.into()))
                    .children([
                        back_button(ctx, theme),
                        Expanded::new().box_child(SizedBox::new()),
                        GeneralButton::with_child(
                            button_content(
                                theme,
                                Icons::HEART,
                                Some(self.like_count),
                                COUNT_BUTTON_WIDTH,
                            ),
                            move || info!("Like requested for post {}", like_post_id),
                        )
                        .width(COUNT_BUTTON_WIDTH)
                        .boxed(),
                        GeneralButton::with_child(
                            button_content(
                                theme,
                                Icons::MESSAGE_CIRCLE,
                                Some(self.comment_count),
                                COUNT_BUTTON_WIDTH,
                            ),
                            move || info!("Comment requested for post {}", comment_post_id),
                        )
                        .width(COUNT_BUTTON_WIDTH)
                        .boxed(),
                    ]),
            )
    }
}

/// The round button that goes back a page, or home when there is no page behind.
pub(crate) fn back_button(ctx: &BuildContext, theme: ForumTheme) -> AnyWidget {
    let navigator = NavigatorController::of(ctx);
    GeneralButton::with_child(
        button_content(theme, Icons::CHEVRON_LEFT, None, BUTTON_SIZE),
        move || {
            if navigator.can_pop() {
                navigator.pop();
            } else {
                navigator.push(AppRouter::Latest);
            }
        },
    )
    .width(BUTTON_SIZE)
    .boxed()
}

/// What goes inside a button: the icon, with its count beside it when it has one.
fn button_content(
    theme: ForumTheme,
    icon: &'static StaticSvg,
    count: Option<i64>,
    width: f32,
) -> impl Widget {
    let icon = Svg::new(icon)
        .style("path", SvgStyle::new().stroke(theme.on_background_color()))
        .width(ICON_SIZE)
        .height(ICON_SIZE);

    let content = match count {
        Some(count) => Row::new()
            .vertical_alignment(BoxAlignment::Center)
            .gaps(LayoutSpacing::horizontal(6.into()))
            .children([
                icon.boxed(),
                Text::new(count.to_string())
                    .text_style(
                        TextStyle::new()
                            .font_size(14)
                            .color(theme.on_background_color()),
                    )
                    .boxed(),
            ])
            .boxed(),
        None => icon.boxed(),
    };

    Container::new()
        .width(Dimension::Px(width))
        .height(Dimension::Px(BUTTON_SIZE))
        .child(Align::new().alignment(Alignment::MidCenter).child(content))
}
