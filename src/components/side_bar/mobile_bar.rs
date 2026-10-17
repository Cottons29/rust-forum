use crate::components::side_bar::SIDE_BAR_ICONS;
use crate::components::side_bar::glass::GlassDecoration;
use crate::components::side_bar::user_menu::UserAvatarButton;
use crate::router::AppRouter;
use crate::theme::ForumTheme;
use aimer::decoration::*;
use aimer::indicator::{TabBar, TabItem, indicator_styles};
use aimer::router::Navigator;
use aimer::style::{BoxShadow, LayoutSpacing, TextAlign, Theme};
use aimer::{
    Align, Alignment, AnyWidget, BoxAlignment, BuildContext, Color, Container, Dimension,
    JustifyContent, Row, StatelessWidget, SvgAsset, SvgStyle, Widget,
};

/// Windows narrower than this (logical pixels) get the floating tab bar.
pub const MOBILE_BREAKPOINT: f32 = 600.0;

/// The tabs, in the order the bar shows them. Each is a place the bar can send the
/// app to; any other page (a category, a post, the settings) is none of them.
const TABS: [(AppRouter, &str, &str); 3] = [
    (AppRouter::Latest, "Latest", SIDE_BAR_ICONS[0]),
    (AppRouter::New, "New", SIDE_BAR_ICONS[1]),
    (AppRouter::Hot, "Hot", SIDE_BAR_ICONS[2]),
];

const TAB_WIDTH: f32 = 72.0;
const TAB_HEIGHT: f32 = 52.0;
/// The bar pads its tabs by this much on every side, so it is this much taller.
const BAR_PADDING: f32 = 6.0;
const BAR_HEIGHT: f32 = TAB_HEIGHT + BAR_PADDING * 2.0;
/// The account button inside its circle, which is as tall as the bar.
const AVATAR_BUTTON_SIZE: f32 = 48.0;
const BAR_GAP: f32 = 12.0;
const ICON_SIZE: u32 = 24;

/// The sidebar on a narrow window: a floating tab bar and the account button.
///
/// The tab bar sends the app to Latest, New or Hot. Its indicator slides to the
/// current tab and lifts into a lens while it moves; it can also be dragged
/// along the bar. On a page that is none of the three it shows no selection. The
/// bar floats above the page, centred along the bottom edge, so the page keeps
/// the whole window instead of losing a column to the sidebar.
#[derive(Clone, StatelessWidget)]
pub struct MobileNavbar;

impl StatelessWidget for MobileNavbar {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let mut theme = *ForumTheme::of(ctx);
        let bottom_inset = aimer::safe_area_insets().bottom;
        let navigator = Navigator::<AppRouter>::of(ctx);

        // The tab the app is on, or none: the bar is controlled, so it follows
        // the navigator and only proposes where the user would like to go.
        let selection = selected_tab(|route| navigator.contains_route(route));

        let tab_bar = tab_bar(theme, selection, {
            let navigator = navigator.clone();
            move |index| {
                let route = TABS[index].0.clone();
                if navigator.current_route() != route {
                    navigator.set_route(route);
                }
            }
        });

        Align::new().alignment(Alignment::BotCenter).box_child(
            Container::new()
                .height(120)
                .child(
                    Row::new()
                        // The container fills the window's width, so the row has to
                        // centre its own contents.
                        .justify_content(JustifyContent::Center)
                        .vertical_alignment(BoxAlignment::Center)
                        .gaps(LayoutSpacing::horizontal((BAR_GAP as u32).into()))
                        .children([tab_bar, avatar_circle(theme)]),
                ),
        )
    }
}


impl MobileNavbar {
    pub fn style(theme: ForumTheme) -> impl Decoration {
        glass(theme)
    }
}

/// The index of the tab the app is on, given whether each route is showing, or
/// `None` on a page that is none of them.
fn selected_tab(is_showing: impl Fn(&AppRouter) -> bool) -> Option<usize> {
    TABS.iter().position(|(route, _, _)| is_showing(route))
}

