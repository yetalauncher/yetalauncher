use log::{info, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{app::utils::maven_identifier_to_path, launcher::launching::mc_structs::{MCArguments, MCLibrary}};


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricVersionManifest {
    pub arguments: MCArguments,
    pub id: String,
    pub libraries: Vec<FabricLibrary>,
    pub main_class: String,
    pub inherits_from: String,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FabricLibrary {
    pub name: String,
    pub url: String,
    pub sha1: Option<String>,
    pub size: Option<u32>
}

impl FabricVersionManifest {
    pub async fn get(mc_ver: &str, fabric_loader_ver: &str, client: &Client) -> Option<Self> {
        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{mc_ver}/{fabric_loader_ver}/profile/json");
        info!("Getting Fabric version manifest from {url}...");

        match client.get(url).send().await.unwrap().json::<Self>().await {
            Ok(manifest) => Some(manifest),
            Err(e) => {
                error!("Failed to get fabric version manifest: {}", e);
                None
            }
        }
    }
}

impl FabricLibrary {
    pub fn to_vanilla(self) -> MCLibrary {
        let path = maven_identifier_to_path(&self.name);

        MCLibrary::new_simple(
            self.name,
            format!("{}{}", self.url, path),
            path,
            None,
            self.sha1
        )
    }
}