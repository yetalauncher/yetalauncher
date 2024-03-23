use std::{fs::{self, File}, io::{self, BufReader}, path::PathBuf};

use log::debug;
use reqwest::Client;
use zip::ZipArchive;

use crate::app::{downloader::Download, utils::{download_file_checked, get_library_dir}};

use super::mc_structs::*;

impl MCLibrary {
    pub fn get_lib_downloads(&self) -> Vec<&MCLibraryDownloadsArtifacts> {
        let mut paths = Vec::new();

        if let Some(artifact) = &self.downloads.artifact {
            paths.push(artifact);
        }

        if let Some(native) = self.get_native() {
            paths.push(native);
        }

        paths
    }

    pub fn get_native(&self) -> Option<&MCLibraryDownloadsArtifacts> {
        if let Some(classifiers) = &self.downloads.classifiers {
            if cfg!(windows) {
                classifiers.natives_windows.as_ref()
            } else if cfg!(macos) {
                classifiers.natives_osx.as_ref()
            } else {
                classifiers.natives_linux.as_ref()
            }
        } else { None }
    }
    
    pub fn get_paths(&self) -> Vec<PathBuf> {
        let lib_dir = get_library_dir();
        self.get_lib_downloads().iter().map(|&download| {
            lib_dir.join(&download.path)
        }).collect()
    }

    pub async fn download_checked(&self, client: &Client) {
        let lib_dir = get_library_dir();
        for download in self.get_lib_downloads() {
            download_file_checked(
                client,
                download.sha1.as_ref(),
                &lib_dir.join(&download.path),
                &download.url
            ).await
        }
    }

    pub fn get_downloads(&self) -> Vec<Download> {
        let lib_dir = get_library_dir();

        self.get_lib_downloads()
        .into_iter()
        .map(|dl| Download::new(
            lib_dir.join(&dl.path), &dl.url, dl.sha1.clone(), None)
        ).collect()
    }

    pub fn extract_natives(&self, natives_path: &PathBuf) -> Result<(), std::io::Error> {
        if let(Some(extract_rule), Some(native)) = (&self.extract, self.get_native()) {
            let path = get_library_dir().join(&native.path);
            let reader = BufReader::new(File::open(path)?);
            let mut zip = ZipArchive::new(reader)?;

            for i in 0..zip.len() {
                let file = zip.by_index(i)?;
                let file_path = match file.enclosed_name() {
                    Some(path) => natives_path.join(path),
                    None => continue
                };

                if extract_rule.exclude.iter().any(|exclude| file.name().contains(exclude)) {
                    continue;
                }
                if file_path.exists() {
                    continue;
                }

                if file.is_dir() {
                    fs::create_dir_all(file_path)?;
                } else {
                    if let Some(parent) = file_path.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }

                    debug!("Extracting native library {file_path:?}...");

                    let mut reader = BufReader::new(file);
                    let mut target = File::create(file_path)?;

                    io::copy(&mut reader, &mut target)?;
                }
            }

            Ok(())
        } else { Ok(()) }
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