/// The floating shadow both the tab bar and the avatar circle sit on.
fn floating_shadow() -> BoxShadow {
    BoxShadow::new()
        .color(Color::BLACK.with_opacity(40))
        .blur(18.0)
        .offset_y(6.0)
}

/// How much of the blurred page shows through the glass. The tint is the theme's
/// surface color, so the bar still reads as the same surface in light and dark.
const GLASS_TINT_ALPHA: f32 = 0.45;

/// The frosted glass the tab bar and the avatar circle are both made of: the
/// page behind them blurred, tinted with the surface color, with a light edge.
fn glass(theme: ForumTheme) -> impl Decoration {
    GlassDecoration::new(theme.surface_color().with_alpha(0.3))
        .light_edge(LightEdge::new().width(3.0).strength(0.6))
        .lens(
            Magnification::new()
                .zoom(1.3)
                .thickness(15.0)
                .bulge(0.3)
                .style(
                    MagnificationStyle::REFRACTION | MagnificationStyle::UNIFORM | MagnificationStyle::BULGE
                ),
        )
        .backdrop(
            BackdropFilter::new()
                .blur_radius(4.0)
                .saturation(0.65)
                .brightness(1.0)
                .contrast(0.9),
        )
}

/// The three tabs, themed: frosted glass behind them, the accent color on the
/// selected one, and a soft accent wash for the pill the lens rests as.
fn tab_bar(
    theme: ForumTheme,
    selection: Option<usize>,
    on_select: impl Fn(usize) + 'static,
) -> AnyWidget {
    TabBar::new()
        .selection(selection)
        .on_select(on_select)
        .item_size(TAB_WIDTH, TAB_HEIGHT)
        .bar_decoration(glass(theme))
        .inactive_color(theme.on_surface_color())
        .selected_color(theme.primary_color())
        .style(indicator_styles::LIQUID_GLASS.rest_fill(theme.primary_color().with_alpha(0.2)))
        .items(TABS.iter().map(|(_, label, svg_path)| {
            let svg_path = *svg_path;
            TabItem::new()
                .label(*label)
                .icon(move |tint| tab_icon(svg_path, tint))
        }))
        .boxed()
}

/// A tab's icon, drawn in `tint`.
fn tab_icon(svg_path: &'static str, tint: Color) -> SvgAsset {
    SvgAsset::new(svg_path)
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .style(".icon-path", SvgStyle::new().stroke(tint))
        .style(".icon-path-fill", SvgStyle::new().fill(tint))
}

/// The account button, in a circle of the same glass as the bar.
fn avatar_circle(theme: ForumTheme) -> AnyWidget {
    Container::new()
        .width(Dimension::Px(BAR_HEIGHT))
        .height(Dimension::Px(BAR_HEIGHT))
        // .decoration(glass(theme))
        .child(
            Align::new()
                .alignment(TextAlign::MidCenter)
                .child(UserAvatarButton::new(BAR_HEIGHT)),
        )
        .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aimer::{router::Route, share::ShareRef};

    #[test]
    fn each_tab_route_selects_its_own_tab() {
        for (index, (route, _, _)) in TABS.iter().enumerate() {
            assert_eq!(selected_tab(|candidate| candidate == route), Some(index));
        }
    }

    #[test]
    fn a_page_that_is_none_of_the_tabs_selects_nothing() {
        let other = [
            AppRouter::Settings,
            AppRouter::CategoryTopics {
                slug: ShareRef::from_static("rust"),
                id: 12,
            },
            AppRouter::PostDetail {
                slug: ShareRef::from_static("hello"),
                id: 345,
            },
        ];
        for route in &other {
            assert_eq!(
                selected_tab(|candidate| candidate == route),
                None,
                "{}",
                route.format()
            );
        }
    }

    #[test]
    fn the_tabs_are_the_three_the_sidebar_lists_with_one_icon_each() {
        let labels: Vec<_> = TABS.iter().map(|(_, label, _)| *label).collect();
        assert_eq!(labels, ["Latest", "New", "Hot"]);
        for (_, _, icon) in TABS {
            assert!(SIDE_BAR_ICONS.contains(&icon), "{icon}");
        }
    }
}
