use std::{path::{Path, PathBuf}, str::FromStr, sync::Arc};

use log::warn;
use serde::{Serialize, Deserialize};
use tokio::fs;

use crate::app::{consts::META_FILE_NAME, settings::AppSettings};

use super::{errors::InstanceGatherError, IResult, InstanceType};


// Handling the "instance.cfg" file
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MMCConfig {
    pub name: String,
    #[serde(rename = "lastLaunchTime")]
    pub last_played: Option<i64>,
    pub icon_key: Option<String>
}

impl MMCConfig {
    pub async fn get(path: &Path) -> IResult<Self> {
        let instance_file = fs::read_to_string(path.join("instance.cfg")).await.map_err(
            |err| InstanceGatherError::FileReadFailed(path.join("instance.cfg").to_path_buf(), err)
        )?.replace("[General]", ""); // Remove the section if there is one

        serde_ini::from_str(&instance_file).map_err(
            |err| InstanceGatherError::ParseFailedIni(InstanceType::MultiMC, path.to_path_buf(), err)
        )
    }

    pub fn get_icon(&self, settings: Arc<AppSettings>) -> Option<String> {
        let internal_icons = [
            "default", "bee", "brick", "chicken", "creeper", "diamond", "dirt", "enderman", "enderpearl", "flame", "fox", "gear", "herobrine",
            "gold", "grass", "iron", "magitech", "meat", "modrinth", "netherstar", "planks", "prismlauncher", "squarecreeper", "steve", 
            "stone", "tnt", "bee_legacy", "brick_legacy", "chicken_legacy", "creeper_legacy", "diamond_legacy", "dirt_legacy",
            "enderman_legacy", "enderpearl_legacy", "flame_legacy", "fox_legacy", "gear_legacy", "herobrine_legacy", "gold_legacy", "grass_legacy", 
            "iron_legacy", "magitech_legacy", "meat_legacy", "modrinth_legacy", "netherstar_legacy", "planks_legacy", "prismlauncher_legacy",
            "squarecreeper_legacy", "steve_legacy", "stone_legacy", "tnt_legacy"
        ]; // We can't display those since they're built into MultiMC / Prism Launcher

        let icon_key = self.icon_key.as_ref()?;

        if internal_icons.iter().any(|&i| icon_key == i) {
            None
        } else {
            let icon_path = PathBuf::from_str(settings.icon_path.as_ref()?).ok()?;
            Some(icon_path.join(icon_key).to_string_lossy().to_string())
        }
    }
}


// Handling the "mmc-instance.json" file
#[derive(Debug, Deserialize)]
pub struct MMCPack {
    pub components: Vec<MMCPackComponent>
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct MMCPackComponent {
    pub cached_name: Option<String>,
    pub cached_version: Option<String>,
    pub dependency_only: Option<bool>,
    pub uid: String,
    pub version: Option<String>
}

impl MMCPack {
    pub async fn get(instance_path: &Path) -> IResult<Self> {
        let path = instance_path.join("mmc-pack.json");
        let pack_file = fs::read(&path).await.map_err(
            |err| InstanceGatherError::FileReadFailed(path.clone(), err)
        )?;

        serde_json::from_slice(&pack_file).map_err(
            |err| InstanceGatherError::ParseFailedJson(InstanceType::MultiMC, path, err)
        )
    }
}

// Handling our metadata ("yamcl-data.json" file)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MMCMetadata {
    pub instance_id: u32
}

impl MMCMetadata {
    pub async fn get(instance_path: &Path) -> IResult<Self> {
        let path = instance_path.join(META_FILE_NAME);

        match fs::read(&path).await {
            Ok(contents) => {
                match serde_json::from_slice(&contents) {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => {
                        warn!("{}", InstanceGatherError::ParseFailedMeta(path, err));
                        Ok(Self::generate(instance_path).await?)
                    },
                }
            },
            Err(err) => {
                warn!("{}", InstanceGatherError::FileReadFailed(path, err));
                Self::generate(instance_path).await
            },
        }
    }

    async fn generate(instance_path: &Path) -> IResult<Self> {
        let path = instance_path.join(META_FILE_NAME);

        let meta = MMCMetadata {
            instance_id: fastrand::u32(..),
        };

        fs::write(&path, serde_json::to_string_pretty(&meta).unwrap(/* this cannot fail */)).await.map_err(
            |err| InstanceGatherError::FileWriteFailed(path, err)
        )?;

        Ok(meta)
    }
}