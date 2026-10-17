use crate::components::side_bar::MobileNavbar;
use crate::theme::ForumTheme;
use aimer::animation::{Animatable, ImplicitAnimatedBuilder};
use aimer::canvas::MagnificationStyle;
use aimer::decoration::{Decoration, LightEdge, Magnification};
use aimer::input::button::{Button, ButtonVisualState};
use aimer::input::callback::VoidCallback;
use aimer::share::Shared;
use aimer::style::{BorderRadius, BoxDecoration, TextAlign, TextStyle, Theme, curves};
use aimer::{
    Align, Alignment, BuildContext, ChildBuilder, Color, Container, Scalable,
    StatelessWidget, Text, Widget,
};
use std::time::Duration;

#[derive(StatelessWidget)]
pub struct GeneralButton {
    label: Shared<str>,
    on_click: VoidCallback,
    child: Option<ChildBuilder>,
    width: f32,
}

impl GeneralButton {
    pub fn new(label: impl Into<Shared<str>>, on_click: impl Into<VoidCallback>) -> Self {
        Self {
            label: label.into(),
            on_click: on_click.into(),
            width: 100f32,
            child: None,
        }
    }

    pub fn with_child(child: impl Widget + 'static, on_click: impl Into<VoidCallback>) -> Self {
        Self {
            label: "".into(),
            on_click: on_click.into(),
            width: 100f32,
            child: Some(ChildBuilder::from_widget(child)),
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl StatelessWidget for GeneralButton {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = ForumTheme::of(ctx);
        let width = self.width;
        let label = self.label.as_share_ref();
        let child = self.child.clone();

        Button::new()
            .no_decor()
            .on_press(self.on_click.clone())
            .build(move |s| {
                let label = label.clone();
                let theme = theme.clone();
                let child = child.clone();
                ImplicitAnimatedBuilder::new(
                    get_button_scale(&s, &theme),
                    Duration::from_millis(300),
                    curves::FAST_OUT_SLOW_IN,
                    move |style| {
                        Scalable::new().scale(style.scale).child(
                            Container::new()
                                .width(width)
                                .decoration([
                                    BorderRadius::all(50).notype(),
                                    style.color.notype(),
                                    // Color::YELLOW.notype(),
                                    MobileNavbar::style(*theme).notype()
                                ])
                                .child(
                                    Align::new()
                                        .alignment(Alignment::MidCenter)
                                        .child(match child.clone() {
                                            Some(c) => c.boxed(),
                                            None => Text::new(label.clone())
                                                .text_align(TextAlign::MidCenter)
                                                .text_style(
                                                    TextStyle::new()
                                                        .font_size(16)
                                                        .color(theme.on_background_color()),
                                                )
                                                .boxed(),
                                        }),
                                ),
                        )
                    },
                )
            })
    }
}

#[derive(Copy, Clone, Animatable, PartialEq)]
struct ButtonStyle {
    scale: f32,
    color: Color,
}

impl ButtonStyle {
    pub fn new(scale: f32, color: Color) -> Self {
        Self { scale, color }
    }
}

fn get_button_scale(s: &ButtonVisualState, theme: &ForumTheme) -> ButtonStyle {
    if s.is_pressed {
        ButtonStyle::new(1.15, theme.primary_color().with_alpha(0.2))
    } else if s.is_hovered {
        ButtonStyle::new(1.0, theme.primary_color().with_alpha(0.01))
    } else {
        ButtonStyle::new(1.0, Color::Transparent)
    }
}
