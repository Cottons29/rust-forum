use aimer::style::{BorderSlice, BorderStyle, BoxBorder, BoxDecoration};
use aimer::{Color, Theme};

/// Tomato-based app theme.
///
/// Derives `Theme` so [`aimer::style::AnimatedTheme`] can interpolate between
/// the light and dark variants. Channels are stored as primitives (`u8`)
/// because the derive requires every field to implement `Animatable`.
#[derive(Clone, Copy, Debug, PartialEq, Theme)]
pub struct ForumTheme {
    background_red: u8,
    background_green: u8,
    background_blue: u8,
    surface_red: u8,
    surface_green: u8,
    surface_blue: u8,
    foreground_tone: u8,
    accent_red: u8,
    accent_green: u8,
    accent_blue: u8,
    pub font_color: Color,
}

impl ForumTheme {
    pub fn light() -> Self {
        Self {
            background_red: 255,
            background_green: 248,
            background_blue: 246,
            surface_red: 255,
            surface_green: 255,
            surface_blue: 255,
            foreground_tone: 30,
            accent_red: 255,
            accent_green: 99,
            accent_blue: 71,
            font_color: Color::BLACK,
        }
    }

    pub fn dark() -> Self {
        Self {
            background_red: 27,
            background_green: 18,
            background_blue: 16,
            surface_red: 38,
            surface_green: 26,
            surface_blue: 23,
            foreground_tone: 235,
            accent_red: 255,
            accent_green: 122,
            accent_blue: 92,
            font_color: Color::WHITE,
        }
    }

    pub fn background_color(&self) -> Color {
        Color::Rgb(
            self.background_red,
            self.background_green,
            self.background_blue,
        )
    }

    pub fn on_background_color(&self) -> Color {
        Color::Gray8(self.foreground_tone)
    }

    pub fn surface_color(&self) -> Color {
        Color::Rgb(self.surface_red, self.surface_green, self.surface_blue)
    }

    pub fn on_surface_color(&self) -> Color {
        Color::Gray8(self.foreground_tone)
    }

    pub fn primary_color(&self) -> Color {
        Color::Rgb(self.accent_red, self.accent_green, self.accent_blue)
    }

    pub fn box_decor(&self) -> BoxDecoration {
        BoxDecoration::new()
            .background_color(self.surface_color())
            .border_radius(8)
            .border(BoxBorder::all(
                BorderSlice::new()
                    .style(BorderStyle::Dotted)
                    .stroke(2)
                    .color(self.primary_color().darken(0.4).with_alpha(0.1)),
            ))
    }

    pub fn box_decor_with_color(&self, color: impl Into<Color>) -> BoxDecoration {
        BoxDecoration::new()
            .background_color(self.surface_color())
            .border_radius(8)
            .border(BoxBorder::all(
                BorderSlice::new()
                    .style(BorderStyle::Dotted)
                    .stroke(2)
                    .color(color.into()),
            ))
    }

    pub fn hover_box_decor(&self) -> BoxDecoration {
        self.box_decor()
            .clone()
            .background_color(self.primary_color().with_opacity(30))
            .border(BoxBorder::all(
                BorderSlice::new()
                    .style(BorderStyle::Solid)
                    .stroke(2)
                    .color(self.primary_color()),
            ))
    }

    pub fn on_primary_color(&self) -> Color {
        Color::WHITE
    }
}
