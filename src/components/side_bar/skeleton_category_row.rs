use std::time::Duration;

use crate::theme::ForumTheme;
use aimer::animation::{AnimatedBuilder, AnimationController, Curve};
use aimer::style::{BoxDecoration, LayoutSpacing, Theme};
use aimer::{
    AnyWidget, BoxAlignment, BuildContext, Color, Container, Dimension, Row, SizedBox,
    StatelessWidget, Widget, ZeroSizedBox,
};
use crate::components::skeleton_topic_card::CARD_HEIGHT;

const PULSE_CYCLE: Duration = Duration::from_millis(1000);
const SWATCH_SIZE: f32 = 12.0;
const NAME_BAR_HEIGHT: f32 = 12.0;
const NAME_BAR_WIDTH: f32 = 110.0;

/// A grey placeholder that mirrors [`super::side_bar::category_row::CategoryRow`]'s
/// shape — a colour swatch and a name — and pulses gently while the categories
/// stream in.
#[derive(Clone, StatelessWidget)]
pub struct SkeletonCategoryRow {
    controller: AnimationController,
}

impl SkeletonCategoryRow {
    pub fn new() -> Self {
        let controller = AnimationController::new(PULSE_CYCLE, Curve::EaseInOut);
        controller.set_repeat(true);
        controller.set_auto_reverse(true);
        controller.forward_from_first_tick();
        Self { controller }
    }
}

impl StatelessWidget for SkeletonCategoryRow {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let controller = self.controller.clone();

        AnimatedBuilder::new(controller, move |value| {
            let tone = theme.primary_color().with_brightness(1.2);
            let bar = tone.with_alpha(0.12).lerp(tone.with_alpha(0.3), value);

            Container::new()
                .padding(LayoutSpacing::vertical(12)).child(
                Row::new()
                    .vertical_alignment(BoxAlignment::Center)
                    .gaps(LayoutSpacing::all(10))
                    .children([
                        SizedBox::new().width(4).boxed(),
                        // The category colour swatch.
                        Container::new()
                            .width(Dimension::Px(SWATCH_SIZE))
                            .height(Dimension::Px(SWATCH_SIZE))
                            .box_decoration(
                                BoxDecoration::new()
                                    .background_color(bar)
                                    .border_radius(SWATCH_SIZE / 2.0),
                            )
                            .child(ZeroSizedBox)
                            .boxed(),
                        // The category name.
                        bar_widget(bar, NAME_BAR_HEIGHT, NAME_BAR_WIDTH),
                        SizedBox::new().width(4).boxed(),
                    ]),
            )
        })
    }
}

fn bar_widget(color: Color, height: f32, width: f32) -> AnyWidget {
    Container::new()
        .height(Dimension::Px(height))
        .width(Dimension::Px(width))
        .box_decoration(
            BoxDecoration::new()
                .background_color(color)
                .border_radius(6),
        )
        .child(ZeroSizedBox)
        .boxed()
}
