mod skeleton;

use crate::api::Api;
use crate::app_shell::is_mobile;
use crate::components::post_action_bar::back_button;
use crate::components::side_bar::parse_hex_color;
use crate::components::topic_card::{count_label, divider_decor, format_count, relative_time};
use crate::html_markdown::{forum_markdown, html_to_markdown};
use crate::models::comments::avatar_url;
use crate::models::user::{SummaryTopic, UserProfile, UserRoot, UserSummaryRoot, format_duration};
use crate::router::AppRouter;
use crate::screen::user_profile::skeleton::SkeletonProfile;
use crate::store::AppStore;
use crate::theme::ForumTheme;
use aimer::input::button::Button;
use aimer::router::NavigatorController;
use aimer::share::ShareRef;
use aimer::style::{BoxDecoration, BoxFit, LayoutSpacing, TextOverflow, TextStyle, Theme};
use aimer::{
    Align, Alignment, AnyWidget, AsyncBuilder, AsyncSnapshot, BoxAlignment, BuildContext, Column,
    Container, Dimension, Expanded, FontWeight, Icons, NetworkImage, Positioned, ProviderHandle,
    Row, Scrollable, SizedBox, Stack, StatelessWidget, StaticSvg, Svg, SvgStyle, Text, Widget,
    ZeroSizedBox, case,
};

const AVATAR_SIZE: f32 = 88.0;
const ICON_SIZE: u32 = 16;

/// A forum member's profile: who they are, how active they are, their best
/// topics, where they post and the badges they hold.
#[derive(Clone, StatelessWidget)]
pub struct UserProfileScreen {
    username: ShareRef<str>,
}

impl UserProfileScreen {
    pub fn new(username: ShareRef<str>) -> Self {
        Self { username }
    }
}

impl StatelessWidget for UserProfileScreen {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let store = ProviderHandle::<AppStore>::of(ctx);
        let username = self.username.clone();
        let key = username.to_string();

        let data = AsyncBuilder::new()
            .request_key(key.clone())
            .key(format!("profile-{key}"))
            .future(move || {
                let store = store.clone();
                let username = username.clone();
                async move { fetch_cached_profile(store, username).await }
            })
            .child(move |snapshot| match snapshot {
                AsyncSnapshot::Waiting => SkeletonProfile::new().boxed(),
                AsyncSnapshot::Error(error) => message(theme, error),
                AsyncSnapshot::Data(profile) => ProfileContent {
                    profile: profile.clone(),
                }
                .boxed(),
            });

        let top_bar = Container::new()
            .height(60)
            .padding(LayoutSpacing::all(10))
            .child(
                Row::new()
                    .vertical_alignment(BoxAlignment::Center)
                    .children([
                        back_button(ctx, theme),
                        Expanded::new().box_child(SizedBox::new()),
                    ]),
            );

        Container::new().color(theme.background_color()).child(
            Stack::new().children([
                Positioned::new()
                    .layer(3)
                    .top(case!(is_mobile(ctx), 45))
                    .box_child(top_bar),
                Scrollable::new()
                    .box_child(
                    Column::new().children([
                        SizedBox::new()
                            .height(case!(is_mobile(ctx), 45) + 70)
                            .boxed(),
                        data.boxed(),
                        SizedBox::new().height(90).boxed(),
                    ]),
                ),
            ]),
        )
    }
}

async fn fetch_cached_profile(
    store: ProviderHandle<AppStore>,
    username: ShareRef<str>,
) -> Result<UserProfile, String> {
    if let Some(profile) = store.read().cached_profile(&username) {
        return Ok(profile);
    }

    let detail = Api::get::<UserRoot>(&format!("u/{username}.json"))
        .await
        .map_err(|e| format!("Error: {e}"))?
        .user;
    let summary = Api::get::<UserSummaryRoot>(&format!("u/{username}/summary.json"))
        .await
        .map_err(|e| format!("Error: {e}"))?;

    let profile = UserProfile { detail, summary };
    store.update(|s| s.cache_profile(&username, profile.clone()));
    Ok(profile)
}

