use iced::{color, theme, widget::button, Background, BorderRadius, Color};


#[derive(Debug, Clone, Copy)]
pub struct YetaTheme {
    background_primary: Color,
    text: Color,
    main: Color,
    border_radius: BorderRadius
}


#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonTheme {
    #[default]
    Default,
    Sidebar
}

impl YetaTheme {
    pub fn palette(&self) -> theme::Palette {
        theme::Palette {
            background: self.background_primary,
            text: self.text,
            primary: self.main,
            success: color!(100, 255, 100),
            danger: color!(255, 100, 100)
        }
    }

    pub fn button() -> theme::Button {
        theme::Button::custom(YetaTheme::default())
    }
}

impl Default for YetaTheme {
    fn default() -> Self {
        Self {
            background_primary: color!(0x060606),
            text: color!(0xEEEEEE),
            main: color!(160, 30, 212),
            border_radius: 6.0.into()
        }
    }
}

impl button::StyleSheet for YetaTheme {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            text_color: self.text,
            border_radius: self.border_radius,
            ..Default::default()
        }
    }

    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            text_color: self.text,
            border_radius: self.border_radius,
            background: Some(Background::Color(self.main)),
            ..Default::default()
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        Self::hovered(self, style)
    }
}
