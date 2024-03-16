use std::path::PathBuf;

use log::{*};
use reqwest::Client;
use tokio::{fs, process::Command};

use crate::{app::{notifier::Notifier, settings::AppSettings, utils::{get_classpath_separator, get_library_dir}}, launcher::{authentication::auth_structs::Accounts, launching::mc_structs::*}};

use super::{authentication::auth_structs::MCAccount, instances::SimpleInstance, java::JavaDetails};

pub mod libraries;
pub mod manifests;
pub mod mc_structs;

#[derive(Debug)]
struct Args {
    jvm: Vec<String>,
    game: Vec<String>,
    main_class: String
}


impl SimpleInstance {
    pub async fn launch(&self, settings: &AppSettings, accounts: &mut Accounts, mut notifier: Notifier) -> Result<(), String> {
        let SimpleInstance { minecraft_path, id, mc_version, name, .. } = &self;
        notifier.set_progress(1, 9);

        if !minecraft_path.is_dir() {
            if !minecraft_path.exists() {
                fs::create_dir_all(minecraft_path).await.map_err(
                    |err| format!("Failed to create minecraft directory: {err}")
                )?;
            } else {
                Err(String::from("Target of minecraft path is invalid!"))?;
            }
        };

        info!("Launching: {minecraft_path:?}, Version: {mc_version}, id: {id}");
        notifier.send_msg(&format!("Launching {name}..."));

        let client = Client::new();
        let java = self.get_java(settings, &client).await.map_err(
            |err| { notifier.send_error(&err); err }
        )?;
        let args = self.get_arguments(java, accounts, &client, &mut notifier).await?;
        let additional_args = java.get_args();
    
        debug!("Args: {:#?}\nCustom Args: {}", args, additional_args);
        info!("Launching NOW!");


        let mut process = Command::new(&java.path)
        .current_dir(&minecraft_path)
        .args(additional_args.split_whitespace())
        .args(args.jvm)
        .arg(args.main_class)
        .args(args.game)
        .spawn()
        .map_err(|err| format!("Failed to run Minecraft command: {err}"))?;
    

        notifier.set_progress(0, 0);
        notifier.send_success("Instance launched successfully!");
    
        let exit_status = process.wait().await.expect("Failed to wait on Java process! How did this happen?");
        info!("Exited with status: {}", exit_status);
    
        if exit_status.success() {
            info!("{minecraft_path:?} exited successfully.");
            notifier.make_new().send_success(&format!("{name} exited successfully."));
        } else {
            warn!("{minecraft_path:?} exited (crashed) with status {}", exit_status);
            notifier.make_new().send_error(
                &format!("{name} crashed with code {}", exit_status.code().map_or("None".to_string(), |c| c.to_string())
            ));
        }

        Ok(())
    }
    
    async fn get_arguments(&self, java: &JavaDetails, accounts: &mut Accounts, client: &Client, notifier: &mut Notifier) -> Result<Args, String> {
        let loader = self.modloader.typ;
    
        notifier.send_progress("Preparing account...", 2);
        info!("Preparing account...");
        let account = accounts.get_active_account(client, false).await
            .ok_or("Could not get the selected account!".to_string())?;
        

        notifier.send_progress(&format!("Getting version details for {}...", self.mc_version), 3);
        info!("Getting version details for {}...", self.mc_version);
        let compact_version = MCVersionDetails::from_id(&self.mc_version, &client)
            .await
            .ok_or("Could not get Minecraft version details!".to_string())?;
    
        debug!("Got compact version info: {:?}", compact_version);


        notifier.send_progress(&format!("Getting version manifest for {}...", self.mc_version), 4);
        info!("Getting version manifest from {}", compact_version.url);
        let mut version = compact_version.get_manifest(&client)
            .await
            .ok_or("Could not get Minecraft version manifest!".to_string())?;
    

        notifier.send_progress("Pre-downloading client jar...", 5);
        debug!("Pre-downloading client jar...");
        version.get_client_jar(&client).await;
    

        notifier.send_progress("Getting the modloader manifest...", 6);
        if let Some(mf) = loader.get_manifest(&self.mc_version, &self.modloader.version, &client, notifier.make_new()).await {
            info!("Merging with manifest of {loader} Loader...");
            version.merge_with(mf)
        }
    
        info!("Finished getting manifest.");


        notifier.send_progress("Preparing the modloader...", 7);
        loader.prepare_launch(&self.mc_version, &self.modloader.version, &client, &java.path, notifier.make_new()).await;
    
        info!("Beginning argument parsing...");
        notifier.send_progress("Preparing the game...", 8);
        Ok(
            Self::parse_arguments(
                Args {
                    jvm: version.get_jvm_args(&client).await,
                    game: version.get_game_args(),
                    main_class: version.get_main_class()
                },
                account,
                version,
                &self.minecraft_path,
                &client,
                notifier
            ).await
        )
    }
    
