use iced::{widget::text, Element};

use crate::{Message, YetaLauncher};

pub struct CreatePage;

impl CreatePage {
    pub fn draw<'a>(_: &YetaLauncher) -> Element<'a, Message> {
        text("Create Instance").into()
    }
}