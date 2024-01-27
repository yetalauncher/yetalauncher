use iced::{widget::text, Element};

use crate::{Message, YetaLauncher};

pub struct AccountPage;

impl AccountPage {
    pub fn draw<'a>(_: &YetaLauncher) -> Element<'a, Message> {
        text("Accounts").into()
    }
}