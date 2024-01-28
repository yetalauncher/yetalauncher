use iced::{widget::text, Element};

use crate::{ui::theme::YetaTheme, Message, YetaLauncher};

pub struct HomePage;

impl HomePage {
    pub fn draw<'a>(_: &YetaLauncher) -> Element<'a, Message> {
        text("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz").font(YetaTheme::FONT_MAIN).into()
    }
}