use std::fs;

use log::*;
use serde::{Deserialize, Serialize};
use slint::{ModelRc, VecModel};

use crate::{slint_generatedMainWindow::*, launcher::java::JavaDetails};

use super::{consts::SETTINGS_FILE_NAME, slint_utils::SlintOption, utils::get_config_dir};




#[derive(Debug, Deserialize, Serialize)]
pub struct AppSettings {
    pub instance_size: u16,
    pub instance_path: Option<String>,
    pub icon_path: Option<String>,
    pub java_settings: Vec<JavaDetails>
}

impl AppSettings {
    pub fn get() -> Self {
        info!("Reading settings...");
        let path = get_config_dir().join(SETTINGS_FILE_NAME);

        if !path.is_file() {
            info!("Settings not found. Generating...");
            Self::generate();
        }

        let file = fs::read_to_string(path).expect("Failed to read settings file!");
        match serde_json::from_str(&file) {
            Ok(settings) => {
                debug!("Successfully loaded settings: {settings:#?}");
                settings
            },
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

    pub fn update(new_settings: AppSettings) {
        new_settings.set()
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

        debug!("Generating settings at {path:?}");
        fs::write(path, serde_json::to_string_pretty(&defaults).unwrap()).expect("Failed to write to settings file!");
        defaults
    }

    pub fn to_slint(&self) -> SlSettings {
        SlSettings { // this is not nice
            icon_path: SlintOption::from(self.icon_path.clone()).into(),
            icon_path_set: self.icon_path.is_some(),
            instance_path: SlintOption::from(self.instance_path.clone()).into(),
            instance_path_set: self.instance_path.is_some(),
            instance_size: self.instance_size.into(),
            java_settings: ModelRc::new(VecModel::from(
                self.java_settings.iter().map(JavaDetails::to_slint).collect::<Vec<SlJavaDetails>>()
            ))
        }
    }
}