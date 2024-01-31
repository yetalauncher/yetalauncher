use app::settings::AppSettings;
use launcher::instances::SimpleInstance;
use log::*;
use simple_logger::SimpleLogger;
use slint::PlatformError;
use ui::pages::Pages;

slint::include_modules!();

pub mod app;
pub mod ui;
pub mod launcher;

#[tokio::main]
async fn main() {
    println!("Starting YetaLauncher...");
    SimpleLogger::new()
    .with_level(log::LevelFilter::Warn)
    .with_module_level("yetalauncher", log::LevelFilter::Debug)
    .env()
    .init()
    .unwrap_or_else(|err| eprintln!("Failed to initialize logger: {err}"));

    YetaLauncher::run().await.expect("YetaLauncher failed to start");
}

pub struct YetaLauncher {
    page: Pages,
    settings: AppSettings,
    instances: Option<Vec<SimpleInstance>>
}

impl YetaLauncher {
    async fn run() -> Result<(), PlatformError> {
        let window = MainWindow::new()?;

        let app = Self {
            page: Pages::Home,
            settings: AppSettings::get(),
            instances: None,
        };

        window.run()?;
        
        Ok(())
    }
}