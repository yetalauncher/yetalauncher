use std::{fs, iter, path::PathBuf, str::FromStr};

use chrono::DateTime;
use log::{*};
use reqwest::Client;

use crate::{app::{consts::MINECRAFT_VERSION_URL, downloader::Downloader, notifier::Notifier, utils::{download_file_checked, get_assets_dir, get_classpath_separator, get_client_jar_dir, get_log4j_dir}}, launcher::modloaders::{fabric::FabricLibrary, LoaderManifests}, slint_generatedMainWindow::SlMCVersionDetails};

use super::mc_structs::*;


impl MCVersionList {
    pub async fn get(client: &Client) -> Option<Self> {
        let version_list: Result<MCVersionList, reqwest::Error> = client.get(MINECRAFT_VERSION_URL).send().await.unwrap().json().await;
        match version_list {
            Ok(list) => Some(list),
            Err(e) => {
                error!("Failed to get Minecraft version list: {e}");
                None
            }
        }
    }
}

impl MCVersionDetails {
    pub async fn from_id(version_id: &str, client: &Client) -> Option<Self> {
        let version_list = MCVersionList::get(client).await?;
        version_list.versions.into_iter().find(|ver| {
            ver.id == version_id
        })
    }

    pub async fn get_manifest(&self, client: &Client) -> Option<MCVersionManifest> {
        let extended_version: Result<MCVersionManifest, reqwest::Error> = client.get(&self.url).send().await.unwrap().json().await;
        match extended_version {
            Ok(ver) => Some(ver),
            Err(e) => {
                error!("Failed to get extended Minecraft version info: {e}");
                None
            }
        }
    }

    pub fn to_simple(self) -> MCSimpleVersion {
        MCSimpleVersion {
            id: self.id,
            typ: self.typ,
            release_time: self.release_time,
        }
    }
}

impl MCSimpleVersion {
    pub fn to_slint(&self) -> SlMCVersionDetails {
        SlMCVersionDetails {
            id: self.id.to_string().into(),
            release_time: self.release_time.to_string().into(),
            typ: self.typ.to_string().into()
        }
    }

    pub fn from_slint(slint: SlMCVersionDetails) -> Self {
        Self {
            id: slint.id.into(),
            typ: slint.typ.into(),
            release_time: DateTime::from_str(&slint.release_time).unwrap()
        }
    }
}

impl MCVersionManifest {
    pub async fn get_jvm_args(&self, client: &Client) -> Vec<String> {
        let mut final_args: Vec<String> = Vec::new();

        if let Some(args) = self.arguments.as_ref() {
            for arg in args.jvm.iter() {
                match arg {
                    MCJvmArg::JvmArg(string) => final_args.push(string.to_string()),
                    MCJvmArg::JvmRule(rule) => {
                        if rule.rules.iter().all(MCRule::applies) {
                            match &rule.value {
                                MCValue::String(string) => final_args.push(string.to_string()),
                                MCValue::StringList(string_list) => final_args.append(&mut string_list.clone())
                            }
                        }
                    }
                }
            }
        }

        if !final_args.iter().any(|arg| arg.contains("-cp")) {
            final_args.push("-cp".to_string());
            final_args.push("${classpath}".to_string());
        }
        if !final_args.iter().any(|arg| arg.contains("java.library.path")) {
            final_args.push("-Djava.library.path=${natives_directory}".to_string());
        }

        if let Some(config) = self.get_log4j_config(client).await {
            final_args.push(config.0.replace("${path}", &config.1.to_string_lossy()))
        }

        final_args
    }

    pub fn get_game_args(&self) -> Vec<String> {
        let mut final_args: Vec<String> = Vec::new();

        match &self.arguments {
            Some(args) => {
                for arg in args.game.iter() {
                    match arg {
                        MCGameArg::GameArg(string) => final_args.push(string.to_string()),
                        MCGameArg::GameRule(rule) => {
                            if rule.rules.iter().all(MCRule::applies) {
                                match &rule.value {
                                    MCValue::String(string) => final_args.push(string.to_string()),
                                    MCValue::StringList(string_list) => final_args.append(&mut string_list.clone()),
                                }
                            }
                        }
                    }
                }
            },
            None => if let Some(args_string) = &self.minecraft_arguments {
                let mut args: Vec<String> = args_string.split_whitespace().map(String::from).collect();
                final_args.append(&mut args);
            } else {
                panic!("No arguments found in this version manifest???")
            }
        }

        final_args
    }

