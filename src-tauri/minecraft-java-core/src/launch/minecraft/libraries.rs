use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use crate::launch::downloader::FileDownloadMetadata;

use crate::launch::minecraft::json::{ArtifactDownload, PackageInfo};
use crate::launch::utils::{create_temp_file_with_content, get_arch_name, get_os_name};

pub async  fn get_libraries(package_info: &PackageInfo) -> Result<Vec<FileDownloadMetadata>, Box<dyn std::error::Error>> {
    let platform = get_os_name();
    let arch = get_arch_name();
    let mut libraries = Vec::new();

    for lib in &package_info.libraries {
        let mut type_ = "Libraries";
        let artifact: Option<ArtifactDownload>;

        if let Some(natives) = &lib.natives {
            type_ = "Natives";
            if let Some(native_str) = natives.get(platform) {
                let modified_native = native_str.replace("${arch}", &arch);
                let art = lib.downloads.classifiers.as_ref().and_then(|map| map.get(&modified_native));
                artifact = art.cloned();
            } else { continue; }
        } else {
            if let Some(rules) = &lib.rules {
                let os_found = rules.iter().any(|rule| {
                    rule.os.as_ref().map_or(false, |os| {
                        os.get("name").map_or(false, |os_name| *os_name == platform)
                    })
                });
                if !os_found { continue; }
            }
            artifact = Some(lib.downloads.artifact.clone().unwrap());
        }

        let artifact = match artifact {
            Some(artifact) => artifact,
            None => continue,
        };
        libraries.push(FileDownloadMetadata {
            type_: type_.to_string(),
            sha1: Some(artifact.sha1),
            size: Some(artifact.size),
            path: format!("libraries/{}", artifact.path),
            url: Some(artifact.url),
            executable: Some(false),
            content: None,
        });
    }

    libraries.push(FileDownloadMetadata {
        type_: "Jar".to_string(),
        sha1: Some(package_info.downloads.client.sha1.clone()),
        size: Some(package_info.downloads.client.size),
        path: format!("versions/{}/{}.jar", package_info.id, package_info.id),
        url: Some(package_info.downloads.client.url.clone()),
        executable: Some(false),
        content: None,
    });
    let temp_file_path = create_temp_file_with_content(to_string(&package_info).unwrap().as_bytes()).await?;
    libraries.push(FileDownloadMetadata {
        type_: "CFILE".to_string(),
        path: format!("versions/{}/{}.json", package_info.id, package_info.id),
        content: Some(temp_file_path),
        executable: Some(false),
        sha1: None,
        size: None,
        url: None,
    });
    return Ok(libraries);
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetsManifest {
    pub(crate) id: String,
    #[serde(rename = "release_time")]
    pub release_time: Option<DateTime<Utc>>,
    pub(crate) data: Vec<FileDownloadMetadata>,
}

pub struct AssetsMetadataOptions {
    reqwest_timeout: Option<std::time::Duration>,
}

impl Default for AssetsMetadataOptions {
    fn default() -> Self {
        Self {
            reqwest_timeout: Some(std::time::Duration::from_secs(10)),
        }
    }
}

async fn get_assets_manifest(url: String, options: AssetsMetadataOptions) -> Result<AssetsManifest, Box<dyn std::error::Error>> {
    let current_time = Utc::now();
    let iso_string = current_time.to_rfc3339();
    let url = format!("{}?t={}", url, iso_string);
    let timeout_duration = options.reqwest_timeout.unwrap_or(std::time::Duration::from_secs(10));
    let client = Client::new();
    let data = client.get(&url).timeout(timeout_duration).send().await?.json::<AssetsManifest>().await?;
    Ok(data)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetsMetadata {
    pub(crate) data: Vec<FileDownloadMetadata>,
}

pub async fn get_assets(url: String, mut options: Option<AssetsMetadataOptions>) -> Result<AssetsMetadata, Box<dyn std::error::Error>> {
    options = options.or(Some(AssetsMetadataOptions::default()));
    let manifest = get_assets_manifest(url, options.unwrap()).await?;
    let mut data = manifest.data;
    let temp_file_path = create_temp_file_with_content(to_string(&data).unwrap().as_bytes()).await?;
    data.push(FileDownloadMetadata {
        type_: "CFILE".to_string(),
        path: format!("versions/{}/assets_manifest.json", manifest.id),
        content: Some(temp_file_path),
        executable: Some(false),
        sha1: None,
        size: None,
        url: None,
    });
    Ok(AssetsMetadata {
        data,
    })
}

pub fn get_natives(path: PathBuf, package_info: &PackageInfo, libraries: Vec<FileDownloadMetadata>) -> Vec<FileDownloadMetadata> {
    let natives: Vec<FileDownloadMetadata> = libraries.iter().filter(|lib| lib.type_ == "Natives").cloned().collect();
    if natives.len() == 0 { return natives; }
    let mut natives_folder = path.clone();
    natives_folder.push("versions");
    natives_folder.push(package_info.id.as_str());
    natives_folder.push("natives");

    if !Path::new(&natives_folder).exists() {
        create_dir_all(&natives_folder).unwrap();
    }
    for native in &natives {
        println!("Native: {:?}", native);
        // TODO
        // let zip = Zip::from(native.url.as_ref().unwrap().as_str());
    }
    return natives;
}