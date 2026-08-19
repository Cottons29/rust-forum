use crate::components::page_title::PageTitle;
use crate::components::theme_selector::ThemeSelector;
use crate::theme::ForumTheme;
use aimer::style::{LayoutSpacing, TextStyle, Theme};
use aimer::{
    BuildContext, Column, Container, FontWeight, SizedBox, StatelessWidget, Text, Widget,
};

/// A minimal settings screen. For now it hosts the theme selector that used to
/// live at the top of the sidebar.
#[derive(Clone, StatelessWidget)]
pub struct SettingsScreen;

impl StatelessWidget for SettingsScreen {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = *ForumTheme::of(ctx);

        Container::new()
            .color(theme.background_color())
            .padding(LayoutSpacing::all(24))
            .child(
                Column::new().children([
                    PageTitle::new("Settings".into())
                        .padding(LayoutSpacing::new())
                        .boxed(),
                    SizedBox::new().height(24).boxed(),
                    Text::new("Theme")
                        .text_style(
                            TextStyle::new()
                                .font_weight(FontWeight::Bold)
                                .font_size(20)
                                .color(theme.on_background_color()),
                        )
                        .boxed(),
                    SizedBox::new().height(12).boxed(),
                    ThemeSelector.boxed(),
                ]),
            )
    }
}
