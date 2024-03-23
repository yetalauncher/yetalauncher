use std::{collections::HashMap, path::PathBuf};

use log::{*};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{app::{downloader::Downloader, notifier::Notifier, utils::{get_installer_extracts_dir, get_library_dir, maven_identifier_to_path}}, launcher::launching::mc_structs::MCLibrary};

use super::{installer::{ForgeInstaller, ForgeProcessor, Side}, legacy_installer::LegacyInstallProfile};


#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ForgeInstallProfile {
    Modern(ModernInstallProfile),
    Legacy(LegacyInstallProfile)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModernInstallProfile {
    pub spec: Option<u16>,
    pub version: String,
    pub minecraft: String,
    pub server_jar_path: Option<String>,
    pub data: HashMap<String, ForgeMappings>,
    pub processors: Vec<ForgeProcessor>,
    pub libraries: Vec<MCLibrary>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeMappings {
    client: String,
    server: String
}


impl ForgeInstallProfile {
    pub async fn get(mc_ver: &str, forge_ver: &str, client: &Client, notifier: &mut Notifier) -> Option<Self> {
        let path = Self::get_path(mc_ver, forge_ver);
        if !path.exists() {
            ForgeInstaller::extract_needed(mc_ver, forge_ver, client, notifier).await;
        }

        let install_profile = fs::read_to_string(path).await.ok()?;
        Some(serde_json::from_str(&install_profile).unwrap())
    }


    pub fn get_path(mc_ver: &str, forge_ver: &str) -> PathBuf {
        get_installer_extracts_dir(mc_ver, forge_ver).join("install_profile.json")
    }
}

impl ModernInstallProfile {
    pub async fn process(&self, side: Side, java_path: &str, notifier: &mut Notifier) {
        let length = self.processors.len();

        let mut notifier = notifier.make_new();
        notifier.send_msg("TEST");

        for (i, proc) in self.processors.iter().enumerate() {
            notifier.set_progress((i + 1) as u32, length as u32);
            notifier.send_msg(&format!("Running task: {}", proc.get_task()));

            proc.run(&side, self, java_path).await;
        }

        notifier.set_progress(0, 0);
        notifier.send_success("Finished running processors")
    }

    pub async fn download_libraries(&mut self, notifier: Notifier) {
        info!("Downloading installer libraries...");

        let mut downloader = Downloader::new(notifier, 8);

        for lib in &mut self.libraries {
            if let Some(artifact) = &mut lib.downloads.artifact {
                if artifact.url.is_empty() && lib.name.contains("minecraftforge") {
                    let maven_name = maven_identifier_to_path(&lib.name);
                    let forge_name = format!("{}-universal.jar", &maven_name[..maven_name.len()-4]);

                    artifact.url = format!("https://maven.minecraftforge.net/{forge_name}");
                }
            }

            for dl in lib.get_downloads() {
                downloader.add(dl)
            }
        }

        downloader.download_all(true, "Forge libraries").await;
    }
}

impl ForgeMappings {
    pub fn get_value(&self, side: &Side) -> String {
        let val = match side {
            Side::Client => &self.client,
            Side::Server => &self.server,
        };

        if val.starts_with("[") && val.ends_with("]") {
            let identifier = &val[1..val.len()-1];

            get_library_dir().join(maven_identifier_to_path(identifier)).to_string_lossy().to_string()
        } else { val.to_string() }
    }
}