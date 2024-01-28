use iced::{widget::{button, column, container}, Background, Element, Length, Theme};

use crate::Message;

use super::theme::YetaTheme;


pub struct Sidebar;

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    HomeButton,
    CreateButton,
    AccountsButton,
    SettingsButton
}

impl Sidebar {
    pub fn draw<'a>() -> Element<'a, Message> {
        let col = column![
            button("Home").style(YetaTheme::sidebar_button()).on_press(Message::SidebarMessage(SidebarMessage::HomeButton)),
            button("Create").style(YetaTheme::sidebar_button()).on_press(Message::SidebarMessage(SidebarMessage::CreateButton)),
            button("Accounts").style(YetaTheme::sidebar_button()).on_press(Message::SidebarMessage(SidebarMessage::AccountsButton)),
            button("Settings").style(YetaTheme::sidebar_button()).on_press(Message::SidebarMessage(SidebarMessage::SettingsButton))
        ];

        container(col).height(Length::Fill).style(YetaTheme::sidebar()).into()
    }
}

pub struct SidebarTheme;

impl container::StyleSheet for SidebarTheme {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        let theme = YetaTheme::default();
        container::Appearance {
            background: Some(Background::Color(theme.background_secondary)),
            border_color: theme.background_tertiary,
            border_width: 1.0,
            ..Default::default()
        }
    }
}

impl button::StyleSheet for SidebarTheme {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        let theme = YetaTheme::default();
        button::Appearance {
            text_color: theme.text,
            border_radius: theme.border_radius,
            ..Default::default()
        }
    }

    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        let theme = YetaTheme::default();
        button::Appearance {
            text_color: theme.text,
            border_radius: theme.border_radius,
            background: Some(Background::Color(theme.main)),
            ..Default::default()
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        Self::hovered(self, style)
    }
}