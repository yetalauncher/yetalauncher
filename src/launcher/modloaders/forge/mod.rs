use std::path::PathBuf;

use tokio::fs;

use reqwest::Client;
use serde::{Serialize, Deserialize};

use crate::{app::{notifier::Notifier, utils::get_installer_extracts_dir}, launcher::launching::mc_structs::{MCArguments, MCLibrary}};

use self::installer::ForgeInstaller;

pub mod installer;
pub mod install_profile;
pub mod legacy_installer;


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgeVersionManifest {
    pub arguments: Option<MCArguments>,
    pub minecraft_arguments: Option<String>,
    pub id: String,
    pub libraries: Vec<ForgeLibrary>,
    pub main_class: String,
    pub inherits_from: String,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ForgeLibrary {
    Vanilla(MCLibrary),
    Forge(LegacyForgeLibrary)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyForgeLibrary {
    pub name: String,
    pub url: Option<String>,
    pub serverreq: Option<bool>,
    pub clientreq: Option<bool>
}

impl ForgeVersionManifest {
    pub async fn get(mc_ver: &str, forge_ver: &str, client: &Client, notifier: &mut Notifier) -> Option<Self> {
        let path = Self::get_path(mc_ver, forge_ver);
        if !path.exists() {
            notifier.set_progress(1, 2);
            ForgeInstaller::extract_needed(mc_ver, forge_ver, client, notifier).await;
            notifier.send_success("Got Forge version manifest");
        }

        let manifest = fs::read_to_string(path).await.expect("Failed to read manifest file!?");
        Some(serde_json::from_str(&manifest).expect("Failed to parse manifest file!?"))
    }

    pub fn get_path(mc_ver: &str, forge_ver: &str) -> PathBuf {
        get_installer_extracts_dir(mc_ver, forge_ver).join("version.json")
    }
}
