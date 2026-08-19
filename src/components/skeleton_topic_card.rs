use std::time::Duration;

use crate::theme::ForumTheme;
use aimer::animation::{AnimatedBuilder, AnimationController, Curve};
use aimer::style::{BorderSlice, BorderStyle, BoxBorder, BoxDecoration, LayoutSpacing, Theme};
use aimer::{
    BuildContext, Color, Column, Container, Dimension, JustifyContent, StatelessWidget, Widget,
    ZeroSizedBox,
};

const PULSE_CYCLE: Duration = Duration::from_millis(1000);
pub(crate) const CARD_HEIGHT: f32 = 160.0;

/// A grey placeholder that mirrors [`super::topic_card::TopicCard`]'s shape and
/// pulses gently while the real topic streams in.
#[derive(Clone, StatelessWidget)]
pub struct SkeletonTopicCard {
    controller: AnimationController,
}

impl SkeletonTopicCard {
    pub fn new() -> Self {
        let controller = AnimationController::new(PULSE_CYCLE, Curve::EaseInOut);
        controller.set_repeat(true);
        controller.set_auto_reverse(true);
        controller.forward_from_first_tick();
        Self { controller }
    }
}

impl StatelessWidget for SkeletonTopicCard {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let controller = self.controller.clone();

        AnimatedBuilder::new(controller, move |value| {
            let tone = theme.primary_color().with_brightness(1.1);
            let bar = tone.with_alpha(0.12).lerp(tone.with_alpha(0.3), value);

            Container::new()
                .height(CARD_HEIGHT)
                .width(Dimension::Auto)
                .box_decoration(theme.box_decor_with_color(bar))
                .child(
                    Container::new().padding(LayoutSpacing::all(24)).child(
                        Column::new()
                            .justify_content(JustifyContent::SpaceBetween)
                            .gaps(LayoutSpacing::all(10))
                            .children([bar_widget(bar, 24.0, 80.0), bar_widget(bar, 16.0, 45.0)]),
                    ),
                )
        })
    }
}

fn bar_widget(color: Color, height: f32, width_percent: f32) -> impl Widget {
    Container::new()
        .height(Dimension::Px(height))
        .width(Dimension::Percent(width_percent))
        .box_decoration(
            BoxDecoration::new()
                .background_color(color)
                .border_radius(6),
        )
        .child(ZeroSizedBox)
}
