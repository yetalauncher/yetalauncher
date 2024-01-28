use iced::{alignment, font::Weight, widget::{self, button, column, container, horizontal_space, row, svg, svg::Handle, Button}, Background, Element, Length, Theme};

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
    pub fn draw<'a>() -> Element<'a, Message, iced::Renderer> {
        let col = column![
            Self::button("Home", "src/resources/tabler-icons/home.svg", SidebarMessage::HomeButton),
            Self::button("Create", "src/resources/tabler-icons/plus.svg", SidebarMessage::CreateButton),
            Self::button("Accounts", "src/resources/tabler-icons/user.svg", SidebarMessage::AccountsButton),
            Self::button("Settings", "src/resources/tabler-icons/settings.svg", SidebarMessage::SettingsButton),
        ];

        container(col)
        .padding(4)
        .height(Length::Fill)
        .width(Length::Fixed(52.0))
        .style(YetaTheme::sidebar())
        .into()
    }

    fn button<'a>(text: &'a str, icon_path: &'a str, message: SidebarMessage) -> Button<'a, Message> {
        button(row![
            svg(Handle::from_path(icon_path)).style(YetaTheme::svg()).height(32).width(32),
            horizontal_space(4),
            widget::text(text).size(16).font(YetaTheme::alt_font(Weight::Semibold)).horizontal_alignment(alignment::Horizontal::Right)
        ])
        .padding(6)
        .style(YetaTheme::sidebar_button())
        .width(Length::Fill)
        .on_press(Message::SidebarMessage(message))
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