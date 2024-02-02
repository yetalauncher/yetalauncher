use std::process::Command;

use log::{*};
use serde::{Deserialize, Serialize};
use slint::ModelRc;

use super::launching::mc_structs::MCVersionDetails;

use crate::{app::slint_utils::SlintOption, slint_generatedMainWindow::*};


#[derive(Debug, Serialize, Deserialize)]
pub struct JavaDetails {
    pub path: String,
    pub label: String,
    pub version: String,
    pub minecraft_versions: JavaMCRange,
    pub xmx: u32,
    pub xms: u32,
    pub args: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaMCRange {
    min: Option<MCVersionDetails>,
    max: Option<MCVersionDetails>
}

pub fn get_java_version(path: String, args: String) -> Result<String, String> {
    info!("Getting Java version for: {} using args: {}", path, args);

    let java_process = Command::new(path).args(args.split_whitespace()).arg("-version").output();

    if java_process.is_err() {
        warn!("Java test failed!");
        Err(String::from("Executing Java Command failed! Is the java path correct?"))
    } else if java_process.as_ref().unwrap().status.success() {
        let output = java_process.unwrap().stderr;
        info!("Java test succeeded:\n{}", String::from_utf8(output.clone()).unwrap());
        Ok(String::from_utf8(output).unwrap())
    } else {
        let output = java_process.unwrap().stderr;
        warn!("Java command failed:\n{}", String::from_utf8(output.clone()).unwrap());
        Err(String::from_utf8(output).unwrap())
    }
}

impl JavaDetails {
    pub fn get_args(&self) -> String {
        format!("-Xmx{}M -Xms{}M {}", self.xmx, self.xms, self.args)
    }

    pub fn to_slint(&self) -> SlJavaDetails {
        SlJavaDetails {
            args: self.args.to_string().into(),
            label: self.label.to_string().into(),
            path: self.path.to_string().into(),
            version: self.version.to_string().into(),
            xms: (self.xms as i32).into(),
            xmx: (self.xmx as i32).into(),
            minecraft_versions: self.minecraft_versions.to_slint()
        }
    }
}

impl JavaMCRange {
    pub fn to_slint(&self) -> (ModelRc<SlMCVersionDetails>, bool, ModelRc<SlMCVersionDetails>, bool) {
        (
            SlintOption::from(self.min.as_ref().map(|details| details.to_slint())).into(),
            self.min.is_some(),
            SlintOption::from(self.min.as_ref().map(|details| details.to_slint())).into(),
            self.min.is_some()
        )
    }
}
