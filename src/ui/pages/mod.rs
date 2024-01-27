use iced::Element;

use crate::{Message, YetaLauncher};

use self::{accounts::AccountPage, create::CreatePage, home::HomePage, settings::SettingsPage};

use super::sidebar::SidebarMessage;

pub mod home;
pub mod create;
pub mod accounts;
pub mod settings;

pub enum Pages {
    Home,
    Create,
    Accounts,
    Settings
}

impl Pages {
    pub fn draw<'a>(app: &YetaLauncher) -> Element<'a, Message> {
        match app.page {
            Pages::Home => HomePage::draw(app),
            Pages::Create => CreatePage::draw(app),
            Pages::Accounts => AccountPage::draw(app),
            Pages::Settings => SettingsPage::draw(app),
        }
    }

    pub fn switch_page(app: &mut YetaLauncher, message: SidebarMessage) {
        match message {
            SidebarMessage::HomeButton => app.page = Pages::Home,
            SidebarMessage::CreateButton => app.page = Pages::Create,
            SidebarMessage::AccountsButton => app.page = Pages::Accounts,
            SidebarMessage::SettingsButton => app.page = Pages::Settings,
        }
    }
}