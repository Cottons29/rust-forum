use crate::router::AppRouter;
use aimer::input::gesture::gesture_detector::GestureDetector;
use aimer::router::NavigatorController;
use aimer::share::ShareRef;
use aimer::{AnyWidget, Widget};

/// Makes `child` open `username`'s profile when tapped.
///
/// A [`Button`](aimer::input::button::Button) hides its child from pointer
/// events, so a link cannot sit inside one: place it beside the button instead.
pub fn profile_link(
    navigator: NavigatorController<AppRouter>,
    username: &str,
    child: impl Widget + 'static,
) -> AnyWidget {
    let username = ShareRef::from_str_slice(username);
    GestureDetector::new()
        .on_tap(move || {
            navigator.push(AppRouter::UserProfile {
                username: username.clone(),
            });
        })
        .child(child)
        .boxed()
}
