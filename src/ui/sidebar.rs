use iced::widget::{column, Column, button};

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
    pub fn draw<'a>() -> Column<'a, Message> {
        column![
            button("Home").style(YetaTheme::button()).on_press(Message::SidebarMessage(SidebarMessage::HomeButton)),
            button("Create").style(YetaTheme::button()).on_press(Message::SidebarMessage(SidebarMessage::CreateButton)),
            button("Accounts").style(YetaTheme::button()).on_press(Message::SidebarMessage(SidebarMessage::AccountsButton)),
            button("Settings").style(YetaTheme::button()).on_press(Message::SidebarMessage(SidebarMessage::SettingsButton))
        ].into()
    }
}