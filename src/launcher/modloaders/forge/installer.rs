use std::{path::PathBuf, iter};

use tokio::fs;
use jars::JarOptionBuilder;
use log::{*};
use reqwest::{Client, StatusCode};
use serde::{Serialize, Deserialize};
use tokio::process::Command;

use crate::{app::{downloader::{Download, DownloadErr}, notifier::Notifier, utils::*}, launcher::modloaders::forge::{legacy_installer::LegacyInstallerManifest, ForgeVersionManifest}};

use super::install_profile::{ForgeInstallProfile, ModernInstallProfile};

pub struct ForgeInstaller;

impl ForgeInstaller {
    async fn download(mc_ver: &str, forge_ver: &str, client: &Client, notifier: &mut Notifier) -> Result<PathBuf, DownloadErr> {
        info!("Downloading Forge installer for {mc_ver}-{forge_ver}...");
        let path = get_forge_cache_dir().join(format!("forge-{mc_ver}-{forge_ver}-installer.jar"));

        let download = Download::new(
            path.clone(),
            &format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_ver}-{forge_ver}/forge-{mc_ver}-{forge_ver}-installer.jar"),
            None,
            None
        ).download(client, notifier).await.map(|_| path.clone());

        if let Err(DownloadErr::Response(StatusCode::NOT_FOUND)) = download {
            debug!("Falling back to {}", format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_ver}-{forge_ver}-{mc_ver}/forge-{mc_ver}-{forge_ver}-{mc_ver}-installer.jar"));
            
            Download::new(
                path.clone(),
                &format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_ver}-{forge_ver}-{mc_ver}/forge-{mc_ver}-{forge_ver}-{mc_ver}-installer.jar"),
                None,
                None
            ).download(client, notifier).await.map(|_| path)
        } else {
            download
        }
    }

    pub async fn prepare_jar(mc_ver: &str, forge_ver: &str, client: &Client, java_path: &str, notifier: &mut Notifier) {
        let path = get_library_dir()
        .join("net/minecraftforge/forge")
        .join(format!("{mc_ver}-{forge_ver}"))
        .join(format!("forge-{mc_ver}-{forge_ver}-client.jar"));

        if !path.is_file() {
            notifier.set_progress(1, 5);

            if let Some(install_profile) = ForgeInstallProfile::get(mc_ver, forge_ver, client, notifier).await {
                match install_profile {
                    ForgeInstallProfile::Modern(mut profile) => {
                        notifier.send_progress("Downloading installer libraries...", 3);
                        profile.download_libraries(notifier.make_new()).await;
            
                        notifier.send_progress("Running installer processors...", 4);
                        profile.process(Side::Client, java_path, notifier).await;
            
                        notifier.set_progress(0, 0);
                        notifier.send_success("Successfully installed forge");
                    },
                    ForgeInstallProfile::Legacy(_) => {
                        debug!("Legacy install profile found, no further actions required");

                        notifier.set_progress(0, 0);
                        notifier.send_success("Successfully installed forge");
                    },
                }

            }
        }
    }

    /// ### Downloads the Forge installer and extracts the manifest and the install_profile from it
    /// Target location: `forge-{mc_ver}-{forge_ver}-[installer.jar/manifest.json/install_profile.json]` in the forge cache dir
    pub async fn extract_needed(mc_ver: &str, forge_ver: &str, client: &Client, notifier: &mut Notifier) {
        notifier.send_progress("Downloading Forge installer...", 1);
        let installer = Self::download(mc_ver, forge_ver, client, &mut notifier.make_new()).await.expect("Failed to download Forge installer!");

        notifier.send_progress("Extracting Forge installer...", 2);
        debug!("Extracting installer jar...");
        let jar = jars::jar(
            installer, 
            JarOptionBuilder::builder()
            .targets(&vec!["version.json", "install_profile.json", "data"])
            .ext("jar")
            .build()
        ).expect("Failed to extract Forge installer jar!");

        let legacy_installer = jar.files.iter().find(|(name, _)| **name == "version.json".to_string()).is_none(); // For versions 1.6 to 1.9

        for (f_path, f_contents) in jar.files.iter().filter(|(_, contents)| !contents.is_empty()) {
            let is_legacy = legacy_installer && f_path == "install_profile.json";

            if is_legacy {
                info!("Detected legacy Forge installer! Running workarounds...");
                let legacy_manifest: LegacyInstallerManifest = serde_json::from_slice(&f_contents).expect("Failed to parse legacy installer manifest!");

                let version_manifest_path = ForgeVersionManifest::get_path(mc_ver, forge_ver);
                let install_profile_path = ForgeInstallProfile::get_path(mc_ver, forge_ver);

                legacy_manifest.install_profile.copy_libraries(&jar.files).await;

                create_dir_parents(&version_manifest_path).await;
                create_dir_parents(&install_profile_path).await;

                fs::write(
                    version_manifest_path,
                    serde_json::to_string_pretty(&legacy_manifest.version_manifest).unwrap()
                ).await.expect("Failed to write manifest to file!");
                fs::write(
                    install_profile_path,
                    serde_json::to_string_pretty(&legacy_manifest.install_profile).unwrap()
                ).await.expect("Failed to write install profile to file!");

            } else {
                fs::write(f_path, f_contents).await.expect("Failed to write manifest to file!");
            }
        }
    }
}



