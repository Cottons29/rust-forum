use crate::models::Topic;
use crate::router::AppRouter;
use crate::store::AppStore;
use crate::theme::ForumTheme;
use aimer::router::NavigatorController;
use aimer::style::{BoxDecoration, LayoutSpacing, TextOverflow, TextStyle, Theme};
use aimer::{
    BuildContext, Button, Column, Container, Dimension, FontWeight, JustifyContent,
    ProviderContext, StatelessWidget, Text, Widget,
};

#[derive(Clone, StatelessWidget)]
pub struct TopicCard {
    topic: Topic,
}

impl TopicCard {
    pub fn new(topic: Topic) -> Self {
        Self { topic }
    }
}

impl StatelessWidget for TopicCard {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let navigator = NavigatorController::<AppRouter>::of(ctx);
        let theme = *ForumTheme::of(ctx);
        let title = self.topic.title.clone();
        let slug = self.topic.slug.clone();
        let id = self.topic.id;

        let category_id = self.topic.category_id;
        let category = ctx
            .select::<AppStore, Option<String>>(move |store| store.category_name(category_id))
            .unwrap_or_else(|| format!("Category {category_id}"));

        let replies = self.topic.posts_count.saturating_sub(1);
        let views = self.topic.views;
        let activity = relative_time(last_activity(&self.topic));
        let meta = format!("{category} · {replies} replies · {views} views · {activity}");

        Container::new().height(160).width(Dimension::Auto).child(
            Button::new()
                .decoration(theme.box_decor())
                .hover_decoration(theme.hover_box_decor())
                .on_press(move || {
                    println!("Pressed on Topic ID: {}", id);
                    navigator.push(AppRouter::PostDetail {
                        slug: slug.clone(),
                        id,
                    });
                })
                .child(
                    Container::new().padding(LayoutSpacing::all(24)).child(
                        Column::new()
                            .gaps(LayoutSpacing::all(10))
                            .justify_content(JustifyContent::SpaceBetween)
                            .children([
                                Text::new(self.topic.title.clone()).text_style(
                                    TextStyle::new()
                                        .text_overflow(TextOverflow::Wrap)
                                        .font_weight(FontWeight::Bold)
                                        .font_size(24)
                                        .color(theme.on_surface_color()),
                                ),
                                Text::new(meta).text_style(
                                    TextStyle::new()
                                        .font_size(16)
                                        .color(theme.on_surface_color().with_opacity(160)),
                                ),
                            ]),
                    ),
                ),
        )
    }
}

/// The most recent activity timestamp for a topic, preferring the last post,
/// then the bump time, then creation.
fn last_activity(topic: &Topic) -> &str {
    if !topic.last_posted_at.is_empty() {
        &topic.last_posted_at
    } else if !topic.bumped_at.is_empty() {
        &topic.bumped_at
    } else {
        &topic.created_at
    }
}

/// Formats a Discourse ISO-8601 timestamp as a short relative time
/// (e.g. `just now`, `5m ago`, `3h ago`, `12d ago`, `4mo ago`), falling back
/// to the raw date for anything older than a year.
fn relative_time(iso: &str) -> String {
    let Some((year, month, day, hour, minute, second)) = parse_iso8601(iso) else {
        return "—".to_string();
    };
    let days = days_from_civil(year, month as i64, day as i64);
    let ts = days * 86_400 + hour as i64 * 3_600 + minute as i64 * 60 + second as i64;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let delta = now - ts;
    if delta < 60 {
        "just now".to_string()
    } else if delta < 3_600 {
        format!("{}m ago", delta / 60)
    } else if delta < 86_400 {
        format!("{}h ago", delta / 3_600)
    } else if delta < 7 * 86_400 {
        format!("{}d ago", delta / 86_400)
    } else if delta < 365 * 86_400 {
        format!("{}mo ago", delta / (30 * 86_400))
    } else {
        format!("{year}-{month:02}-{day:02}")
    }
}

/// Parses a timestamp like `2022-06-24T15:56:48.339Z` into its date/time parts.
fn parse_iso8601(iso: &str) -> Option<(i64, u32, u32, u32, u32, u32)> {
    let year = iso.get(0..4)?.parse::<i64>().ok()?;
    let month = iso.get(5..7)?.parse::<u32>().ok()?;
    let day = iso.get(8..10)?.parse::<u32>().ok()?;
    let hour = iso.get(11..13)?.parse::<u32>().ok()?;
    let minute = iso.get(14..16)?.parse::<u32>().ok()?;
    let second = iso.get(17..19)?.parse::<u32>().ok()?;
    Some((year, month, day, hour, minute, second))
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Howard Hinnant's
/// algorithm), used to turn a timestamp's calendar date into a second count.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_from_civil_matches_known_dates() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(1970, 1, 2), 1);
        assert_eq!(days_from_civil(2000, 1, 1), 10_957);
        assert_eq!(days_from_civil(2022, 6, 24), 19_167);
    }

    #[test]
    fn parse_iso8601_reads_parts() {
        assert_eq!(
            parse_iso8601("2022-06-24T15:56:48.339Z"),
            Some((2022, 6, 24, 15, 56, 48))
        );
    }
}
