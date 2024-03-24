use std::{sync::{Arc, RwLock}, time::Instant};

use app::notifier::InternalNotifier;
use launcher::{authentication::auth_structs::Accounts, instances::SimpleInstance};
use log::*;
use reqwest::Client;
use rfd::AsyncFileDialog;
use simple_logger::SimpleLogger;
use slint::{invoke_from_event_loop, spawn_local, Model, ModelRc, PlatformError, VecModel, Weak};
use clone_macro::clone;
use tokio::{runtime::Runtime, sync::mpsc};
use tokio_util::sync::CancellationToken;

use crate::{app::{settings::AppSettings, slint_utils::SlintOption}, launcher::{authentication::add_account, instances, java::{get_java_version, JavaDetails}, launching::mc_structs::{MCSimpleVersion, MCVersionDetails, MCVersionList}}};

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

    YetaLauncher::start().expect("Failed to start YetaLauncher!");

    info!("Exiting...");
}

#[derive(Debug)]
pub struct YetaLauncher {
    settings: RwLock<AppSettings>,
    accounts: RwLock<Accounts>,
    instances: RwLock<Option<Vec<SimpleInstance>>>
}

impl YetaLauncher {
    pub fn start() -> Result<(), PlatformError> {
        let time = Instant::now();
        let window = MainWindow::new()?;
        window.show()?;

        debug!("Loading (at {:?})...", Instant::now() - time);
        let app = YetaLauncher::new();
        app.run(window, time)?;

        Ok(())
    }