#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeProcessor {
    sides: Option<Vec<Side>>,
    jar: String,
    classpath: Vec<String>,
    args: Vec<String>
}


impl ForgeProcessor {
    pub async fn run(&self, side: &Side, install_profile: &ModernInstallProfile, java_path: &str) {
        let shouldrun = self.sides.is_none() || self.sides.as_ref().is_some_and(|s| s.contains(side));

        if !shouldrun { return; }

        info!("Starting processor {}...", self.jar);

        let classpath = self.classpath
        .iter()
        .chain(iter::once(&self.jar))
        .map(|cp| get_library_dir().join(maven_identifier_to_path(&cp)).to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join(&get_classpath_separator());

        let args = self.parse_args(install_profile, side);

        let main_class = get_jar_main_class(get_library_dir().join(maven_identifier_to_path(&self.jar)));

        info!("Running processor...");

        let process = Command::new(java_path)
        .arg("-cp").arg(classpath)
        .arg(main_class)
        //.arg("-jar").arg(get_library_dir().join(maven_identifier_to_path(&self.jar)))
        .args(args)
        .spawn()
        .expect(&format!("Failed to run Processor {}", self.jar))
        .wait()
        .await
        .expect(&format!("Failed to wait on Processor {} process", self.jar));

        if process.success() {
            info!("Processor exited successfully.")
        } else {
            panic!("Processor crashed with code: {:?}", process.code())
        }
    }

    fn parse_args(&self, install_profile: &ModernInstallProfile, side: &Side) -> Vec<String> {
        self.args.iter().map(|arg| {
            let mut final_arg = arg.to_string();
            if arg.starts_with("{") && arg.ends_with("}") {
                let key = &arg[1..arg.len()-1];

                final_arg = match key {
                    "SIDE" => format!("{:?}", side).to_lowercase(),
                    "MINECRAFT_JAR" => get_client_jar_dir().join(format!("{}.jar", &install_profile.minecraft)).to_string_lossy().to_string(),
                    _ => install_profile.data.get(key).expect(&format!("Key {key} was not found in data!")).get_value(side)
                };
            } else if arg.starts_with("[") && arg.ends_with("]") {
                let identifier = &arg[1..arg.len()-1];

                let path = get_library_dir().join(maven_identifier_to_path(identifier));
                
                if !path.is_file() {
                    panic!("File at {:?} could not be found!", path)
                } else {
                    final_arg = path.to_string_lossy().to_string()
                }
            }

            let data_dir = get_data_dir().to_string_lossy().to_string();
            if final_arg.starts_with("/") && !final_arg.contains(data_dir.as_str()) {
                let (mc_ver, forge_ver) = install_profile.version.split_once("-forge-").unwrap();
                final_arg = get_installer_extracts_dir(mc_ver, forge_ver).join(&final_arg[1..]).to_string_lossy().to_string()
            }

            final_arg
        }).collect()
    }

    pub fn get_task(&self) -> &str {
        let arg = &self.args[1];

        if !arg.starts_with("{") && !arg.starts_with("[") && arg == &arg.to_uppercase() {
            arg
        } else {
            &self.jar
        }
    }
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Server,
    Client
}