use iced::widget::container;


pub struct YetaTheme;

impl container::StyleSheet for YetaTheme {
    type Style = ();

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: todo!(),
            background: todo!(),
            border_radius: todo!(),
            border_width: todo!(),
            border_color: todo!(),
        }
    }
}