    async fn parse_arguments(args_struct: Args, account: &MCAccount, version: MCVersionManifest, minecraft_path: &PathBuf, client: &Client, notifier: &mut Notifier) -> Args {
        let replacements = [
            ("${auth_player_name}", account.mc_profile.name.to_string()),
            ("${auth_uuid}", account.mc_profile.id.to_string()),
            ("${auth_access_token}", account.mc_response.access_token.to_string()),
            ("${auth_xuid}", account.xsts_response.display_claims.xui[0].uhs.to_string()), // idk what else a "xuid" could be
            ("${user_properties}", "something".to_string()),
    
            ("${classpath}", version.get_classpath(client, notifier.clone()).await),
            ("${assets_root}", version.get_client_assets(client, notifier.clone()).await),
            ("${version_name}", version.id.replace(' ', "_").replace(':', "_")),
            ("${assets_index_name}", version.asset_index.id),
            ("${version_type}", version.typ),
    
            ("${natives_directory}", minecraft_path.join("natives").to_string_lossy().to_string()),
            ("${launcher_name}", "yetalauncher".to_string()),
            ("${launcher_version}", "323".to_string()),
            ("${game_directory}", minecraft_path.to_string_lossy().to_string()),
            ("${user_type}", "msa".to_string()),
            ("${resolution_width}", 1200.to_string()),
            ("${resolution_height}", 800.to_string()),
    
            // Forge specifics
            ("${classpath_separator}", get_classpath_separator()),
            ("${library_directory}", get_library_dir().to_string_lossy().to_string())
        ];
    
        let to_remove = [
            "quickPlay",
            "--demo"
        ];
    
        let args_final: (Vec<String>, Vec<String>) = [args_struct.jvm, args_struct.game].map(|args| {
            args.into_iter().map(|mut arg| {
                for replacement in &replacements {
                    arg = arg.replace(replacement.0, &replacement.1)
                }
                arg
            }).filter(|arg| {
                !to_remove.iter().any(|remover| arg.contains(remover))
            }).collect()
        }).into();
    
        Args {
            jvm: args_final.0,
            game: args_final.1,
            main_class: args_struct.main_class
        }
    }

    async fn get_java<'a>(&self, settings: &'a AppSettings, client: &Client) -> Result<&'a JavaDetails, String> {
        let version = MCVersionDetails::from_id(&self.mc_version, client).await.ok_or_else(
            || "Could not get version details for this Minecraft version!".to_string()
        )?;

        settings.java_settings
        .iter()
        .find(|&java| {
            java.minecraft_versions.max.as_ref().map_or(
                false, 
                |max| max.release_time >= version.release_time
            )
            &&
            java.minecraft_versions.min.as_ref().map_or(
                false, 
                |min| min.release_time <= version.release_time
            )
        }).ok_or_else(
            || "Could not find Java to use for this version in the settings!".to_string()
        )
    }
}