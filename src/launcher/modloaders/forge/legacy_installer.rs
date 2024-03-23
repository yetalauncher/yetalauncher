use std::collections::HashMap;

use log::info;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::app::utils::{create_dir_parents, get_library_dir, maven_identifier_to_path};

use super::ForgeVersionManifest;



#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyInstallerManifest {
    #[serde(rename = "install")]
    pub install_profile: LegacyInstallProfile,
    #[serde(rename = "versionInfo")]
    pub version_manifest: ForgeVersionManifest
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyInstallProfile {
    pub version: String,
    pub minecraft: String,
    pub path: String,
    pub file_path: String,
}

impl LegacyInstallProfile {
    pub async fn copy_libraries(&self, files: &HashMap<String, Vec<u8>>) {
        for (f_name, f_contents) in files {
            if f_name == &self.file_path {
                let output_path = get_library_dir().join(maven_identifier_to_path(&self.path));
                info!("Extracting Forge jar {f_name} to {output_path:?}...");

                create_dir_parents(&output_path).await;
                fs::write(output_path, f_contents).await.expect("Failed to write Forge library file!");
            }
        }
    }
}