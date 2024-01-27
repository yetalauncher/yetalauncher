use iced::widget::{column, Column, button};

use crate::Message;


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
            button("Home").on_press(Message::SidebarMessage(SidebarMessage::HomeButton)),
            button("Create").on_press(Message::SidebarMessage(SidebarMessage::CreateButton)),
            button("Accounts").on_press(Message::SidebarMessage(SidebarMessage::AccountsButton)),
            button("Settings").on_press(Message::SidebarMessage(SidebarMessage::SettingsButton))
        ].into()
    }
}