    fn run(self, window: MainWindow, time: Instant) -> Result<(), PlatformError> {
        let app = Arc::new(self);
        let runtime = Runtime::new().unwrap();
        let rt = runtime.handle().clone();

        let mut int_notifier = InternalNotifier::new();
        let notifier = int_notifier.make_notifier();
        let cancel_token = CancellationToken::new();

        let settings = window.global::<Settings>();

        settings.set_settings(app.settings.read().unwrap().to_slint());


        rt.spawn(clone!([cancel_token, { window.as_weak() } as window], async move {
            int_notifier.subscribe(cancel_token, clone!([window], move |notifications| {
                let slint_notifs: Vec<SlNotif> = notifications.iter().map(
                    |notif| notif.to_slint()
                ).rev().collect();

                slint::invoke_from_event_loop(clone!([window], move || {
                    //trace!("Syncing {} notification(s)", slint_notifs.len());

                    window.unwrap()
                    .global::<App>()
                    .set_notifications(
                        ModelRc::new(VecModel::from(slint_notifs))
                    );
                })).unwrap();
            })).await;
        }));



        settings.on_update_instance_path(clone!([{ window.as_weak() } as window, app, rt], move || {
            let _guard = rt.enter();
            rt.spawn(clone!([window, app], async move {
                debug!("Opening folder picker...");
                if let Some(folder) = AsyncFileDialog::new().set_title("Select Instance folder").pick_folder().await {
                    let mut settings = app.settings.write().unwrap();
    
                    settings.instance_path = Some(folder.path().to_str().expect("Failed to convert folder path to valid UTF-8!").to_string());
                    settings.set();
                }
                invoke_from_event_loop(move || {
                    app.sync_settings(window);
                }).unwrap();
            }));
        }));

        settings.on_update_icon_path(clone!([{ window.as_weak() } as window, app, rt], move || {
            let _guard = rt.enter();
            rt.spawn(clone!([window, app], async move {
                debug!("Opening folder picker...");
                
                if let Some(folder) = AsyncFileDialog::new().set_title("Select Icon folder").pick_folder().await {
                    let mut settings = app.settings.write().unwrap();
    
                    settings.icon_path = Some(folder.path().to_str().expect("Failed to convert folder path to valid UTF-8!").to_string());
                    settings.set();
                }
                invoke_from_event_loop(move || {
                    app.sync_settings(window);
                }).unwrap();
            }));
        }));

        settings.on_update_instance_size(clone!([{ window.as_weak() } as window, app], move |new_size| {
            let mut settings = app.settings.write().unwrap();
            settings.instance_size = new_size as u16;
            settings.set();
            app.sync_settings(window.clone());
        }));

        settings.on_add_java_setting(clone!([{ window.as_weak() } as window, app], move || {
            let mut settings = app.settings.write().unwrap();
            settings.java_settings.push(JavaDetails::default());
            app.sync_settings(window.clone());
        }));

        settings.on_save_settings(clone!([{ window.as_weak() } as window, app], move || {
            let mut settings = app.settings.write().unwrap();
            let new_settings = window.unwrap().global::<Settings>().get_settings();

            settings.java_settings = new_settings.java_settings.iter()
            .map(|java_setting| JavaDetails::from_slint(java_setting))
            .collect();

            settings.set();
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


        settings.on_get_mc_versions(clone!([{ window.as_weak() } as window, rt], move || {
            let _guard = rt.enter();
            rt.spawn(clone!([window], async move {
                let client = Client::new();

                if let Some(list) = MCVersionList::get(&client).await {

                    invoke_from_event_loop(move || {
                        let slint_list = ModelRc::new(
                            VecModel::from(
                                list.versions
                                .into_iter()
                                .map(MCVersionDetails::to_simple)
                                .map(|versions: MCSimpleVersion| MCSimpleVersion::to_slint(&versions))
                                .collect::<Vec<SlMCVersionDetails>>()
                            )
                        );

                        window.unwrap().global::<Settings>().set_version_list(slint_list);
                    }).unwrap();
                }
            }));
        }));

        settings.on_get_instances(clone!([{ window.as_weak() } as window, app, rt, notifier], move |force| {
            rt.spawn(clone!([window, app, notifier, rt], async move {

                if app.instances.read().unwrap().is_none() || force {
                    let instances = instances::get_instances(app.clone(), notifier.make_new()).await;

                    match instances {
                        Ok(inst) => *app.instances.write().unwrap() = Some(inst),
                        Err(err) => error!("Failed to gather instances: {err}")
                    }
                }

                invoke_from_event_loop(clone!([app, window, rt], move || {
                    spawn_local(clone!([app, window, rt], async move {
                        let slint_instances = if let Some(instances) = app.instances.read().unwrap().as_ref() {
                            let mut result = Vec::new();
                            let _guard = rt.enter();
                            for inst in instances {
                                result.push(inst.to_slint().await);
                            }
                            Some(result)
                        } else { None };
        
                        let slint_list = match slint_instances {
                            Some(instances) => ModelRc::new(
                                VecModel::from(
                                    instances
                                )
                            ),
                            None => ModelRc::default()
                        };
        
                        window.unwrap().global::<Settings>().set_instances(slint_list);
                        window.unwrap().global::<Settings>().set_is_loading_instances(false);
                    })).unwrap();
                })).unwrap();

            }));

        }));

        settings.on_grid_instances(clone!([], move |width, instances, instance_size| {
            ModelRc::new({
                let mut result = Vec::new();
                let mut vec = Vec::new();

                let instances = instances.iter();
                let per_row = (width / ((30 - instance_size) * 15) as f32).ceil() as i32;
                let mut i = 0;

                for inst in instances {
                    vec.push(inst);
                    i += 1;
                    if i == per_row {
                        result.push(ModelRc::new(VecModel::from(vec)));
                        vec = Vec::new();
                        i = 0;
                    }
                }
                if !vec.is_empty() {
                    result.push(ModelRc::new(VecModel::from(vec)));
                }
                
                VecModel::from(result)
            })
        }));

        settings.on_launch_instance(clone!([app, rt, notifier], move |instance_id| {
            rt.spawn(clone!([app, notifier], async move {
                SimpleInstance::launch(app, instance_id, notifier.make_new()).await.unwrap();
            }));
        }));

        settings.on_get_accounts(clone!([{ window.as_weak() } as window, app], move || {
            window.unwrap().global::<Settings>().set_accounts(
                app.accounts.read().unwrap().to_slint()
            );
        }));

        settings.on_grid_accounts(clone!([app], move |width, accounts| {
            ModelRc::new({
                let mut result = Vec::new();
                let mut vec = Vec::new();

                let accounts = accounts.iter();
                let per_row = (width / ((30 - app.settings.read().unwrap().instance_size) * 15) as f32).ceil() as i32;
                let mut i = 0;

                for acc in accounts {
                    vec.push(acc);
                    i += 1;
                    if i == per_row {
                        result.push(ModelRc::new(VecModel::from(vec)));
                        vec = Vec::new();
                        i = 0;
                    }
                }
                if !vec.is_empty() {
                    result.push(ModelRc::new(VecModel::from(vec)));
                }
                
                VecModel::from(result)
            })
        }));

        settings.on_set_selected_account(clone!([app, { window.as_weak() } as window], move |index| {
            let mut accounts = app.accounts.write().unwrap();
            accounts.set_selected_index(index as u32);
            app.sync_accounts(window.clone());
        }));

        settings.on_remove_account(clone!([app, { window.as_weak() } as window], move |index| {
            let mut accounts = app.accounts.write().unwrap();
            accounts.remove_account(index as usize);
            app.sync_accounts(window.clone());
        }));

        settings.on_add_account(clone!([rt, app, { window.as_weak() } as window, notifier], move || {
            let _guard = rt.enter();
            rt.spawn(clone!([rt, app, window, notifier], async move {
                let (sender, mut receiver) = mpsc::unbounded_channel();

                add_account(rt, app.clone(), notifier.make_new(), sender).await;

                if let Some(()) = receiver.recv().await {
                    invoke_from_event_loop(move || {
                        app.sync_accounts(window);
                    }).unwrap();
                }
            }));
        }));


        info!("Running (took {:?})", Instant::now() - time);
        window.run()?;

        debug!("Shutting down...");
        cancel_token.cancel();
        Ok(())
    }

    fn new() -> Self {
        Self {
            settings: RwLock::new(AppSettings::get()),
            accounts: RwLock::new(Accounts::get()),
            instances: RwLock::new(None)
        }
    }

    fn sync_settings(&self, window: Weak<MainWindow>) {
        window.unwrap().global::<Settings>().set_settings(self.settings.read().unwrap().to_slint());
    }

    fn sync_accounts(&self, window: Weak<MainWindow>) {
        window.unwrap().global::<Settings>().set_accounts(self.accounts.read().unwrap().to_slint());
    }
}