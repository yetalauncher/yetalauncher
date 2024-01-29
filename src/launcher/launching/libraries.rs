use std::path::PathBuf;

use reqwest::Client;

use crate::app::utils::{download_file_checked, get_library_dir};

use super::mc_structs::*;

impl MCLibrary {
    pub fn get_downloads(&self) -> Vec<&MCLibraryDownloadsArtifacts> {
        let mut paths = Vec::new();
        if let Some(artifact) = &self.downloads.artifact {
            paths.push(artifact);
        }
        if let Some(classifiers) = &self.downloads.classifiers {
            let natives = if cfg!(windows) {
                &classifiers.natives_windows
            } else if cfg!(macos) {
                &classifiers.natives_osx
            } else {
                &classifiers.natives_linux // in the hopes of these natives working on platforms like OpenBSD too (probably not)
            };
            if let Some(n) = natives.as_ref() {
                paths.push(n);
            }
        }
        paths
    }
    
    pub fn get_paths(&self) -> Vec<PathBuf> {
        let lib_dir = get_library_dir();
        self.get_downloads().iter().map(|&download| {
            lib_dir.join(&download.path)
        }).collect()
    }

    pub async fn download_checked(&self, client: &Client) {
        let lib_dir = get_library_dir();
        for download in self.get_downloads() {
            download_file_checked(
                client,
                download.sha1.as_ref(),
                &lib_dir.join(&download.path),
                &download.url
            ).await
        }
    }
}

impl MCRule {
    pub fn applies(&self) -> bool {
        if let Some(os_rule) = &self.os {
            let arch_matches = os_rule.arch.as_ref().map_or(true, |arch| {
                match arch.as_str() {
                    "x86" => cfg!(target_arch = "x86"),
                    "x86_64" => cfg!(target_arch = "x86_64"), // haven't seen this one yet, but might exist; won't hurt to have
                    _ => false
                }
            });

            let os_matches = os_rule.name.as_ref().map_or(true, |os| {
                match os.as_str() {
                    "linux" => cfg!(target_os = "linux"),
                    "osx" => cfg!(target_os = "macos"),
                    "windows" => cfg!(target_os = "windows"),
                    _ => false
                }
            });

            match self.action {
                Action::Allow => arch_matches && os_matches,
                Action::Disallow => !(arch_matches && os_matches) // idk if this is accurate, doesn't even happen though
            }
        } else { true }
    }
}

