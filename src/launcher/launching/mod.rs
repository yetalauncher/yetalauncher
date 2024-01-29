use std::{process::Command, path::PathBuf};

use log::{*};
use reqwest::Client;

use crate::{app::{accounts::get_active_account, utils::{get_classpath_separator, get_library_dir, NotificationState, Notifier}}, launcher::launching::mc_structs::*};

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

pub async fn launch_instance(instance: SimpleInstance, java: JavaDetails) -> Result<(), String> {
    let SimpleInstance { minecraft_path, id, mc_version, .. } = instance.clone();
    let notifier = Notifier::new(&format!("{id}_status"));
    info!("Launching: {minecraft_path:?}, Version: {mc_version}, id: {id}");

    let args = get_arguments(&instance, &java).await?;
    let additional_args = java.get_args();

    debug!("Args: {:#?}\nCustom Args: {}", args, additional_args);
    info!("Launching NOW!");

    let mut process = Command::new(java.path)
    .current_dir(&minecraft_path)
    .args(additional_args.split_whitespace())
    .args(args.jvm)
    .arg(args.main_class)
    .args(args.game)
    .spawn()
    .map_err(|err| format!("Failed to run Minecraft command: {err}"))?;

    notifier.notify("Instance launched successfully!", NotificationState::Success);

    let exit_status = process.wait().expect("Failed to wait on Java process! How did this happen?");
    info!("Exited with status: {}", exit_status);

    if exit_status.success() {
        info!("{minecraft_path:?} exited successfully.");
        notifier.notify("Instance exited successfully.", NotificationState::Success);
    } else {
        warn!("{minecraft_path:?} exited (crashed) with status {}", exit_status);
        notifier.notify(&format!("Instance crashed with code {}", exit_status.code().unwrap_or(323)), NotificationState::Error);
    }

    Ok(())
}

async fn get_arguments(instance: &SimpleInstance, java: &JavaDetails) -> Result<Args, String> {
    let client = Client::new();

    let loader = instance.modloader.typ;

    let mut account = get_active_account()
        .ok_or("Could not get the selected account!".to_string())?;
    
    account.refresh(&client, false).await;

    info!("Getting version details for {}", instance.mc_version);
    let compact_version = MCVersionDetails::from_id(instance.mc_version.clone(), &client)
        .await
        .ok_or("Could not get Minecraft version details!".to_string())?;

    debug!("Got compact version info: {:?}", compact_version);
    info!("Getting version manifest from {}", compact_version.url);

    let mut version = compact_version.get_manifest(&client)
        .await
        .ok_or("Could not get Minecraft version manifest!".to_string())?;

    debug!("Pre-downloading client jar...");
    version.get_client_jar(&client).await;

    if let Some(mf) = loader.get_manifest(&instance.mc_version, &instance.modloader.version, &client).await {
        info!("Merging with manifest of {loader} Loader...");
        version.merge_with(mf)
    }

    info!("Finished getting manifest.");

    loader.prepare_launch(&instance.mc_version, &instance.modloader.version, &client, &java.path).await;

    info!("Beginning argument parsing...");
    Ok(
        parse_arguments(
            Args {
                jvm: version.get_jvm_args(&client).await,
                game: version.get_game_args(),
                main_class: version.get_main_class()
            },
            account,
            version,
            &instance.minecraft_path,
            &client
        ).await
    )
}

async fn parse_arguments(args_struct: Args, account: MCAccount, version: MCVersionManifest, minecraft_path: &PathBuf, client: &Client) -> Args {
    let replacements = vec![
        ("${auth_player_name}", account.mc_profile.name),
        ("${auth_uuid}", account.mc_profile.id),
        ("${auth_access_token}", account.mc_response.access_token),
        ("${auth_xuid}", account.xsts_response.display_claims.xui[0].uhs.to_string()), // idk what else a "xuid" could be
        ("${user_properties}", "something".to_string()),

        ("${classpath}", version.get_classpath(client).await),
        ("${assets_root}", version.get_client_assets(client).await),
        ("${version_name}", version.id.replace(' ', "_").replace(':', "_")),
        ("${assets_index_name}", version.asset_index.id),
        ("${version_type}", version.typ),

        ("${natives_directory}", format!("{minecraft_path:?}/natives")),
        ("${launcher_name}", "yamcl".to_string()),
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