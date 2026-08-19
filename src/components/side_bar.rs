mod category_row;
mod category_section;
mod nav_list;
pub mod skeleton_category_row;
mod user_menu;
use std::time::Duration;

use crate::components::side_bar::category_section::CategorySection;
use crate::components::side_bar::nav_list::NavigationRow;
use crate::components::side_bar::user_menu::UserMenuButton;
use crate::router::AppRouter;
use crate::theme::ForumTheme;
use aimer::Resizable;
use aimer::animation::{Animatable, Curve, ImplicitAnimatedBuilder};
use aimer::style::{BorderSlice, BorderStyle, BoxDecoration, BoxOutline, LayoutSpacing, Theme};
use aimer::{
    BoxAlignment, BuildContext, Color, Column, Container, Direction, Expanded, Scrollable,
    SizedBox, State, StateUpdater, StatefulWidget, Widget,
};

const SIDE_BAR_ICONS: &[&str] = &[
    "assets/app-apps-applications-tiles-svgrepo-com(1).svg",
    "assets/clock-two-thirty-svgrepo-com.svg",
    "assets/hot-svgrepo-com.svg",
];

const START_WIDTH: f32 = 250.0;
const MIN_WIDTH: f32 = 200.0;
const MAX_WIDTH: f32 = 500.0;
const HANDLE_THICKNESS: f32 = 10.0;
const HANDLE_OUTSET: f32 = 6.0;

#[derive(StatefulWidget)]
pub struct AppSideBar;

impl AppSideBar {
    pub fn new() -> Self {
        Self
    }
}

pub struct AppSidebarState {
    resizable: bool,
    updater: StateUpdater<Self>,
}

impl StatefulWidget for AppSideBar {
    type State = AppSidebarState;

    fn create_state(self) -> Self::State {
        AppSidebarState {
            resizable: false,
            updater: StateUpdater::new(),
        }
    }
}

impl State<AppSideBar> for AppSidebarState {
    fn init_state(&mut self, updater: StateUpdater<Self>)
    where
        Self: Sized,
    {
        self.updater = updater;
    }

    fn adopt_config_from(&mut self, _new: Self) {}

    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let zone_updater = self.updater.clone();
        let theme = *ForumTheme::of(ctx);

        ImplicitAnimatedBuilder::new(
            if self.resizable { 1.0 } else { 0.0 },
            Duration::from_millis(300),
            Curve::EaseInOut,
            move |delta| {
                let zone_updater = zone_updater.clone();
                let normal_stroke = 1.0;
                let resizable_stroke = 3.0;
                let border_stroke = normal_stroke.lerp(&resizable_stroke, *delta);
                let border_opacity = 0.1.lerp(&1.0, *delta);
                Container::new()
                    .box_decoration(
                        BoxDecoration::new()
                            .background_color(theme.surface_color())
                            .outline(
                                BoxOutline::new().right(
                                    BorderSlice::new()
                                        .color(theme.primary_color().with_alpha(border_opacity))
                                        .style(BorderStyle::Solid)
                                        .stroke(border_stroke),
                                ),
                            ),
                    )
                    .child(
                        Resizable::new()
                            .height(f32::MAX)
                            .width(START_WIDTH)
                            .min_width(MIN_WIDTH)
                            .max_width(MAX_WIDTH)
                            .handle_thickness(HANDLE_THICKNESS)
                            .handle_outset(HANDLE_OUTSET)
                            .direction(Direction::RIGHT)
                            .on_resize_zone(move |zone| {
                                zone_updater.set_state(move |state| {
                                    state.resizable = zone != Direction::NONE;
                                });
                            })
                            .child(
                                Container::new().padding(LayoutSpacing::all(12)).child(
                                    Column::new()
                                        .vertical_alignment(BoxAlignment::Start)
                                        .children([
                                            Expanded::new()
                                                .child(
                                                    Scrollable::new()
                                                        .vertical_scroll_bar(None)
                                                        .child(
                                                            Column::new().children([
                                                                #[cfg(target_os = "macos")]
                                                                SizedBox::new().height(32).boxed(),
                                                                Column::new()
                                                                    .gaps(LayoutSpacing::all(2))
                                                                    .children([
                                                                        NavigationRow::new(
                                                                            "Latest",
                                                                            AppRouter::Latest,
                                                                            SIDE_BAR_ICONS[0],
                                                                        ),
                                                                        NavigationRow::new(
                                                                            "New",
                                                                            AppRouter::New,
                                                                            SIDE_BAR_ICONS[1],
                                                                        ),
                                                                        NavigationRow::new(
                                                                            "Hot",
                                                                            AppRouter::Hot,
                                                                            SIDE_BAR_ICONS[2],
                                                                        ),
                                                                    ])
                                                                    .boxed(),
                                                                SizedBox::new().height(16).boxed(),
                                                                CategorySection.boxed(),
                                                            ]),
                                                        ),
                                                )
                                                .boxed(),
                                            SizedBox::new().height(12).boxed(),
                                            UserMenuButton.boxed(),
                                        ]),
                                ),
                            ),
                    )
            },
        )
    }
}

fn parse_hex_color(hex: &str) -> Color {
    match u32::from_str_radix(hex, 16) {
        Ok(rgb) => Color::Hex(rgb),
        Err(_) => Color::GRAY,
    }
}