    pub async fn get_classpath(&self, natives_path: &PathBuf, client: &Client, notifier: Notifier) -> String {
        let separator = get_classpath_separator();
        let mut downloader = Downloader::new(notifier, 8);

        let libraries: Vec<&MCLibrary> = self.libraries
            .iter()
            .filter(|&lib| if let Some(rules) = &lib.rules {
                rules.iter().all(MCRule::applies)
            } else { true })
            .collect();

        for lib in &libraries {
            for dl in lib.get_downloads() {
                downloader.add(dl);
            }
        }

        downloader.download_all(true, "Minecraft libraries").await;

        libraries.iter()
            .inspect(|&lib| if lib.extract.is_some() {
                lib.extract_natives(natives_path).unwrap();
            })
            .flat_map(|&lib| lib.get_paths() )
            .map(|path| path.to_string_lossy().to_string() )
            .chain(iter::once(
                self.get_client_jar(client).await.to_string_lossy().to_string()
            ))
            .collect::<Vec<String>>()
            .join(&separator)
    }

    pub fn get_main_class(&self) -> String {
        self.main_class.to_string()
    }

    pub async fn get_client_jar(&self, client: &Client) -> PathBuf {
        let path = get_client_jar_dir().join(format!("{}.jar", self.id));
        download_file_checked(
            client,
            Some(&self.downloads.client.sha1),
            &path,
            &self.downloads.client.url
        ).await;
        path
    }

    pub async fn get_log4j_config(&self, client: &Client) -> Option<(String, PathBuf)> {
        if let Some(logging) = &self.logging {
            let path = get_log4j_dir().join(&logging.client.file.id);
            download_file_checked(
                client,
                Some(&logging.client.file.sha1),
                &path,
                &logging.client.file.url
            ).await;
            Some((logging.client.argument.to_string(), path))
        } else { None }
    }

    pub async fn get_client_assets(&self, client: &Client, notifier: Notifier) -> String {
        let assets_dir = get_assets_dir();
        let index_path = &assets_dir.join("indexes").join(format!("{}.json", &self.asset_index.id));

        if !index_path.exists() {
            download_file_checked(
                client, 
                Some(&self.asset_index.sha1), 
                index_path,
                &self.asset_index.url
            ).await;
    
            let file = fs::read_to_string(index_path).unwrap();
            let index: AssetIndexFile = serde_json::from_str(&file).unwrap();
            let mut downloader = Downloader::new(notifier, 12);
    
            for asset in index.objects {
                let (prefix, name) = (&asset.1.hash[..2], &asset.1.hash);
                let url = format!("https://resources.download.minecraft.net/{prefix}/{name}");
                let path = assets_dir.join("objects").join(prefix).join(name);

                downloader.add_from(path, url, None, Some(asset.1.size));
            }

            downloader.download_all(false, "Minecraft assets").await;
        }

        assets_dir.to_string_lossy().to_string()
    }

    pub fn merge_with(&mut self, other: LoaderManifests) {
        match other {
            LoaderManifests::Fabric(mut fabric) => {
                self.id = fabric.id;
                self.main_class = fabric.main_class;

                if let Some(args) = &mut self.arguments {
                    args.game.append(&mut fabric.arguments.game);
                    args.jvm.append(&mut fabric.arguments.jvm);
                }

                self.libraries.append(
                    &mut fabric.libraries
                    .into_iter()
                    .map(FabricLibrary::to_vanilla)
                    .collect()
                )
            },
            LoaderManifests::Forge(mut forge) => {
                self.id = forge.id;
                self.main_class = forge.main_class;

                if let (Some(args), Some(forge_args)) = (&mut self.arguments, &mut forge.arguments) {
                    args.game.append(&mut forge_args.game);
                    args.jvm.append(&mut forge_args.jvm);
                }
                if let Some(forge_mcargs) = &mut forge.minecraft_arguments {
                    self.minecraft_arguments = Some(forge_mcargs.to_string())
                }

                let mut libraries = Vec::new();

                for lib in forge.libraries {
                    libraries.push(lib.to_vanilla())
                }

                libraries.append(&mut self.libraries);
                self.libraries = libraries;
            },
        }
    }
}
