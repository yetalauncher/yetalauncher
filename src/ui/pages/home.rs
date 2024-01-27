use iced::{widget::text, Element};

use crate::{Message, YetaLauncher};

pub struct HomePage;

impl HomePage {
    pub fn draw<'a>(_: &YetaLauncher) -> Element<'a, Message> {
        text("Homeeeeee").into()
    }
}