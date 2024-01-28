use std::borrow::Cow;

use iced::{executor, font, widget::row, Application, Command, Settings};
use log::*;
use simple_logger::SimpleLogger;
use ui::{pages::Pages, sidebar::{Sidebar, SidebarMessage}, theme::YetaTheme};

pub mod ui;


fn main() {
    SimpleLogger::new()
    .with_level(log::LevelFilter::Warn)
    .with_module_level("yetalauncher", log::LevelFilter::Debug)
    .env()
    .init()
    .unwrap_or_else(|err| eprintln!("Failed to initialize logger: {err}"));

    info!("Starting YetaLauncher...");

    let launch_settings = Settings {
        window: iced::window::Settings {
            size: (1000, 600),
            ..Default::default()
        },
        default_font: YetaTheme::font(font::Weight::Light),
        ..Default::default()
    };

    YetaLauncher::run(launch_settings).expect("YetaLauncher failed to start");
}

pub struct YetaLauncher {
    page: Pages
}

#[derive(Debug, Clone)]
pub enum Message {
    SidebarMessage(SidebarMessage),
    FontLoaded(Result<(), font::Error>)
}

impl Application for YetaLauncher {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (YetaLauncher, iced::Command<Message>) {
        (
            Self {
                page: Pages::Home
            },
            Command::batch([
                font::load(Cow::from(YetaTheme::NUNITO_BYTES)).map(Message::FontLoaded),
            ])
        )
    }

    fn title(&self) -> String {
        String::from("YetaLauncher")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::SidebarMessage(message) => Pages::switch_page(self, message),
            Message::FontLoaded(result) => if let Err(err) = result {
                warn!("Failed to load font: {err:?}")
            } else { debug!("Font loaded") }
        };
        Command::none()
    }

    fn view(&self) -> iced::Element<Message>{
        row![
            Sidebar::draw(),
            Pages::draw(self)
        ].into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::custom(YetaTheme::default().palette())
    }
}