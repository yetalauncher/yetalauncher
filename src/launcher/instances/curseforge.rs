use std::{path::{Path, PathBuf}, str::FromStr, sync::Arc};

use log::*;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use tokio::fs;

use crate::app::{consts::META_FILE_NAME, settings::AppSettings, utils::download_file_checked};

use super::{errors::InstanceGatherError, IResult, InstanceType};



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

    async fn download_icon(instance_path: &PathBuf, settings: Arc<AppSettings>) -> IResult<Option<String>> {
        let instance = Self::get(instance_path).await?;

        if let Some(path) = settings.icon_path.as_ref() {
            let file = PathBuf::from_str(&path.to_string()).map_err(
                |err| InstanceGatherError::IconPathParseFailed(path.to_string(), err)
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
    pub saved_icon: Option<String>
}

impl CFMetadata {
    pub async fn get(instance_path: &PathBuf, settings: Arc<AppSettings>) -> IResult<Self> {
        let path = instance_path.join(META_FILE_NAME);

        match fs::read(&path).await {
            Ok(contents) => {
                let result = match serde_json::from_slice(&contents) {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => {
                        warn!("{}", InstanceGatherError::ParseFailedMeta(path, err));
                        Ok(Self::generate(instance_path, settings.clone()).await?)
                    },
                };
                
                if result.as_ref().is_ok_and( // If parsing/generation succeeded, and:
                    |meta| if let Some(saved_icon) = &meta.saved_icon { // saved_icon is not null, and:
                        PathBuf::from_str(saved_icon).map_or( // the file path couldn't be parsed, or:
                            true, |icon| !icon.exists() // targeted icon file does not exist
                        )
                    } else { false } // (if file path is unset, no need to regenerate ever)
                ) {
                    Self::generate(instance_path, settings).await // then regenerate the metadata
                } else {
                    result
                }
            },
            Err(err) => {
                warn!("{}", InstanceGatherError::FileReadFailed(path, err));
                Self::generate(instance_path, settings).await
            }
        }
    }

    async fn generate(instance_path: &PathBuf, settings: Arc<AppSettings>) -> IResult<Self> {
        let path = instance_path.join(META_FILE_NAME);

        let meta = CFMetadata {
            instance_id: fastrand::u32(..),
            saved_icon: match CFInstance::download_icon(instance_path, settings).await {
                Ok(icon) => icon,
                Err(err) => {
                    warn!("{err}");
                    None
                }
            }
        };

        fs::write(&path, serde_json::to_string_pretty(&meta).unwrap(/* this cannot fail */)).await.map_err(
            |err| InstanceGatherError::FileWriteFailed(path, err)
        )?;

        Ok(meta)
    }
}