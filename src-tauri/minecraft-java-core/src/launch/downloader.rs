use std::io::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Semaphore};

use crate::launch::utils::read_temp_file_content;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileDownloadMetadata {
    #[serde(rename = "type")]
    pub(crate) type_: String,
    pub(crate) path: String,
    pub(crate) executable: Option<bool>,
    pub(crate) sha1: Option<String>,
    pub(crate) size: Option<u64>,
    pub(crate) url: Option<String>,
    pub(crate) content: Option<PathBuf>,
}

struct Downloader {
    path: PathBuf,
    semaphore: Arc<Semaphore>,
}

impl Downloader {
    fn new(path: PathBuf, max_concurrent_downloads: usize) -> Self {
        Downloader {
            path,
            semaphore: Arc::new(Semaphore::new(max_concurrent_downloads)),
        }
    }

    // fn on_progress(&self, handler: Box<dyn Fn(u64, u64, FileDownloadMetadata)>) {
    //     // Abonnement à l'événement de progression
    // }
    //
    // fn on_speed(&self, handler: Box<dyn Fn(u64)>) {
    //     // Abonnement à l'événement de vitesse
    // }
    //
    // fn on_estimated(&self, handler: Box<dyn Fn(u64)>) {
    //     // Abonnement à l'événement de temps estimé
    // }
    //
    // fn on_error(&self, handler: Box<dyn Fn(Error)>) {
    //     // Abonnement à l'événement d'erreur
    // }

    async fn download_file_multiple(&self, files_list: &Vec<FileDownloadMetadata>, _total_size: u64, max_retries: usize) {
        let (tx, mut rx) = mpsc::channel(32);
        let client = Client::new();
        let mut handles = vec![];

        let progress_handle = tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                println!("Progress: {}", progress);
                // Mettre à jour la logique de progression ici
            }
        });

        for file in files_list.iter().cloned() {
            let path = self.path.clone();
            let file_path = path.join(standardize_path(&file.path));

            if file.url.is_none() {
                let bytes = read_temp_file_content(file.content.unwrap()).await.unwrap();
                if let Err(e) = save_to_file(file_path.clone(), &bytes).await {
                    eprintln!("Error storing file: {:?}", e);
                } else {
                    let _ = tx.send(bytes.len() as u64).await;
                }
                continue;
            }

            let tx = tx.clone();
            let client = client.clone();
            let semaphore = self.semaphore.clone();

            if let Some(ref sha1) = file.sha1 {
                if file_exists_and_matches_sha1(&file_path, sha1).await {
                    println!("File already downloaded and verified: {:?}", file_path);
                    continue;
                }
            }

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Failed to acquire semaphore permit");

                println!("Downloading file: {:?}", file);

                for attempt in 0..=max_retries {
                    match client.get(&file.url.clone().unwrap()).timeout(std::time::Duration::from_secs(10)).send().await {
                        Ok(response) if response.status().is_success() => {
                            if let Ok(bytes) = response.bytes().await {
                                if let Err(e) = save_to_file(file_path.clone(), &bytes).await {
                                    eprintln!("Error saving file: {:?}", e);
                                } else {
                                    let _ = tx.send(bytes.len() as u64).await;
                                    break;
                                }
                            }
                        }
                        Ok(response) => {
                            eprintln!("Error downloading file: HTTP Status {}", response.status());
                            if attempt == max_retries {
                                break;
                            }
                        }
                        Err(e) => {
                            if attempt == max_retries {
                                eprintln!("Error downloading file after {} attempts: {:?}", max_retries, e);
                            }
                        }
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            println!("Waiting for download to finish...");
            let _ = handle.await;
        }

        drop(tx);
        let _ = progress_handle.await;
    }
}

async fn file_exists_and_matches_sha1(file_path: &PathBuf, expected_sha1: &str) -> bool {
    if let Ok(mut file) = File::open(file_path).await {
        let mut hasher = Sha1::new();
        let mut buffer = [0; 1024];

        loop {
            match file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => return false,
            }
        }

        let result = hasher.finalize();
        return format!("{:x}", result) == expected_sha1;
    }

    false
}

fn standardize_path(path: &str) -> PathBuf {
    Path::new(path).to_path_buf()
}

async fn save_to_file(file_name: PathBuf, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = file_name.parent() {
        fs::create_dir_all(parent).await?;
    }
    println!("Saving file: {:?}", file_name);
    let mut file = File::create(file_name).await?;
    file.write_all(bytes).await?;
    println!("File saved!");
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadMultipleFilesOptions {
    reqwest_timeout: Option<std::time::Duration>,
}

impl Default for DownloadMultipleFilesOptions {
    fn default() -> Self {
        Self {
            reqwest_timeout: Some(std::time::Duration::from_secs(10)),
        }
    }
}

pub async fn download_single_file(path: PathBuf, file: FileDownloadMetadata, mut options: Option<DownloadMultipleFilesOptions>) {
    options = options.or(Some(DownloadMultipleFilesOptions::default()));
    println!("options: {:?}", options);

    let downloader = Downloader::new(path, 75);
    downloader.download_file_multiple(&vec![file], 0, 100).await;
}

pub async fn download_multiple_files(path: PathBuf, files: &Vec<FileDownloadMetadata>, mut options: Option<DownloadMultipleFilesOptions>) {
    options = options.or(Some(DownloadMultipleFilesOptions::default()));
    println!("options: {:?}", options);

    let downloader = Downloader::new(path, 75);
    downloader.download_file_multiple(files, 0, 100).await;
}
