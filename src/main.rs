use log::*;
use simple_logger::SimpleLogger;
use slint::PlatformError;

slint::include_modules!();

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

    YetaLauncher::run().await.expect("YetaLauncher failed to start");
}

pub struct YetaLauncher;

impl YetaLauncher {
    async fn run() -> Result<(), PlatformError> {
        let window = MainWindow::new()?;


        info!("Starting...");
        window.run()?;
        Ok(())
    }
}