use std::{path::PathBuf, fs::{self, create_dir_all}, iter};

use jars::JarOptionBuilder;
use log::{*};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use tokio::process::Command;

use crate::app::{notifier::Notifier, utils::{download_file_checked, get_classpath_separator, get_client_jar_dir, get_data_dir, get_forge_cache_dir, get_library_dir, maven_identifier_to_path}};

use super::forge::ForgeInstallProfile;

pub struct ForgeInstaller;

impl ForgeInstaller {
    async fn download(mc_ver: &str, forge_ver: &str, client: &Client) -> PathBuf {
        info!("Downloading Forge installer for {mc_ver}-{forge_ver}...");
        let path = get_forge_cache_dir().join(format!("forge-{mc_ver}-{forge_ver}-installer.jar"));
        download_file_checked(
            client,
            None,
            &path,
            &format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_ver}-{forge_ver}/forge-{mc_ver}-{forge_ver}-installer.jar")
        ).await;

        path
    }

    pub async fn prepare_jar(mc_ver: &str, forge_ver: &str, client: &Client, java_path: &str, notifier: &mut Notifier) {
        let path = get_library_dir()
        .join("net/minecraftforge/forge")
        .join(format!("{mc_ver}-{forge_ver}"))
        .join(format!("forge-{mc_ver}-{forge_ver}-client.jar"));

        if !path.is_file() {
            notifier.set_progress(1, 5);
            let mut install_profile = ForgeInstallProfile::get(mc_ver, forge_ver, client, notifier).await.expect("Failed to get Forge install profile!");

            notifier.send_progress("Downloading installer libraries...", 3);
            install_profile.download_libraries(notifier.make_new()).await;

            notifier.send_progress("Running installer processors...", 4);
            install_profile.process(Side::Client, java_path, notifier).await;

            notifier.set_progress(0, 0);
            notifier.send_success("Successfully installed forge");
        }
    }

    /// ### Downloads the Forge installer and extracts the manifest and the install_profile from it
    /// Target location: `forge-{mc_ver}-{forge_ver}-[installer.jar/manifest.json/install_profile.json]` in the forge cache dir
    pub async fn extract_needed(mc_ver: &str, forge_ver: &str, client: &Client, notifier: &mut Notifier) {
        notifier.send_progress("Downloading Forge installer...", 1);
        let installer = Self::download(mc_ver, forge_ver, client).await;

        notifier.send_progress("Extracting Forge installer...", 2);
        debug!("Extracting installer jar...");
        let jar = jars::jar(
            installer, 
            JarOptionBuilder::builder()
            .targets(&vec!["version.json", "install_profile.json", "data"])
            .ext("the library has a bug that requires both of these to be set") // otherwise it will always extract all files
            .build()
        ).expect("Failed to extract Forge installer jar!");

        let files = jar.files.iter().filter(|(_, contents)| !contents.is_empty());
        for (f_path, f_contents) in files {
            let full_path = get_installer_extracts_dir(mc_ver, forge_ver).join(f_path);
            if let Some(p) = full_path.parent() {
                create_dir_all(p).expect(&format!("Failed to create parent directories {p:?}"))
            }
            fs::write(full_path, f_contents).expect("Failed to write manifest to file!");
        }
    }
}



#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeProcessor {
    sides: Option<Vec<Side>>,
    jar: String,
    classpath: Vec<String>,
    args: Vec<String>
}


