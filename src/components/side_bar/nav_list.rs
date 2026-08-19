use crate::router::AppRouter;
use crate::theme::ForumTheme;
use aimer::router::Navigator;
use aimer::style::{BoxDecoration, LayoutSpacing, TextStyle, Theme};
use aimer::{
    BoxAlignment, BuildContext, Button, Color, Container, Row, SizedBox, StatelessWidget, SvgAsset,
    SvgStyle, Text, Widget,
};

#[derive(StatelessWidget)]
pub struct NavigationRow {
    label: &'static str,
    route: AppRouter,
    svg_path: &'static str,
}

impl NavigationRow {
    pub fn new(label: &'static str, router: AppRouter, svg_path: &'static str) -> Self {
        Self {
            label,
            route: router,
            svg_path,
        }
    }
}

impl StatelessWidget for NavigationRow {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let navigator = Navigator::<AppRouter>::of(ctx);
        let route = self.route.clone();
        let theme = ForumTheme::of(ctx);
        let is_active_route = navigator.contains_route(&self.route);

        let should_activate = navigator.contains_route(&route);
        let can_route = navigator.current_route() != route;
        let box_decoration = BoxDecoration::new()
            .background_color(if should_activate {
                theme.primary_color()
            } else {
                Color::Transparent
            })
            .border_radius(8);

        let hover_decor = box_decoration.clone().background_color(if should_activate {
            theme.primary_color()
        } else {
            theme.primary_color().with_alpha(0.2)
        });

        let svg_color = if !is_active_route {
            theme.primary_color()
        } else {
            Color::WHITE
        };

        Button::new()
            .hover_decoration(hover_decor)
            .decoration(box_decoration)
            .on_press(move || {
                if can_route {
                    navigator.set_route(route.clone());
                }
            })
            .child(
                Container::new().padding(LayoutSpacing::vertical(6).left(18)).child(
                    Row::new()
                        .vertical_alignment(BoxAlignment::Center)
                        .gaps(LayoutSpacing::all(10))
                        .children([
                            SvgAsset::new(self.svg_path)
                                .width(24)
                                .height(24)
                                .style(".icon-path", SvgStyle::new().stroke(svg_color.into()))
                                .style(".icon-path-fill", SvgStyle::new().fill(svg_color))
                                .boxed(),
                            Text::new(self.label)
                                .text_style(TextStyle::new().font_size(16).color(
                                    if should_activate {
                                        Color::WHITE
                                    } else {
                                        theme.on_surface_color()
                                    },
                                ))
                                .boxed(),
                        ]),
                ),
            )
    }
}
