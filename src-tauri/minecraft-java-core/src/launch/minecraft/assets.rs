use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use crate::launch::downloader::FileDownloadMetadata;
use crate::launch::minecraft::json::PackageInfo;
use crate::launch::utils::create_temp_file_with_content;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileDetail {
    hash: String,
    size: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetsManifest {
    pub (crate) objects: HashMap<String, FileDetail>,
}

pub struct AssetsMetadataOptions {
    reqwest_timeout: Option<Duration>,
}

impl Default for AssetsMetadataOptions {
    fn default() -> Self {
        Self {
            reqwest_timeout: Some(Duration::from_secs(10)),
        }
    }
}

async fn get_assets_manifest(url: String, options: AssetsMetadataOptions) -> Result<AssetsManifest, Box<dyn Error>> {
    let timeout_duration = options.reqwest_timeout.unwrap_or(std::time::Duration::from_secs(10));
    let client = Client::new();
    let data = client.get(&url).timeout(timeout_duration).send().await?.json::<AssetsManifest>().await?;
    Ok(data)
}

pub async fn get_game_assets(package: &PackageInfo, mut options: Option<AssetsMetadataOptions>) -> Result<Vec<FileDownloadMetadata>, Box<dyn std::error::Error>> {
    options = options.or(Some(AssetsMetadataOptions::default()));
    let manifest = get_assets_manifest(package.asset_index.url.clone(), options.unwrap()).await?;
    let mut assets = Vec::new();
    let temp_file_path = create_temp_file_with_content(to_string(&manifest).unwrap().as_bytes()).await?;

    assets.push(FileDownloadMetadata {
        type_: "CFILE".to_string(),
        path: format!("assets/indexes/{}.json", package.asset_index.id),
        content: Some(temp_file_path),
        executable: Some(false),
        sha1: None,
        size: None,
        url: None,
    });

    return Ok(assets);

    // for (_, detail) in manifest.objects {
    //     let hash_prefix = &detail.hash[..2];
    //     assets.push(FileDownloadMetadata {
    //         url: Some(format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, detail.hash)),
    //         path: format!("assets/objects/{}/{}", hash_prefix, detail.hash),
    //         sha1: Some(detail.hash),
    //         size: Some(detail.size),
    //         content: None,
    //         executable: Some(false),
    //         type_: "CFILE".to_string(),
    //     });
    // }
    //
    // return Ok(assets);
}