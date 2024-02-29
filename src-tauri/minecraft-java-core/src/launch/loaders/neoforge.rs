use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::launch::downloader::{download_single_file, FileDownloadMetadata};
use crate::launch::loaders::Loader;
use crate::launch::utils::LoaderInfo;

pub async fn install_neoforge(path: PathBuf, loader_config: Loader, loader_info: LoaderInfo) {
    let (file_path, old_api) = download_installer(path, loader_config, loader_info, None).await;
    println!("file_path: {:?}", file_path);
    println!("old_api: {:?}", old_api);
}

#[derive(Debug, Clone)]
pub struct ManifestsOptions {
    reqwest_timeout: Option<Duration>,
}

impl Default for ManifestsOptions {
    fn default() -> Self {
        Self {
            reqwest_timeout: Some(Duration::from_secs(10)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetadataManifest {
    #[serde(rename = "isSnapshot")]
    pub is_snapshot: bool,
    pub versions: Vec<String>,
}

async fn get_metadata_manifest(url: String, options: ManifestsOptions) -> Result<MetadataManifest, Box<dyn Error>> {
    let timeout_duration = options.reqwest_timeout.unwrap_or(Duration::from_secs(10));

    let client = Client::new();
    let data = client
        .get(url)
        .timeout(timeout_duration)
        .send().await?
        .json::<MetadataManifest>().await?;
    Ok(data)
}

async fn download_installer(path: PathBuf, loader_config: Loader, loader_info: LoaderInfo, mut options: Option<ManifestsOptions>) -> (PathBuf, bool) {
    options = options.or(Some(ManifestsOptions::default()));

    let legacy_metadata = get_metadata_manifest(loader_info.legacy_metadata.unwrap(), options.clone().unwrap()).await.unwrap();
    let metadata = get_metadata_manifest(loader_info.metadata, options.clone().unwrap()).await.unwrap();
    let mut old_api = true;
    let mut versions = legacy_metadata.versions.iter()
        .filter(|v| v.contains(&format!("{}-", loader_config.version)))
        .cloned()
        .collect::<Vec<String>>();

    if versions.is_empty() {
        let version_parts: Vec<&str> = loader_config.version.split('.').collect();
        let minecraft_version = format!("{}.{}", version_parts.get(1).unwrap_or(&""), version_parts.get(2).unwrap_or(&""));
        versions = metadata.versions.iter()
            .filter(|&v| v.starts_with(&minecraft_version))
            .cloned()
            .collect::<Vec<String>>();
        old_api = false;
    }
    if versions.is_empty() {
        panic!("No versions found for Neoforge");
    }

    let build = match loader_config.build.as_str() {
        "latest" | "recommended" => versions.last(),
        _ => versions.iter().find(|&loader| loader == loader_config.build.as_str()),
    };

    if build.is_none() {
        panic!("No build found for Neoforge");
    }

    let neoforge_url = if old_api {
        loader_info.legacy_install.unwrap().replace("${version}", build.unwrap())
    } else {
        loader_info.install.unwrap().replace("${version}", build.unwrap())
    };

    let file_path = path.join(format!("neoforge-{}-installer.jar", build.unwrap()));

    download_single_file(path, FileDownloadMetadata {
        type_: "CFILE".to_string(),
        path: file_path.to_string_lossy().to_string(),
        url: Option::from(neoforge_url),
        executable: None,
        content: None,
        sha1: None,
        size: None,
    }, None).await;

    return (file_path, old_api);
}