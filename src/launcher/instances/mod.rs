use std::{cmp::Ordering, fs::File, io::BufReader, path::PathBuf, sync::Arc};

use clone_macro::clone;
use slint::{Image, SharedPixelBuffer};
use tokio::{fs, runtime::Handle, task::JoinSet, time::Instant};
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use log::{*};
use serde::{Deserialize, Serialize};

use crate::{app::{notifier::Notifier, slint_utils::SlintOption, utils::format_time_delta}, SlInstanceType, SlSimpleInstance, YetaLauncher};

use self::{errors::InstanceGatherError, multimc::*, curseforge::*};

use super::modloaders::ModLoaders;

pub mod errors;
pub mod curseforge;
pub mod multimc;

// Instance Gather Result
pub type IResult<T> = core::result::Result<T, InstanceGatherError>;


#[derive(Debug, Clone)]
pub struct SimpleInstance {
    pub name: String,
    pub icon_path: Option<String>,
    pub minecraft_path: PathBuf,
    pub instance_path: PathBuf,
    pub id: u32,
    pub mc_version: String,
    pub modloader: ModLoader,
    pub last_played: Option<DateTime<Utc>>,
    pub last_played_for: Option<TimeDelta>,
    pub total_time_played: Option<TimeDelta>,
    pub play_count: Option<i32>,
    pub instance_type: InstanceType
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModLoader {
    pub name: String,
    pub typ: ModLoaders,
    pub version: String
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstanceType {
    CurseForge,
    MultiMC
}


pub async fn get_instances(app: Arc<YetaLauncher>, notifier: Notifier) -> IResult<Vec<SimpleInstance>> {
    notifier.send_msg("Scanning instances...");
    let time_start = Instant::now();

    let dir = app.settings.read().unwrap().instance_path.clone().ok_or(InstanceGatherError::PathUnset)?;
    let mut paths = fs::read_dir(&dir).await.or(Err(InstanceGatherError::DirectoryReadFailed(dir.to_string())))?;

    let mut instances = Vec::new();
    let mut tasks: JoinSet<Option<IResult<SimpleInstance>>> = JoinSet::new();

    while let Ok(Some(path)) = paths.next_entry().await {
        if path.file_type().await.map_err(
            |err| InstanceGatherError::FileTypeFailed(path.path(), err)
        )?.is_dir() {
            tasks.spawn(clone!([app], async move {
                let p = &path.path();
                trace!("Scanning folder {p:?}");

                if p.join("minecraftinstance.json").is_file() {
                    trace!("Found minecraftinstance.json in {p:?}");
                    Some(SimpleInstance::get_from_cf(&path.path(), app).await)
                } else if p.join("instance.cfg").is_file() {
                    trace!("Found instance.cfg in {p:?}");
                    Some(SimpleInstance::get_from_mmc(&path.path(), app).await)
                } else {
                    info!("The folder at {p:?} does not contain a recognized minecraft instance!");
                    None
                }
            }));
        }
    }

    while let Some(Ok(opt)) = tasks.join_next().await {
        if let Some(result) = opt {
            let instance = result?;
            debug!("{:?} - {} | Last played: {:?}", &instance.instance_type, &instance.name, &instance.last_played);
            instances.push(instance);
        }
    }

    instances.sort_unstable_by(|a, b| 
        if let Some(l_a) = a.last_played {
            if let Some(l_b) = b.last_played {
                l_b.cmp(&l_a)
            } else {
                Ordering::Less
            }
        } else if b.last_played.is_some() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    );

    info!("Finished gathering {} instances in {}s", instances.len(), &(Instant::now() - time_start).as_secs_f32().to_string()[..5]);
    notifier.send_success(&format!("Finished gathering {} instances ({}s)", instances.len(), &(Instant::now() - time_start).as_secs_f32().to_string()[..4]));

    Ok(instances)
}


impl SimpleInstance {
    pub async fn get_from_mmc(path: &PathBuf, app: Arc<YetaLauncher>) -> IResult<Self> {
        let meta = MMCMetadata::get(path).await?;
        let instance_cfg = MMCConfig::get(path).await?;
        let pack_json = MMCPack::get(path).await?;

        Ok(SimpleInstance {
            instance_type: InstanceType::MultiMC,
            instance_path: path.clone(),

            id: meta.instance_id,
            play_count: meta.play_count,

            icon_path: instance_cfg.get_icon(app),
            name: instance_cfg.name,
            last_played: instance_cfg.last_played.and_then(|time| DateTime::from_timestamp_millis(time)),
            last_played_for: instance_cfg.last_played_for.map(|seconds| TimeDelta::seconds(seconds)),
            total_time_played: instance_cfg.total_time_played.map(|seconds| TimeDelta::seconds(seconds)),

            minecraft_path: if path.join(".minecraft").exists() {
                path.join(".minecraft")
            } else {
                path.join("minecraft")
            },
            mc_version: pack_json.components.iter()
                .find(|&comp| comp.uid == "net.minecraft")
                .map(|mc| mc.version.clone())
                .ok_or(InstanceGatherError::MinecraftNotFound(path.clone()))?
                .ok_or(InstanceGatherError::MinecraftNotFound(path.clone()))?,
            modloader: {
                let loader = pack_json.components.iter().find(|&comp| {
                    ModLoaders::from_uid(&comp.uid).is_some()
                });
                if let Some(loader) = loader {
                    ModLoader { 
                        name: loader.cached_name.as_ref().map_or_else(
                            || ModLoaders::from_uid(&loader.uid).unwrap_or(ModLoaders::Vanilla).to_string(), 
                            |name| name.to_string()
                        ),
                        typ: ModLoaders::from_uid(&loader.uid).unwrap_or(ModLoaders::Vanilla),
                        version: loader.version.clone().unwrap_or("Unknown Version!".to_string())
                    }
                } else {
                    ModLoader {
                        name: "Vanilla".into(),
                        typ: ModLoaders::Vanilla,
                        version: "".into(),
                    }
                }
            },
        })
    }

    pub async fn get_from_cf(path: &PathBuf, app: Arc<YetaLauncher>) -> IResult<Self> {
        let meta = CFMetadata::get(path, app).await?;
        let instance_json = CFInstance::get(path).await?;

        Ok(SimpleInstance {
            instance_type: InstanceType::CurseForge,
            minecraft_path: path.clone(),
            instance_path: path.clone(),

            icon_path: meta.saved_icon,
            id: meta.instance_id,
            total_time_played: meta.total_time_played.map(TimeDelta::seconds),

            name: instance_json.name,
            mc_version: instance_json.game_version,
            play_count: Some(instance_json.played_count),
            last_played: {
                let time = NaiveDateTime::parse_and_remainder(&instance_json.last_played, "%Y-%m-%dT%H:%M:%S").map_err(
                    |err| InstanceGatherError::NaiveDateTimeParseFailed(instance_json.last_played.to_string(), err)
                )?.0.and_utc();

                if time.timestamp() > 10 { Some(time) } else { None }
            },
            last_played_for: None,
            modloader: {
                let vanilla = ModLoader {
                    name: "Vanilla".into(),
                    typ: ModLoaders::Vanilla,
                    version: "".into(),
                };
                if let Some(base_loader) = instance_json.base_mod_loader {
                    if let Some(loader) = ModLoaders::from_cf(&base_loader.name) {
                        ModLoader {
                            name: loader.to_string(),
                            typ: loader,
                            version: base_loader.version
                        }
                    } else { vanilla }
                } else { vanilla }
            },
        })
    }

    pub async fn to_slint(&self) -> SlSimpleInstance {
        SlSimpleInstance {
            icon_path: //Image::load_from_path(PathBuf::from(&self.icon_path).as_path()).unwrap_or_default(),
            self.load_image().await.unwrap_or(
                Image::load_from_path(PathBuf::from("resources/default_instance.png").as_path()).unwrap_or_default()
            ),
            id: (self.id as i32).into(),
            instance_path: self.instance_path.to_string_lossy().to_string().into(),
            instance_type: self.instance_type.to_slint(),
            last_played: SlintOption::from(self.last_played.map(
                |time| format!("{} ({} ago)", time.format("%d/%m/%Y, %H:%M"), format_time_delta(Utc::now() - time))
            )).into(),
            last_played_for: SlintOption::from(self.last_played_for.map(format_time_delta)).into(),
            total_time_played: SlintOption::from(self.total_time_played.map(format_time_delta)).into(),
            play_count: SlintOption::from(self.play_count).into(),
            mc_version: self.mc_version.to_string().into(),
            minecraft_path: self.minecraft_path.to_string_lossy().to_string().into(),
            modloader: self.modloader.name.to_string().into(),
            name: self.name.to_string().into()
        }
    }

    async fn load_image(&self) -> Option<Image> {
        if let Some(path) = self.icon_path.clone() {
            let image = Handle::current().spawn(async move {
                
                let reader = BufReader::new(
                    File::open(&path).or_else(
                        |_| File::open(format!("{path}.png"))
                    ).map_err(
                    |err| warn!("Failed to open instance icon '{path}': {err}")
                ).ok()?);
    
                let image = image::io::Reader::new(reader).with_guessed_format().map_err(
                    |err| warn!("Failed to guess icon format '{path}': {err}")
                ).ok()?;
                
                let decoded = image.decode().map_err(
                    |err| warn!("Failed to decode instance icon '{path}': {err}")
                ).ok()?;

                Some(decoded.into_rgba8())
            }).await.unwrap()?;
    
            Some(
                Image::from_rgba8(
                SharedPixelBuffer::clone_from_slice(
                        image.as_raw(),
                        image.width(),
                        image.height()
                    )
                )
            )
        } else { None }
    }
}

impl InstanceType {
    pub fn to_slint(&self) -> SlInstanceType {
        match self { // this sucks even more, but is necessary
            InstanceType::CurseForge => SlInstanceType::CurseForge,
            InstanceType::MultiMC => SlInstanceType::MultiMC,
        }
    }
}