impl ForgeProcessor {
    pub async fn run(&self, side: &Side, install_profile: &ForgeInstallProfile, java_path: &str) {
        let shouldrun = self.sides.is_none() || self.sides.as_ref().is_some_and(|s| s.contains(side));

        if !shouldrun { return; }

        info!("Starting processor {}...", self.jar);

        let classpath = self.classpath
        .iter()
        .chain(iter::once(&self.jar))
        .map(|cp| get_library_dir().join(maven_identifier_to_path(&cp)).to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join(&get_classpath_separator());

        let args = self.parse_args(install_profile, side);

        let main_class = get_jar_main_class(get_library_dir().join(maven_identifier_to_path(&self.jar)));

        info!("Running processor...");

        let process = Command::new(java_path)
        .arg("-cp").arg(classpath)
        .arg(main_class)
        //.arg("-jar").arg(get_library_dir().join(maven_identifier_to_path(&self.jar)))
        .args(args)
        .spawn()
        .expect(&format!("Failed to run Processor {}", self.jar))
        .wait()
        .await
        .expect(&format!("Failed to wait on Processor {} process", self.jar));

        if process.success() {
            info!("Processor exited successfully.")
        } else {
            panic!("Processor crashed with code: {:?}", process.code())
        }
    }

    fn parse_args(&self, install_profile: &ForgeInstallProfile, side: &Side) -> Vec<String> {
        self.args.iter().map(|arg| {
            let mut final_arg = arg.to_string();
            if arg.starts_with("{") && arg.ends_with("}") {
                let key = &arg[1..arg.len()-1];

                final_arg = match key {
                    "SIDE" => format!("{:?}", side).to_lowercase(),
                    "MINECRAFT_JAR" => get_client_jar_dir().join(format!("{}.jar", &install_profile.minecraft)).to_string_lossy().to_string(),
                    _ => install_profile.data.get(key).expect(&format!("Key {key} was not found in data!")).get_value(side)
                };
            } else if arg.starts_with("[") && arg.ends_with("]") {
                let identifier = &arg[1..arg.len()-1];

                let path = get_library_dir().join(maven_identifier_to_path(identifier));
                
                if !path.is_file() {
                    panic!("File at {:?} could not be found!", path)
                } else {
                    final_arg = path.to_string_lossy().to_string()
                }
            }

            let data_dir = get_data_dir().to_string_lossy().to_string();
            if final_arg.starts_with("/") && !final_arg.contains(data_dir.as_str()) {
                let (mc_ver, forge_ver) = install_profile.version.split_once("-forge-").unwrap();
                final_arg = get_installer_extracts_dir(mc_ver, forge_ver).join(&final_arg[1..]).to_string_lossy().to_string()
            }

            final_arg
        }).collect()
    }

    pub fn get_task(&self) -> &str {
        let arg = &self.args[1];

        if !arg.starts_with("{") && !arg.starts_with("[") && arg == &arg.to_uppercase() {
            arg
        } else {
            &self.jar
        }
    }
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Server,
    Client
}

pub fn get_jar_main_class(jar_path: PathBuf) -> String {
    let jar = jars::jar(
        &jar_path,
        JarOptionBuilder::builder()
        .keep_meta_info().target("META-INF/MANIFEST.MF")
        .ext("MF")
        .build()    
    ).expect(&format!("Failed to open jar {jar_path:?}"));

    let jar_mf = String::from_utf8_lossy(
        jar.files.iter().find(|&(path, _)| {
            path == "META-INF/MANIFEST.MF"
        }).expect(&format!("Could not find MANIFEST.MF in jar {jar_path:?}")).1
    );

    let main_class = jar_mf.split("\n").find(|&line| {
        line.starts_with("Main-Class:")
    }).expect(&format!("Could not find main class in jar manifest {jar_mf}"))
    .split_once(": ")
    .unwrap()
    .1.trim();

    main_class.to_string()
}

pub fn get_installer_extracts_dir(mc_ver: &str, forge_ver: &str) -> PathBuf {
    get_forge_cache_dir().join(format!("forge-{mc_ver}-{forge_ver}"))
}
pub fn get_manifest_path(mc_ver: &str, forge_ver: &str) -> PathBuf {
    get_installer_extracts_dir(mc_ver, forge_ver).join("version.json")
}
pub fn get_install_profile_path(mc_ver: &str, forge_ver: &str) -> PathBuf {
    get_installer_extracts_dir(mc_ver, forge_ver).join("install_profile.json")
}
