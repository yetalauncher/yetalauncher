use std::{path::{PathBuf, Path}, str::FromStr};

use log::*;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use tokio::fs;

use crate::app::{settings::AppSettings, utils::download_file_checked};

use super::{errors::InstanceGatherError, IResult, InstanceType, META_FILENAME};



// Handling the "minecraftinstance.json" file
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CFInstance {
    pub last_played: String,
    pub name: String,
    pub game_version: String,
    pub base_mod_loader: Option<CFBaseLoader>,
    pub installed_modpack: Option<CFInstalledPack>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CFBaseLoader {
    #[serde(rename = "forgeVersion")]
    pub version: String,
    pub minecraft_version: String,
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CFInstalledPack {
    pub thumbnail_url: Option<String>,
    pub addon_i_d: u64
}

#[derive(Debug, Serialize, Deserialize)]
struct CFProject {
    icon_url: String
}


impl CFInstance {
    pub async fn get(instance_path: &Path) -> IResult<Self> {
        let path = instance_path.join("minecraftinstance.json");
        let pack_file = fs::read(&path).await.map_err(
            |err| InstanceGatherError::FileReadFailed(path.clone(), err)
        )?;

        serde_json::from_slice(&pack_file).map_err(
            |err| InstanceGatherError::ParseFailedJson(InstanceType::CurseForge, path, err)
        )
    }

    async fn download_icon(instance_path: &PathBuf) -> IResult<Option<String>> {
        let instance = Self::get(instance_path).await?;

        if let Some(path) = AppSettings::get().icon_path {
            let file = PathBuf::from_str(&path).map_err(
                |err| InstanceGatherError::IconPathParseFailed(path, err)
            )?.join(format!("curseforge_{}", fastrand::u32(..)));
    
            if let Some(pack) = instance.installed_modpack {
                let client = Client::new();
                if let Some(url) = pack.thumbnail_url {
                    download_file_checked(&client, None, &file, &url).await;
                    Ok(Some(file.to_string_lossy().to_string()))
                } else {
                    info!("Requesting icon for project {}", pack.addon_i_d);
                    let project: CFProject = client
                    .get(format!("https://curserinth-api.kuylar.dev/v2/project/{}", pack.addon_i_d))
                    .send()
                    .await
                    .map_err(|err|
                        InstanceGatherError::IconDownloadFailed(instance.name.to_string(), format!("Failed to send curserinth request: {err}"))
                    )?
                    .json()
                    .await
                    .map_err(|err|
                        InstanceGatherError::IconDownloadFailed(instance.name.to_string(), format!("Failed to parse curserinth response: {err}"))
                    )?;

                    download_file_checked(&client, None, &file, &project.icon_url).await;
                    Ok(Some(file.to_string_lossy().to_string()))
                }
            } else { Ok(None) }
        } else { Ok(None) }
    }
}

// Handling our metadata ("yamcl-data.json" file)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CFMetadata {
    pub instance_id: u32,
    pub saved_icon: String
}

impl CFMetadata {
    pub async fn get(instance_path: &PathBuf) -> IResult<Self> {
        let path = instance_path.join(META_FILENAME);

        match fs::read(&path).await {
            Ok(contents) => {
                let result = match serde_json::from_slice(&contents) {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => {
                        warn!("{}", InstanceGatherError::ParseFailedMeta(path, err));
                        Ok(Self::generate(instance_path).await?)
                    },
                };
        
                if result.as_ref().is_ok_and(
                    |meta| PathBuf::from_str(&meta.saved_icon).map_or(
                        true, |icon| !icon.exists()
                    )
                ) {
                    Self::generate(instance_path).await
                } else {
                    result
                }
            },
            Err(err) => {
                warn!("{}", InstanceGatherError::FileReadFailed(path, err));
                Self::generate(instance_path).await
            }
        }
    }

    async fn generate(instance_path: &PathBuf) -> IResult<Self> {
        let path = instance_path.join(META_FILENAME);

        let meta = CFMetadata {
            instance_id: fastrand::u32(..),
            saved_icon: match CFInstance::download_icon(instance_path).await {
                Ok(Some(icon)) => icon,
                Ok(None) => "default_instance.png".to_string(),
                Err(err) => {
                    warn!("{err}");
                    "default_instance.png".to_string()
                }
            }
        };

        fs::write(&path, serde_json::to_string_pretty(&meta).unwrap(/* this cannot fail */)).await.map_err(
            |err| InstanceGatherError::FileWriteFailed(path, err)
        )?;

        Ok(meta)
    }
}