use std::sync::{Arc, RwLock};

use launcher::instances::SimpleInstance;
use log::*;
use reqwest::Client;
use rfd::AsyncFileDialog;
use simple_logger::SimpleLogger;
use slint::{spawn_local, Model, ModelRc, PlatformError, VecModel};
use clone_macro::clone;
use tokio::runtime::Runtime;

use crate::{app::{settings::AppSettings, slint_utils::SlintOption}, launcher::{instances, java::{get_java_version, JavaDetails}, launching::mc_structs::{MCSimpleVersion, MCVersionDetails, MCVersionList}}};

slint::include_modules!();
pub use slint_generatedMainWindow::*;

pub mod app;
pub mod ui;
pub mod launcher;

fn main() {
    println!("Initializing YetaLauncher...");
    SimpleLogger::new()
    .with_level(log::LevelFilter::Debug)
    .env()
    .init()
    .unwrap_or_else(|err| eprintln!("Failed to initialize logger: {err}"));

    let app = YetaLauncher::new();

    app.run().expect("YetaLauncher failed to start!");

}

#[derive(Debug)]
pub struct YetaLauncher {
    settings: AppSettings,
    instances: Option<Vec<SimpleInstance>>
}

unsafe impl Send for YetaLauncher {

}

impl YetaLauncher {
    fn run(self) -> Result<(), PlatformError> {
        let window = Arc::new(MainWindow::new()?);
        let app = Arc::new(RwLock::new(self));
        let runtime = Runtime::new().unwrap();
        let rt = runtime.handle().clone();

        let settings = window.global::<Settings>();

        settings.set_settings(app.read().unwrap().settings.to_slint());

        settings.on_update_instance_path(clone!([window, app, rt], move || {
            spawn_local(clone!([window, app, rt], async move {
                let _guard = rt.enter();
                debug!("Opening folder picker...");
                if let Some(folder) = AsyncFileDialog::new().set_title("Select Instance folder").pick_folder().await {
                    let mut app = app.write().unwrap();
    
                    app.settings.instance_path = Some(folder.path().to_str().expect("Failed to convert folder path to valid UTF-8!").to_string());
                    app.settings.set();
                    app.sync_settings(&window);
                }
            })).unwrap();
        }));

        settings.on_update_icon_path(clone!([window, app, rt], move || {
            spawn_local(clone!([window, app, rt], async move {
                let _guard = rt.enter();
                debug!("Opening folder picker...");
                
                if let Some(folder) = AsyncFileDialog::new().set_title("Select Icon folder").pick_folder().await {
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

        settings.on_update_java_path(clone!([rt], move || {
            let picker = rt.block_on(async move {
                AsyncFileDialog::new().set_title("Select Java binary").pick_file().await
            });
            
            if let Some(file) = picker {
                SlintOption::Some(file.path().to_str().expect("Failed to convert file path to valid UTF-8!").to_string()).into()
            } else {
                SlintOption::None::<&str>.into()
            }
        }));

        settings.on_test_java(clone!([], move |path, args| {
            let result = get_java_version(path.to_string(), args.to_string());

            match result {
                Ok(ver) => ModelRc::new(
                    VecModel::from(vec![
                        ver.replace('"', "").split_whitespace().nth(2).unwrap_or("Could not detect").into()
                    ])
                ),
                Err(_) => SlintOption::<String>::None.into()
            }
        }));


        settings.on_get_mc_versions(clone!([window, rt], move || {
            spawn_local(clone!([window, rt], async move {
                let _guard = rt.enter();
                let client = Client::new();

                if let Some(list) = MCVersionList::get(&client).await {
                    window.global::<Settings>().set_version_list(
                        ModelRc::new(
                            VecModel::from(
                                list.versions
                                .into_iter()
                                .map(MCVersionDetails::to_simple)
                                .map(|versions: MCSimpleVersion| MCSimpleVersion::to_slint(&versions))
                                .collect::<Vec<SlMCVersionDetails>>()
                            )
                        )
                    );
                }
            })).unwrap();
        }));

        settings.on_get_instances(clone!([window, app, rt], move || {
            spawn_local(clone!([window, app, rt], async move {
                let _guard = rt.enter();
                let mut app = app.write().unwrap();

                if app.instances.is_none() {
                    let instances = instances::get_instances(Arc::new(app.settings.clone())).await;

                    match instances {
                        Ok(inst) => app.instances = Some(inst),
                        Err(err) => error!("Failed to gather instances: {err}")
                    }
                }

                window.global::<Settings>().set_instances(match &app.instances {
                    Some(inst) => ModelRc::new(
                        VecModel::from(
                            inst.into_iter()
                            .map(SimpleInstance::to_slint)
                            .collect::<Vec<SlSimpleInstance>>()
                        )
                    ),
                    None => ModelRc::default(),
                })
                
            })).unwrap();
        }));


        info!("Starting...");
        window.run()?;
        Ok(())
    }

    fn new() -> Self {
        Self {
            settings: AppSettings::get(),
            instances: None
        }
    }

    fn sync_settings(&self, window: &Arc<MainWindow>) {
        window.global::<Settings>().set_settings(self.settings.to_slint());
    }
}