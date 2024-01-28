use iced::{widget::row, Sandbox, Settings};
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
        ..Default::default()
    };

    YetaLauncher::run(launch_settings).expect("YetaLauncher failed to start");
}

pub struct YetaLauncher {
    page: Pages
}

#[derive(Debug, Clone)]
pub enum Message {
    SidebarMessage(SidebarMessage)
}

impl Sandbox for YetaLauncher {
    type Message = Message;

    fn new() -> Self {
        Self {
            page: Pages::Home
        }
    }

    fn title(&self) -> String {
        String::from("YetaLauncher")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::SidebarMessage(message) => Pages::switch_page(self, message)
        };
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