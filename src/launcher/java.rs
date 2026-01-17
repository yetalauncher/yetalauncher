use std::process::Command;

use log::{*};
use serde::{Deserialize, Serialize};
use slint::{Model, ModelRc};

use super::launching::mc_structs::MCSimpleVersion;

use crate::{app::slint_utils::SlintOption, slint_generatedMainWindow::*};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaDetails {
    pub path: String,
    pub label: String,
    pub version: String,
    pub minecraft_versions: JavaMCRange,
    pub xmx: u32,
    pub xms: u32,
    pub args: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaMCRange {
    pub min: Option<MCSimpleVersion>,
    pub max: Option<MCSimpleVersion>
}

pub fn get_java_version(path: String, args: String) -> Result<String, String> {
    info!("Getting Java version for: {} using args: {}", path, args);

    let java_process = Command::new(path).args(args.split_whitespace()).arg("-version").output();

    match java_process {
        Ok(process) => {
            let output = String::from_utf8(process.stderr).unwrap();

            if process.status.success() {
                info!("Java test succeeded:\n{output}");
                Ok(output)
            } else {
                warn!("Java command failed:\n{output}");
                Err(output)
            }
        }
        Err(err) => {
            warn!("Java test failed: {err}");
            Err(String::from("Executing Java Command failed! Is the java path correct? Error: {err}"))
        }
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
            xms: self.xms as i32,
            xmx: self.xmx as i32,
            minecraft_versions: self.minecraft_versions.to_slint()
        }
    }

    pub fn from_slint(slint: SlJavaDetails) -> Self {
        Self {
            path: slint.path.into(),
            label: slint.label.into(),
            version: slint.version.into(),
            minecraft_versions: JavaMCRange::from_slint(slint.minecraft_versions),
            xmx: slint.xmx.try_into().unwrap_or(0),
            xms: slint.xms.try_into().unwrap_or(0),
            args: slint.args.into(),
        }
    }
}

impl JavaMCRange {
    pub fn to_slint(&self) -> (ModelRc<SlMCVersionDetails>, bool, ModelRc<SlMCVersionDetails>, bool) {
        (
            SlintOption::from(self.max.as_ref().map(|details| details.to_slint())).into(),
            self.max.is_some(),
            SlintOption::from(self.min.as_ref().map(|details| details.to_slint())).into(),
            self.min.is_some()
        )
    }

    pub fn from_slint(slint: (ModelRc<SlMCVersionDetails>, bool, ModelRc<SlMCVersionDetails>, bool)) -> Self {
        Self {
            max: if slint.1 {
                Some(
                    MCSimpleVersion::from_slint(slint.0.iter().next().expect("ModelRc was empty!"))
                )
            } else { None },
            min: if slint.3 {
                Some(
                    MCSimpleVersion::from_slint(slint.2.iter().next().expect("ModelRc was empty!"))
                )
            } else { None },
        }
    }
}

impl Default for JavaDetails {
    fn default() -> Self {
        Self {
            path: String::from(""),
            label: String::from(""),
            version: String::from(""),
            minecraft_versions: JavaMCRange { min: None, max: None },
            xmx: 4096,
            xms: 2048,
            args: String::from("-XX:+UseG1GC"),
        }
    }
}
