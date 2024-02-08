use std::sync::{Arc, RwLock};

use log::*;
use rfd::AsyncFileDialog;
use simple_logger::SimpleLogger;
use slint::{spawn_local, Model, PlatformError};
use clone_macro::clone;

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
        let app = Arc::new(RwLock::new(self));

        let settings = window.global::<Settings>();

        settings.set_settings(app.read().unwrap().settings.to_slint());

        settings.on_update_instance_path(clone!([window, app], move || {
            spawn_local(clone!([window, app], async move {
                debug!("Opening folder picker...");
                
                if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
                    let mut app = app.write().unwrap();
    
                    app.settings.instance_path = Some(folder.path().to_str().expect("Failed to convert folder path to valid UTF-8!").to_string());
                    app.settings.set();
                    app.sync_settings(&window);
                }
            })).unwrap();
        }));

        settings.on_update_icon_path(clone!([window, app], move || {
            spawn_local(clone!([window, app], async move {
                debug!("Opening folder picker...");
                
                if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
                    let mut app = app.write().unwrap();
    
                    app.settings.icon_path = Some(folder.path().to_str().expect("Failed to convert folder path to valid UTF-8!").to_string());
                    app.settings.set();
                    app.sync_settings(&window);
                }
            })).unwrap();
        }));

        settings.on_add_java_setting(clone!([window, app], move || {
            let mut app = app.write().unwrap();
            app.settings.java_settings.push(JavaDetails::default());
            app.sync_settings(&window);
        }));

        settings.on_save_settings(clone!([window, app], move || {
            let mut app = app.write().unwrap();
            let new_settings = window.global::<Settings>().get_settings();

            app.settings.java_settings = new_settings.java_settings
            .iter()
            .map(|java_setting| JavaDetails::from_slint(java_setting))
            .collect();

            app.settings.set();
        }));

        info!("Starting...");
        window.run()?;
        Ok(())
    }

    fn new() -> Self {
        Self {
            settings: AppSettings::get()
        }
    }

    fn sync_settings(&self, window: &Arc<MainWindow>) {
        window.global::<Settings>().set_settings(self.settings.to_slint());
    }
}