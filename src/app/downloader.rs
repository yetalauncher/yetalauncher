use std::{io::{Cursor, Write}, path::PathBuf};

use log::*;
use reqwest::Client;
use sha1_smol::Sha1;
use tokio::{fs, io, sync::mpsc::{unbounded_channel, UnboundedSender}, task::JoinSet};

use super::{notifier::Notifier, utils::split_vec_into};


pub type DResult = Result<(), DownloadErr>;

#[derive(Debug)]
pub enum DownloadErr {
    OutOfRetries,
    NotAFile,
    NoFilePerm(io::Error),
    Request(reqwest::Error),
    Response(reqwest::StatusCode),
    FileCreate(io::Error),
    FileWrite(io::Error)
}

pub struct Downloader {
    downloads: Vec<Download>,
    notifier: Notifier,
    client: Client,
    concurrency: usize
}

#[derive(Debug, Clone)]
pub struct Download {
    sha1: Option<String>,
    size: Option<u32>,
    path: PathBuf,
    url: String
}


impl Downloader {
    pub fn new(notifier: Notifier, concurrent_limit: usize) -> Self {
        Self {
            downloads: Vec::new(),
            client: Client::new(),
            concurrency: concurrent_limit,
            notifier
        }
    }

    pub fn add(&mut self, download: Download) {
        self.downloads.push(download);
    }

    pub fn add_from(&mut self, path: PathBuf, url: String, sha1: Option<String>, size: Option<u32>) {
        self.downloads.push(Download {
            sha1, size, path, url
        });
    }

    pub async fn download_all(&mut self, shuffle: bool, text: &str) {
        let mut tasks = JoinSet::new();
        let mut notifier = self.notifier.make_new();

        let (sender, mut receiver) = unbounded_channel::<()>();
        let (mut count, total) = (0, self.downloads.len() as u32);

        notifier.set_progress(count, total);
        notifier.send_msg(&format!("Downloading {text}..."));

        // Shuffles the download list in an attempt to balance out total download size across batches
        if shuffle { fastrand::shuffle(&mut self.downloads); }

        for batch in split_vec_into(self.downloads.clone(), self.concurrency) {
            let client = self.client.clone();
            let sender = sender.clone();
            let mut notifier = notifier.make_new();

            tasks.spawn(async move {
                Download::download_batch(batch, &client, &mut notifier, sender).await
            });
        }

        while let Some(()) = receiver.recv().await {
            count += 1;

            notifier.set_progress(count, total);
            notifier.send_msg(&format!("Downloading {text}..."));
            
            if count == total { break }
        }

        while let Some(Ok(result)) = tasks.join_next().await {
            result.unwrap();
        }

        notifier.set_progress(0, 0);
        notifier.send_success(&format!("Finished downloading {total} {text}"));
    }
}


impl Download {
    pub fn new(path: PathBuf, url: &str, sha1: Option<String>, size: Option<u32>) -> Self {
        Self { path, url: url.to_string(), sha1, size }
    }

    pub async fn download_batch(batch: Vec<Self>, client: &Client, notifier: &mut Notifier, sender: UnboundedSender<()>) -> DResult {
        for download in batch {
            download.download(client, notifier).await?;
            sender.send(()).unwrap();
        }

        notifier.send_remove();
        Ok(())
    }

    pub async fn download(&self, client: &Client, notifier: &mut Notifier) -> DResult {
        let mut tries = 0;

        while self.should_download().await? {
            tries += 1;

            if tries > 8 {
                Err(DownloadErr::OutOfRetries)?;
            }

            if let Some(parent_path) = self.path.parent() {
                if !parent_path.exists() {
                    fs::create_dir_all(parent_path).await.map_err(
                        |err| DownloadErr::FileCreate(err)
                    )?;
                }
            }
    
            match client.get(&self.url).send().await {
                Ok(mut response) => {
                    if response.status().is_success() {
                        notifier.send_msg(&format!("Downloading: {}", &self.url));
                        trace!("Downloading: {} to {:?}", &self.url, &self.path);
        
                        let total = response.content_length();
                        let mut current: usize = 0;
            
                        let mut bytes: Vec<u8> = Vec::new();
                        let mut writer = fs::File::create(&self.path).await.map_err(DownloadErr::FileCreate)?;
            
                        while let Ok(Some(chunk)) = response.chunk().await {
                            if let Err(err) = bytes.write_all(chunk.as_ref()) {
                                error!("Failed to write to in-memory file??: {err}");
                            }
                            current += chunk.len();
        
                            if let Some(total) = total {
                                notifier.send_msg(&format!("Downloading: {}", &self.url));
                                notifier.set_progress(current as u32 / 1000, total as u32 / 1000);
                            }
                        }
            
                        io::copy(&mut Cursor::new(bytes), &mut writer).await.map_err(DownloadErr::FileWrite)?;
                    } else {
                        Err(DownloadErr::Response(response.status()))?
                    }
                },
                Err(err) => Err(DownloadErr::Request(err))?
            }
        }
        
        Ok(())
    }

    async fn should_download(&self) -> Result<bool, DownloadErr> {
        match self.path.metadata() {
            Ok(meta) => {
                if !meta.is_file() {
                    Err(DownloadErr::NotAFile)
                } else if 
                    self.size.as_ref().map_or(true, |size| meta.len() == *size as u64)
                    &&
                    self.checksum_matches().await
                    &&
                    meta.is_file()
                {
                    trace!("Skipped downloading {}", self.url);
                    Ok(false)
                } else {
                    Ok(true)
                }
            },
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => Ok(true),
                _ => Err(DownloadErr::NoFilePerm(err))
            }
        }
    }

    async fn checksum_matches(&self) -> bool {
        if let Some(sha1) = &self.sha1 {
            if let Ok(contents) = fs::read(&self.path).await {
                let checksum = Sha1::from(contents).digest().to_string();
                &checksum == sha1
            } else { false }
        } else { true }
    }
}