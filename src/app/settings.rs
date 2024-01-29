use std::fs;

use log::warn;
use serde::{Deserialize, Serialize};

use crate::launcher::java::JavaDetails;

use super::utils::get_config_dir;



const SETTINGS_FILE_NAME: &str = "settings.json";


#[derive(Debug, Deserialize, Serialize)]
pub struct AppSettings {
    pub instance_size: u16,
    pub instance_path: Option<String>,
    pub icon_path: Option<String>,
    pub java_settings: Vec<JavaDetails>
}

pub fn get_settings() -> AppSettings {
    AppSettings::get()
}

pub fn update_settings(new_settings: AppSettings) {
    new_settings.set()
}

impl AppSettings {
    pub fn get() -> Self {
        let path = get_config_dir().join(SETTINGS_FILE_NAME);

        if !path.is_file() {
            Self::generate();
        }

        let file = fs::read_to_string(path).expect("Failed to read settings file!");
        match serde_json::from_str(&file) {
            Ok(settings) => settings,
            Err(err) => {
                warn!("Failed to parse settings: {err}, resetting them!");
                Self::generate()
            },
        }
    }

    pub fn set(self) {
        let path = get_config_dir().join(SETTINGS_FILE_NAME);

        fs::write(path, serde_json::to_string_pretty(&self).unwrap()).expect("Failed to write to settings file!");
    }

    fn generate() -> Self {
        let path = get_config_dir().join(SETTINGS_FILE_NAME);

        let defaults = AppSettings {
            instance_size: 16,
            instance_path: None,
            icon_path: None,
            java_settings: Vec::new(),
        };

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create config directory!");
            }
        }

        fs::write(path, serde_json::to_string_pretty(&defaults).unwrap()).expect("Failed to write to settings file!");
        defaults
    }
}