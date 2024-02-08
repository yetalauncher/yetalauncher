use std::sync::{Arc, RwLock};

use log::*;
use rfd::AsyncFileDialog;
use simple_logger::SimpleLogger;
use slint::{spawn_local, Model, PlatformError};

use crate::{app::settings::AppSettings, launcher::java::JavaDetails};

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
        let window = Arc::new(MainWindow::new()?);
        let app_ref = Arc::new(RwLock::new(self));

        let settings = window.global::<Settings>();

        settings.set_settings(app_ref.read().unwrap().settings.to_slint());

        let (window2, app2) = (window.clone(), app_ref.clone());
        settings.on_update_instance_path(move || {
            let (window2, app2) = (window2.clone(), app2.clone());
            spawn_local(async move {
                debug!("Opening folder picker...");
                
                if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
                    let mut app = app2.write().unwrap();
    
                    app.settings.instance_path = Some(folder.path().to_str().expect("Failed to convert folder path to valid UTF-8!").to_string());
                    app.settings.set();
                    window2.global::<Settings>().set_settings(app.settings.to_slint());
                }
            }).unwrap();
        });

        let (window2, app2) = (window.clone(), app_ref.clone());
        settings.on_update_icon_path(move || {
            let (window2, app2) = (window2.clone(), app2.clone());
            spawn_local(async move {
                debug!("Opening folder picker...");
                
                if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
                    let mut app = app2.write().unwrap();
    
                    app.settings.icon_path = Some(folder.path().to_str().expect("Failed to convert folder path to valid UTF-8!").to_string());
                    app.settings.set();
                    window2.global::<Settings>().set_settings(app.settings.to_slint());
                }
            }).unwrap();
        });

        let (window2, app2) = (window.clone(), app_ref.clone());
        settings.on_add_java_setting(move || {
            let mut app = app2.write().unwrap();
            app.settings.java_settings.push(JavaDetails::default());
            window2.global::<Settings>().set_settings(
                app.settings.to_slint()
            );
        });

        let (window2, app2) = (window.clone(), app_ref.clone());
        settings.on_save_settings(move || {
            let mut app = app2.write().unwrap();
            let new_settings = window2.global::<Settings>().get_settings();

            app.settings.java_settings = new_settings.java_settings
            .iter()
            .map(|java_setting| JavaDetails::from_slint(java_setting))
            .collect();

            app.settings.set();
        });

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