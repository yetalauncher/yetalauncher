use iced::{color, theme, BorderRadius, Color};

use super::sidebar::SidebarTheme;


#[derive(Debug, Clone, Copy)]
pub struct YetaTheme {
    pub background_primary: Color,
    pub background_secondary: Color,
    pub background_tertiary: Color,
    pub text: Color,
    pub main: Color,
    pub border_radius: BorderRadius
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

    pub fn sidebar_button() -> theme::Button {
        theme::Button::custom(SidebarTheme)
    }

    pub fn sidebar() -> theme::Container {
        theme::Container::Custom(Box::new(SidebarTheme))
    }
}

impl Default for YetaTheme {
    fn default() -> Self {
        Self {
            background_primary: color!(0x060606),
            background_secondary: color!(0x111214),
            background_tertiary: color!(0x171718),
            text: color!(0xEEEEEE),
            main: color!(160, 30, 212),
            border_radius: 5.0.into()
        }
    }
}