/// A line of plain text standing in for the profile while it loads or fails.
fn message(theme: ForumTheme, text: &str) -> AnyWidget {
    Container::new().padding(LayoutSpacing::all(24)).box_child(
        Text::new(text.to_string()).text_style(
            TextStyle::new()
                .text_overflow(TextOverflow::Wrap)
                .font_size(16)
                .color(theme.on_background_color().with_opacity(160)),
        ),
    )
}

/// The loaded profile. Reads the theme in its own `build` so it follows theme
/// changes, independent of the `AsyncBuilder`'s snapshot.
#[derive(Clone, StatelessWidget)]
struct ProfileContent {
    profile: UserProfile,
}

impl StatelessWidget for ProfileContent {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let profile = &self.profile;
        let summary = &profile.summary.user_summary;

        let mut sections = vec![header(theme, profile)];
        if let Some(bio) = profile
            .detail
            .bio_cooked
            .as_deref()
            .filter(|bio| !bio.is_empty())
        {
            sections.push(forum_markdown(
                &html_to_markdown(bio),
                &theme,
                LayoutSpacing::new(),
            ));
        }
        sections.push(stats(theme, profile));

        if !profile.summary.topics.is_empty() {
            sections.push(section_title(theme, "Top topics"));
            sections.push(
                Column::new()
                    .children(
                        profile
                            .summary
                            .topics
                            .iter()
                            .map(|topic| {
                                TopicRow {
                                    slug: ShareRef::from_str_slice(&topic.slug),
                                    id: topic.id,
                                    title: topic.title.clone(),
                                    likes: topic.like_count,
                                    replies: replies_of(topic),
                                    created: relative_time(&topic.created_at),
                                }
                                .boxed()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .boxed(),
            );
        }

        if !summary.top_categories.is_empty() {
            sections.push(section_title(theme, "Top categories"));
            sections.push(
                Column::new()
                    .gaps(LayoutSpacing::all(10))
                    .children(
                        summary
                            .top_categories
                            .iter()
                            .map(|category| {
                                category_row(
                                    theme,
                                    &category.color,
                                    &category.name,
                                    &format!(
                                        "{} topics · {} posts",
                                        format_count(category.topic_count),
                                        format_count(category.post_count),
                                    ),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                    .boxed(),
            );
        }

        let badges: Vec<AnyWidget> = summary
            .badges
            .iter()
            .filter_map(|granted| {
                let name = profile.badge_name(granted.badge_id)?;
                let label = if granted.count > 1 {
                    format!("{name} ×{}", granted.count)
                } else {
                    name.to_string()
                };
                Some(icon_label(theme, Icons::AWARD, &label, 16))
            })
            .collect();
        if !badges.is_empty() {
            sections.push(section_title(theme, "Badges"));
            sections.push(
                Column::new()
                    .gaps(LayoutSpacing::all(10))
                    .children(badges)
                    .boxed(),
            );
        }

        Container::new()
            .padding(LayoutSpacing::horizontal(24.into()))
            .child(
                Column::new()
                    .gaps(LayoutSpacing::all(20))
                    .children(sections),
            )
    }
}

/// The avatar with the name, handle, role and dates beside it.
fn header(theme: ForumTheme, profile: &UserProfile) -> AnyWidget {
    let detail = &profile.detail;
    let muted = theme.on_background_color().with_opacity(160);
    let display_name = detail
        .name
        .as_deref()
        .filter(|name| !name.is_empty())
        .unwrap_or(&detail.username);

    let mut details = vec![
        Text::new(display_name.to_string())
            .text_style(
                TextStyle::new()
                    .text_overflow(TextOverflow::Wrap)
                    .font_weight(FontWeight::Bold)
                    .font_size(26)
                    .color(theme.on_background_color()),
            )
            .boxed(),
        Text::new(format!("@{}", detail.username))
            .text_style(TextStyle::new().font_size(16).color(muted))
            .boxed(),
    ];
    if let Some(title) = detail.title.as_deref().filter(|title| !title.is_empty()) {
        details.push(
            Text::new(title.to_string())
                .text_style(TextStyle::new().font_size(14).color(muted))
                .boxed(),
        );
    }
    details.push(icon_label(theme, Icons::AWARD, &profile.role(), 14));
    details.push(icon_label(
        theme,
        Icons::CALENDAR,
        &format!("Joined {}", detail.created_at.get(..10).unwrap_or("—")),
        14,
    ));
    if let Some(seen) = detail.last_seen_at.as_deref() {
        details.push(icon_label(
            theme,
            Icons::EYE,
            &format!("Seen {}", relative_time(seen)),
            14,
        ));
    }

    Row::new()
        .vertical_alignment(BoxAlignment::Center)
        .gaps(LayoutSpacing::horizontal(16.into()))
        .children([
            avatar(theme, &detail.avatar_template, display_name),
            Expanded::new().box_child(Column::new().gaps(LayoutSpacing::all(4)).children(details)),
        ])
        .boxed()
}

fn avatar(theme: ForumTheme, template: &str, name: &str) -> AnyWidget {
    let round = BoxDecoration::new().border_radius(AVATAR_SIZE / 2.0);
    if template.is_empty() {
        let initial = name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default();
        return Container::new()
            .width(Dimension::Px(AVATAR_SIZE))
            .height(Dimension::Px(AVATAR_SIZE))
            .box_decoration(round.background_color(theme.primary_color()))
            .child(
                Align::new().alignment(Alignment::MidCenter).child(
                    Text::new(initial).text_style(
                        TextStyle::new()
                            .font_size(36)
                            .color(theme.on_primary_color()),
                    ),
                ),
            )
            .boxed();
    }
    Container::new()
        .width(Dimension::Px(AVATAR_SIZE))
        .height(Dimension::Px(AVATAR_SIZE))
        .box_decoration(round)
        .child(
            // `Contain`, not `Cover`: a square avatar fills the box either way,
            // and `Cover` would drop the container's rounded clip.
            NetworkImage::new(avatar_url(template, 192))
                .width(AVATAR_SIZE)
                .height(AVATAR_SIZE)
                .fit(BoxFit::Contain),
        )
        .boxed()
}

/// The activity figures: nine tiles in a 3×3 grid.
fn stats(theme: ForumTheme, profile: &UserProfile) -> AnyWidget {
    let summary = &profile.summary.user_summary;
    let tiles = [
        ("Days visited", format_count(summary.days_visited)),
        ("Read time", format_duration(summary.time_read)),
        ("Topics viewed", format_count(summary.topics_entered)),
        ("Posts read", format_count(summary.posts_read_count)),
        ("Likes given", format_count(summary.likes_given)),
        ("Likes received", format_count(summary.likes_received)),
        ("Topics", format_count(summary.topic_count)),
        ("Posts", format_count(summary.post_count)),
        ("Solutions", format_count(summary.solved_count)),
    ];

    Column::new()
        .gaps(LayoutSpacing::all(4))
        .children(
            tiles
                .chunks(3)
                .map(|pair| {
                    Row::new()
                        .gaps(LayoutSpacing::horizontal(10.into()))
                        .children(
                            pair.iter()
                                .map(|(label, value)| {
                                    Expanded::new().box_child(stat_tile(theme, label, value))
                                })
                                .collect::<Vec<_>>(),
                        )
                        .boxed()
                })
                .collect::<Vec<_>>(),
        )
        .boxed()
}

fn stat_tile(theme: ForumTheme, label: &str, value: &str) -> AnyWidget {
    Container::new()
        .padding(LayoutSpacing::all(10))
        .box_decoration(
            BoxDecoration::new()
                .background_color(theme.on_background_color().with_alpha(0.06))
                .border_radius(4),
        )
        .child(
            Column::new().gaps(LayoutSpacing::all(2)).children([
                Text::new(value.to_string())
                    .text_style(
                        TextStyle::new()
                            .font_weight(FontWeight::Bold)
                            .font_size(20)
                            .color(theme.on_background_color()),
                    )
                    .boxed(),
                Text::new(label.to_string())
                    .text_style(
                        TextStyle::new()
                            .text_overflow(TextOverflow::Wrap)
                            .font_size(12)
                            .color(theme.on_background_color().with_opacity(160)),
                    )
                    .boxed(),
            ]),
        )
        .boxed()
}

fn section_title(theme: ForumTheme, title: &str) -> AnyWidget {
    Container::new()
        .padding(LayoutSpacing::new().top(8))
        .box_child(
            Text::new(title.to_string()).text_style(
                TextStyle::new()
                    .font_weight(FontWeight::Bold)
                    .font_size(20)
                    .color(theme.on_background_color()),
            ),
        )
}

/// A category the user posts in: its color, name and how much they posted.
fn category_row(theme: ForumTheme, color: &str, name: &str, counts: &str) -> AnyWidget {
    Row::new()
        .vertical_alignment(BoxAlignment::Center)
        .gaps(LayoutSpacing::horizontal(10.into()))
        .children([
            Container::new()
                .width(Dimension::Px(12.0))
                .height(Dimension::Px(12.0))
                .box_decoration(
                    BoxDecoration::new()
                        .background_color(parse_hex_color(color))
                        .border_radius(6),
                )
                .child(ZeroSizedBox)
                .boxed(),
            Text::new(name.to_string())
                .text_style(
                    TextStyle::new()
                        .font_weight(FontWeight::Bold)
                        .font_size(16)
                        .color(theme.on_background_color()),
                )
                .boxed(),
            Expanded::new().box_child(SizedBox::new()),
            Text::new(counts.to_string())
                .text_style(
                    TextStyle::new()
                        .font_size(14)
                        .color(theme.on_background_color().with_opacity(160)),
                )
                .boxed(),
        ])
        .boxed()
}

/// A small icon with a line of text beside it.
fn icon_label(theme: ForumTheme, glyph: &'static StaticSvg, text: &str, size: u32) -> AnyWidget {
    let color = theme.on_background_color().with_opacity(180);
    Row::new()
        .vertical_alignment(BoxAlignment::Center)
        .gaps(LayoutSpacing::horizontal(8.into()))
        .children([
            Svg::new(glyph)
                .style("path", SvgStyle::new().stroke(color))
                .width(ICON_SIZE)
                .height(ICON_SIZE)
                .boxed(),
            Text::new(text.to_string())
                .text_style(TextStyle::new().font_size(size).color(color))
                .boxed(),
        ])
        .boxed()
}

/// How many replies a topic has: its posts after the opening one, never fewer
/// than none.
fn replies_of(topic: &SummaryTopic) -> i64 {
    topic.posts_count.saturating_sub(1).max(0)
}

/// One of the user's top topics; pressing it opens the post.
///
/// Its numbers read as they do on a topic card of a feed: an icon with a count
/// for the likes and for the replies, at the left. The card shows its views at
/// the right, which a profile's summary does not have, so the row shows when the
/// topic was started there instead.
#[derive(Clone, StatelessWidget)]
struct TopicRow {
    slug: ShareRef<str>,
    id: i64,
    title: String,
    likes: i64,
    replies: i64,
    created: String,
}

impl StatelessWidget for TopicRow {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);
        let navigator = NavigatorController::<AppRouter>::of(ctx);
        let slug = self.slug.clone();
        let id = self.id;

        Button::new()
            .decoration(divider_decor(theme))
            .hover_decoration(
                divider_decor(theme).background_color(theme.primary_color().with_opacity(10)),
            )
            .on_press(move || {
                navigator.push(AppRouter::PostDetail {
                    slug: slug.clone(),
                    id,
                });
            })
            .child(
                Container::new().padding(LayoutSpacing::vertical(14)).child(
                    Column::new().gaps(LayoutSpacing::all(4)).children([
                        Text::new(self.title.clone())
                            .text_style(
                                TextStyle::new()
                                    .text_overflow(TextOverflow::Wrap)
                                    .font_weight(FontWeight::Bold)
                                    .font_size(17)
                                    .color(theme.on_background_color()),
                            )
                            .boxed(),
                        Row::new()
                            .vertical_alignment(BoxAlignment::Center)
                            .gaps(LayoutSpacing::horizontal(20.into()))
                            .children([
                                count_label(theme, Icons::HEART, self.likes),
                                count_label(theme, Icons::MESSAGE_CIRCLE, self.replies),
                                Expanded::new().box_child(SizedBox::new()),
                                Text::new(self.created.clone())
                                    .text_style(
                                        TextStyle::new()
                                            .font_size(13)
                                            .color(theme.on_background_color().with_opacity(160)),
                                    )
                                    .boxed(),
                            ])
                            .boxed(),
                    ]),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::SummaryTopic;

    fn topic(posts_count: i64) -> SummaryTopic {
        SummaryTopic { posts_count, ..Default::default() }
    }

    #[test]
    fn replies_are_the_posts_after_the_opening_one() {
        assert_eq!(replies_of(&topic(5)), 4);
        assert_eq!(replies_of(&topic(2)), 1);
        assert_eq!(replies_of(&topic(1)), 0);
    }

    /// The app on the profile of `alice`, already loaded, with one top topic or
    /// with none.
    fn profile_app(with_top_topic: bool) -> aimer::HeadlessAimerApp<impl aimer::Widget + 'static> {
        use aimer::quiver::winit::dpi::PhysicalSize;
        use aimer::router::Navigator;
        use aimer::{AimerApp, HeadlessOptions, Provider};

        let mut store = AppStore::default();
        store.cache_profile(
            "alice",
            UserProfile {
                summary: UserSummaryRoot {
                    topics: with_top_topic
                        .then(|| SummaryTopic {
                            id: 7,
                            title: "A post worth reading".to_owned(),
                            slug: "a-post".to_owned(),
                            posts_count: 12,
                            like_count: 3400,
                            created_at: "2024-01-01T00:00:00Z".to_owned(),
                            ..Default::default()
                        })
                        .into_iter()
                        .collect(),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let mut app = AimerApp::start_headless_with(
            aimer::Provider::new()
                .create(move || store.clone())
                .child(Navigator::new(AppRouter::UserProfile { username: ShareRef::from_static("alice") }, |r| r.boxed())),
            HeadlessOptions { size: PhysicalSize::new(450, 900), scale_factor: 1.0 },
        );
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(20));
            app.pump_frames(3);
        }
        app
    }

    #[test]
    fn a_profile_that_is_still_loading_shows_the_skeleton() {
        use aimer::quiver::winit::dpi::PhysicalSize;
        use aimer::router::Navigator;
        use aimer::{AimerApp, HeadlessOptions, Provider};

        // Nothing is cached for this user, so the screen has to ask for the
        // profile. The first frames are before any answer could arrive.
        let store = AppStore::default();
        let mut app = AimerApp::start_headless_with(
            Provider::new()
                .create(move || store.clone())
                .child(Navigator::new(AppRouter::UserProfile { username: ShareRef::from_static("nobody") }, |r| r.boxed())),
            HeadlessOptions { size: PhysicalSize::new(450, 900), scale_factor: 1.0 },
        );
        app.pump_frames(3);
        let census = app.paint_source_census().expect("a mounted page has a root");

        // The shell around the profile paints about 120 nodes, and a line of
        // "Loading…" adds a few; the skeleton, with its avatar, tiles and rows, adds
        // about 170 (293 in all when this was measured).
        assert!(census.local_v2 >= 200, "only {} nodes paint", census.local_v2);
    }

    #[test]
    fn a_profile_with_a_top_topic_paints_it_through_the_retained_path() {
        let without = profile_app(false).paint_source_census().expect("a mounted page has a root");
        let app = profile_app(true);
        let census = app.paint_source_census().expect("a mounted page has a root");

        // The row (its title, the two icons and their counts, the date) is on
        // the page: it adds nodes that paint, and none that is dropped.
        assert!(census.local_v2 > without.local_v2, "{} vs {}", census.local_v2, without.local_v2);
        assert!(census.drawn_unmapped.is_empty(), "unmapped {:?}", census.drawn_unmapped);
        assert!(census.invalid_bounds.is_empty(), "invalid bounds {:?}", census.invalid_bounds);
        assert!(census.sync_error.is_none(), "{:?}", census.sync_error);
        assert!(census.declined_paint.len() <= without.declined_paint.len(),
            "the row declined to paint: {:?}", census.declined_paint);
    }

    #[test]
    fn a_topic_with_no_posts_has_no_replies_rather_than_a_negative_count() {
        assert_eq!(replies_of(&topic(0)), 0);
        assert_eq!(replies_of(&topic(-3)), 0);
    }
}
