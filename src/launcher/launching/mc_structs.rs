use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value;


#[derive(Debug, Serialize, Deserialize)]
pub struct MCVersionList {
    pub latest: MCLatest,
    pub versions: Vec<MCVersionDetails>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCVersionDetails {
    pub id: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub url: String,
    pub time: DateTime<Utc>,
    pub release_time: DateTime<Utc>,
    pub sha1: String,
    pub compliance_level: u32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MCSimpleVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub release_time: DateTime<Utc>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCLatest {
    pub release: String,
    pub snapshot: String
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCVersionManifest {
    pub arguments: Option<MCArguments>,
    pub minecraft_arguments: Option<String>,
    pub asset_index: MCAssetIndex,
    pub assets: String,
    pub compliance_level: u16,
    pub downloads: MCDownloads,
    pub id: String,
    pub java_version: MCJavaVersion,
    pub libraries: Vec<MCLibrary>,
    pub main_class: String,
    pub minimum_launcher_version: u16,
    pub release_time: String,
    pub time: String,
    pub logging: Option<MCLogging>,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCArguments {
    pub game: Vec<MCGameArg>,
    pub jvm: Vec<MCJvmArg>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCGameArg {
    GameArg(String),
    GameRule(MCGameRule)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCJvmArg {
    JvmArg(String),
    JvmRule(MCJvmRule)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCGameRule {
    pub rules: Vec<MCRule>,
    pub value: MCValue
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCJvmRule {
    pub rules: Vec<MCRule>,
    pub value: MCValue
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCValue {
    String(String),
    StringList(Vec<String>)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCAssetIndex {
    pub id: String,
    pub url: String,
    pub size: u64,
    pub total_size: u64,
    pub sha1: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCDownloads {
    pub client: MCDownload,
    pub client_mappings: Option<MCDownload>,
    pub server: MCDownload,
    pub server_mappings: Option<MCDownload>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCDownload {
    pub url: String,
    pub size: u64,
    pub sha1: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCJavaVersion {
    pub component: String,
    pub major_version: u16
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCLogging {
    pub client: MCLoggingClient
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCLoggingClient {
    pub argument: String,
    pub file: MCLoggingClientFile,
    #[serde(rename = "type")]
    pub typ: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCLoggingClientFile {
    pub id: String,
    pub size: u64,
    pub url: String,
    pub sha1: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndexFile {
    pub objects: HashMap<String, MCAsset>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCAsset {
    pub hash: String,
    pub size: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCLibrary {
    pub downloads: MCLibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<MCRule>>,
    pub natives: Option<Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCLibraryDownloads {
    pub artifact: Option<MCLibraryDownloadsArtifacts>,
    pub classifiers: Option<MCLibraryDownloadsClassifiers>,
    pub natives: Option<HashMap<String, String>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCLibraryDownloadsArtifacts {
    pub path: String,
    pub url: String,
    pub size: u32,
    pub sha1: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MCLibraryDownloadsClassifiers {
    pub natives_linux: Option<MCLibraryDownloadsArtifacts>,
    pub natives_osx: Option<MCLibraryDownloadsArtifacts>,
    pub natives_windows: Option<MCLibraryDownloadsArtifacts>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCRule {
    pub action: Action,
    pub os: Option<OSRule>,
    pub features: Option<FeatureFlags>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Allow,
    Disallow
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub is_demo_user: Option<bool>,
    pub has_custom_resolution: Option<bool>,
    pub has_quick_plays_support: Option<bool>,
    pub is_quick_play_singleplayer: Option<bool>,
    pub is_quick_play_multiplayer: Option<bool>,
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OSRule {
    pub name: Option<String>,
    pub arch: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureRule {
    pub name: String
}