use std::{path::PathBuf, fmt::Debug, str::FromStr};

use log::error;
use serde::Serialize;
use thiserror::Error;
use tokio::io;

use super::InstanceType;



#[derive(Debug, Error)]
#[allow(unused)]
pub enum InstanceGatherError {
    #[error("Unknown error occured")]
    Unknown,
    #[error("Instance directory path is unset! Head to the settings to change it.")]
    PathUnset,

    #[error("Failed to download icon for instance {0}: {1}")]
    IconDownloadFailed(String, String),

    #[error("Could not read directory at {0}")]
    DirectoryReadFailed(String),
    #[error("Failed to get file type of element at {0:?}: {1}")]
    FileTypeFailed(PathBuf, #[source] io::Error),
    #[error("Failed to read file at {0:?}: {1}")]
    FileReadFailed(PathBuf, #[source] io::Error),
    #[error("Failed to write to file at {0:?}: {1}")]
    FileWriteFailed(PathBuf, #[source] io::Error),

    #[error("Failed to parse {0:?} instance at {1:?}: {2}")]
    ParseFailedIni(InstanceType, PathBuf, #[source] serde_ini::de::Error),
    #[error("Failed to parse {0:?} instance at {1:?}: {2}")]
    ParseFailedJson(InstanceType, PathBuf, #[source] serde_json::Error),
    #[error("Failed to parse json file at {0:?}: {1}")]
    ParseFailedMeta(PathBuf, #[source] serde_json::Error),
    #[error("Failed to parse last played string {0:?}: {1}")]
    NaiveDateTimeParseFailed(String, #[source] chrono::ParseError),
    #[error("Failed to parse icon path {0:?}: {1}")]
    IconPathParseFailed(String, #[source] <PathBuf as FromStr>::Err),

    #[error("Minecraft version could not be found in mmc-pack.json of {0}")]
    MinecraftNotFound(PathBuf),
}


impl Serialize for InstanceGatherError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        error!("{self}");
        serializer.serialize_str(&self.to_string())
    }
}
