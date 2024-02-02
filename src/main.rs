use log::*;
use simple_logger::SimpleLogger;
use slint::PlatformError;

use crate::app::settings::AppSettings;

slint::include_modules!();
pub use slint_generatedMainWindow::*;

pub mod app;
pub mod ui;
pub mod launcher;

#[tokio::main]
async fn main() {
    println!("Initializing YetaLauncher...");
    SimpleLogger::new()
    .with_level(log::LevelFilter::Debug)
    .env()
    .init()
    .unwrap_or_else(|err| eprintln!("Failed to initialize logger: {err}"));

    let app = YetaLauncher::new();

    app.run().await.expect("YetaLauncher failed to start");
}

pub struct YetaLauncher {
    settings: AppSettings
}

impl YetaLauncher {
    async fn run(&self) -> Result<(), PlatformError> {
        let window = MainWindow::new()?;

        window.global::<Settings>().set_settings(self.settings.to_slint());

        info!("Starting...");
        window.run()?;
        Ok(())
    }

    fn new() -> Self {
        Self {
            settings: AppSettings::get()
        }
    }
}