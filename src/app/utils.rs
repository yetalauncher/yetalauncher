use std::{io::Cursor, path::PathBuf};

use log::*;
use reqwest::Client;
use tokio::{fs::{self, create_dir_all, File}, io};
use sha1_smol::Sha1;
use dirs::{config_dir, data_dir};


/// Checks if the checksum of the file at `path` matches `checksum` and downloads it from `url` if not.
pub async fn download_file_checked(client: &Client, checksum: Option<&String>, path: &PathBuf, url: &String) {
    if !path.is_file() || if let Some(csum) = checksum {
        if let Ok(contents) = fs::read(path).await {
            let contents_checksum = Sha1::from(contents).digest().to_string();
            &contents_checksum != csum
        } else { true }
    } else { false } {
        download_file(client, path, url).await
    } else {
        debug!("Skipped downloading {}", path.to_string_lossy())
    }
}

async fn download_file(client: &Client, path: &PathBuf, url: &String) {
    debug!("Downloading to {} from {url}", path.to_string_lossy());
    let response = client.get(url).send().await.unwrap();
    if let Some(parent_path) = path.parent() {
        if !parent_path.exists() {
            create_dir_all(parent_path).await.expect(&format!("Failed to create directories: {}", parent_path.to_string_lossy()));
        }
    }
    let mut file = File::create(path).await.expect(&format!("Failed create file: {}", path.to_string_lossy()));
    let mut content = Cursor::new(response.bytes().await.unwrap());
    io::copy(&mut content, &mut file).await.expect(&format!("Failed to write to {}", path.to_string_lossy()));
}

pub fn maven_identifier_to_path(identifier: &str) -> String {
    let mut id = identifier.to_string();
    let extension = if let Some(i) = identifier.find("@") {
        let ext = &identifier[i..];
        id = id.replace(ext, "");
        &ext[1..]
    } else { "jar" };

    let parts: Vec<&str> = id.splitn(3, ":").collect();
    let (raw_path, raw_name, raw_version) = (parts[0], parts[1], parts[2]);

    let path = raw_path.replace(".", "/");
    let version_path = raw_version.split(":").next().unwrap_or(raw_version);
    let version = raw_version.replace(":", "-");

    format!("{path}/{raw_name}/{version_path}/{raw_name}-{version}.{extension}")
}

#[derive(Debug, Clone)]
pub struct Notifier {
    #[allow(dead_code)] // for now, until this is reimplemented
    notif_id: String
}

#[derive(Debug, Clone)]
pub struct Notif {
    pub text: String,
    pub progress: u32,
    pub max_progress: u32,
    pub status: NotificationState
}

#[derive(Debug, Clone)]
pub enum NotificationState {
    Running,
    Error,
    Success
}

impl Notifier {
    pub fn notify_status(&self, contents: Notif) {
        info!("Notifying is unimplemented, but: {}, State {:?}, {} of {}", contents.text, contents.status, contents.progress, contents.max_progress);
    }

    pub fn notify(&self, text: &str, status: NotificationState) {
        info!("Notifying is unimplemented, but: {}, State {:?}", text, status);
    }

    pub fn new(notif_id: &str) -> Self {
        Self { notif_id: notif_id.to_string() }
    }
}

impl Notif {
    pub fn new(text: &str, progress: u32, max_progress: u32, status: NotificationState) -> Self {
        Self { text: text.to_string(), progress, max_progress, status}
    }
}


pub fn get_classpath_separator() -> String { String::from(if cfg!(windows) { ";" } else { ":" }) }


pub fn get_config_dir() -> PathBuf { config_dir().expect("Failed to get system config directory!").join("yetalauncher") }
pub fn get_data_dir() -> PathBuf { data_dir().expect("Failed to get system data directory!").join("yetalauncher") }

pub fn get_client_jar_dir() -> PathBuf { get_data_dir().join("client_jars") }
pub fn get_library_dir() -> PathBuf { get_data_dir().join("libraries") }
pub fn get_assets_dir() -> PathBuf { get_data_dir().join("assets") }
pub fn get_log4j_dir() -> PathBuf { get_data_dir().join("log4j_configs") }

pub fn get_forge_cache_dir() -> PathBuf { get_data_dir().join("forge_cache") }