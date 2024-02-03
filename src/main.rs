use std::sync::{Arc, RwLock};

use log::*;
use rfd::AsyncFileDialog;
use simple_logger::SimpleLogger;
use slint::{spawn_local, PlatformError};

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

    app.run().await.expect("YetaLauncher failed to start!");
}

#[derive(Debug)]
pub struct YetaLauncher {
    settings: AppSettings
}

impl YetaLauncher {
    async fn run(self) -> Result<(), PlatformError> {
        let window = MainWindow::new()?;
        let window_ref = Arc::new(window);
        let app_ref = Arc::new(RwLock::new(self));

        window_ref.global::<Settings>().set_settings(app_ref.read().unwrap().settings.to_slint());

        let (window2, app2) = (window_ref.clone(), app_ref.clone());
        window_ref.global::<Settings>().on_update_instance_path(move || {
            let (window2, app2) = (window2.clone(), app2.clone());
            spawn_local(async move {
                info!("Opening folder picker...");
                
                if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
                    let mut app = app2.write().unwrap();
    
                    app.settings.instance_path = Some(folder.path().to_str().expect("Failed to convert folder path to valid UTF-8!").to_string());
                    app.settings.set();
                    window2.global::<Settings>().set_settings(app.settings.to_slint());
                }
            }).unwrap();
        });

        info!("Starting...");
        window_ref.run()?;
        Ok(())
    }

    fn new() -> Self {
        Self {
            settings: AppSettings::get()
        }
    }
}