use std::time::Duration;

use crate::components::side_bar::{AppSideBar, MOBILE_BREAKPOINT, MobileNavbar};
use crate::router::AppRouter;
use crate::store::AppStore;
use crate::theme::ForumTheme;
use aimer::Dimension::Percent;
use aimer::animation::curves;
use aimer::provider::media_query::MediaQuery;
use aimer::router::Outlet;
use aimer::style::{AnimatedTheme, Theme, ThemeMode};
use aimer::*;

#[derive(Clone, StatefulWidget)]
pub struct AppShell {
    current_route: AppRouter,
}

impl AppShell {
    pub fn new(current_route: AppRouter) -> Self {
        Self { current_route }
    }
}

pub struct AppShellState {
    current_route: AppRouter,
}

impl StatefulWidget for AppShell {
    type State = AppShellState;

    fn create_state(self) -> Self::State {
        AppShellState {
            current_route: self.current_route,
        }
    }
}

impl State<AppShell> for AppShellState {
    fn init_state(&mut self, _updater: StateUpdater<Self>) {}

    fn adopt_config_from(&mut self, new: Self) {
        self.current_route = new.current_route;
    }

    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme_mode = ctx.select::<AppStore, ThemeMode>(|s| s.theme_mode);
        AnimatedTheme::new()
            .adaptive(ForumTheme::light(), ForumTheme::dark())
            .mode(theme_mode)
            .duration(Duration::from_millis(300))
            .curve(curves::EASE_IN_OUT)
            .child(ShellFrame)
    }
}

#[derive(Clone, StatelessWidget)]
struct ShellFrame;

impl StatelessWidget for ShellFrame {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        // Only the answer is watched, so a window drag rebuilds this at the breakpoint
        // and nowhere else.

        Container::new()
            .color(theme.background_color())
            .child(if is_mobile(ctx) {
                Stack::new()
                    .children([
                        SizedBox::new()
                            .width(Percent(100.0))
                            .height(Percent(100.0))
                            .box_child(Outlet),
                        MobileNavbar.boxed(),
                    ])
                    .boxed()
            } else {
                Row::new()
                    .justify_content(JustifyContent::SpaceBetween)
                    .children([
                        AppSideBar::new().boxed(),
                        Expanded::new().flex(0.95).box_child(Outlet),
                    ])
                    .boxed()
            })
    }
}


pub fn is_mobile(ctx: &BuildContext) -> bool {
    MediaQuery::select(ctx, |media| media.size.width < MOBILE_BREAKPOINT)
}
