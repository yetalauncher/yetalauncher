use std::{fs, collections::HashMap};

use log::info;
use reqwest::Client;
use serde::{Serialize, Deserialize};

use crate::{app::utils::{get_library_dir, maven_identifier_to_path}, launcher::launching::mc_structs::{MCArguments, MCLibrary}};

use super::forge_installer::{ForgeInstaller, get_manifest_path, get_install_profile_path, ForgeProcessor, Side};


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgeVersionManifest {
    pub arguments: Option<MCArguments>,
    pub minecraft_arguments: Option<String>,
    pub id: String,
    pub libraries: Vec<MCLibrary>,
    pub main_class: String,
    pub inherits_from: String,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgeInstallProfile {
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

impl ForgeVersionManifest {
    pub async fn get(mc_ver: &str, forge_ver: &str, client: &Client) -> Option<Self> {
        let path = get_manifest_path(mc_ver, forge_ver);
        if !path.exists() {
            ForgeInstaller::extract_needed(mc_ver, forge_ver, client).await
        }

        let manifest = fs::read_to_string(path).expect("Failed to read manifest file!?");
        serde_json::from_str(&manifest).ok()
    }

}

impl ForgeInstallProfile {
    pub async fn get(mc_ver: &str, forge_ver: &str, client: &Client) -> Option<Self> {
        let path = get_install_profile_path(mc_ver, forge_ver);
        if !path.exists() {
            ForgeInstaller::extract_needed(mc_ver, forge_ver, client).await
        }

        let install_profile = fs::read_to_string(path).expect("Failed to read install profile file!?");
        Some(serde_json::from_str(&install_profile).unwrap())
    }

    pub fn process(&self, side: Side, java_path: &str) {
        for proc in &self.processors {
            proc.run(&side, self, java_path);
        }
    }

    pub async fn download_libraries(&mut self, client: &Client) {
        info!("Downloading installer libraries...");
        for lib in &mut self.libraries {
            if let Some(artifact) = &mut lib.downloads.artifact {
                if artifact.url.is_empty() && lib.name.contains("minecraftforge") {
                    artifact.url = format!("https://maven.minecraftforge.net/{}", maven_identifier_to_path(&lib.name))
                }
            }
            lib.download_checked(client).await;
        }
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