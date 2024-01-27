use iced::{widget::text, Element};

use crate::{Message, YetaLauncher};

pub struct SettingsPage;

impl SettingsPage {
    pub fn draw<'a>(_: &YetaLauncher) -> Element<'a, Message> {
        text("Settings").into()
    }
}