use crate::theme::ForumTheme;
use aimer::style::{LayoutSpacing, TextStyle, Theme};
use aimer::{BuildContext, Container, FontWeight, StatelessWidget, Text, Widget};

#[derive(StatelessWidget)]
pub struct PageTitle {
    title: String,
    padding: LayoutSpacing,
}

impl PageTitle {
    pub fn new(title: String) -> Self {
        Self {
            title,
            padding: LayoutSpacing::all(24).bottom(0),
        }
    }

    pub fn padding(mut self, padding: LayoutSpacing) -> Self {
        self.padding = padding;
        self
    }
}

impl StatelessWidget for PageTitle {
    fn build(&self, ctx: &BuildContext) -> impl Widget {
        let theme = ForumTheme::of(ctx);
        Container::new()
            .padding(self.padding)
            .child(
                Text::new(self.title.clone()).text_style(
                    TextStyle::new()
                        .font_weight(FontWeight::Value(1500))
                        .font_size(38)
                        .color(theme.on_background_color()),
                ),
            )
    }
}
