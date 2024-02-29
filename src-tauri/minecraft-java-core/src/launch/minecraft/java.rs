use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::launch::downloader::FileDownloadMetadata;

use crate::launch::minecraft::json::PackageInfo;
use crate::launch::utils::{get_os_arch_mapping, get_os_name};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaRuntimeMetadata {
    gamecore: Platform,

    linux: Platform,
    #[serde(rename = "linux-i386")]
    linux_i386: Platform,

    #[serde(rename = "mac-os")]
    mac_os: Platform,
    #[serde(rename = "mac-os-arm64")]
    mac_os_arm64: Platform,

    #[serde(rename = "windows-arm64")]
    windows_arm64: Platform,
    #[serde(rename = "windows-x64")]
    windows_x64: Platform,
    #[serde(rename = "windows-x86")]
    windows_x86: Platform,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Platform {
    #[serde(rename = "java-runtime-alpha")]
    java_runtime_alpha: Vec<JavaRuntime>,
    #[serde(rename = "java-runtime-beta")]
    java_runtime_beta: Vec<JavaRuntime>,
    #[serde(rename = "java-runtime-gamma")]
    java_runtime_gamma: Vec<JavaRuntime>,
    #[serde(rename = "java-runtime-gamma-snapshot")]
    java_runtime_gamma_snapshot: Vec<JavaRuntime>,
    #[serde(rename = "jre-legacy")]
    jre_legacy: Vec<JavaRuntime>,
    #[serde(rename = "minecraft-java-exe")]
    minecraft_java_exe: Vec<JavaRuntime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JavaRuntime {
    availability: Availability,
    manifest: Manifest,
    version: Version,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Availability {
    group: u32,
    progress: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Manifest {
    sha1: String,
    size: u32,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Version {
    name: String,
    released: String,
}

#[derive(Clone)]
pub struct JavaMetadataOptions {
    reqwest_timeout: Option<Duration>,
}

impl Default for JavaMetadataOptions {
    fn default() -> Self {
        Self {
            reqwest_timeout: Some(Duration::from_secs(10)),
        }
    }
}

async fn get_java_runtime(options: &JavaMetadataOptions) -> Result<JavaRuntimeMetadata, Box<dyn Error>> {
    let url = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";
    let timeout_duration = options.reqwest_timeout.unwrap_or(Duration::from_secs(10));

    let client = Client::new();
    let data = client
        .get(url)
        .timeout(timeout_duration)
        .send().await?
        .json::<JavaRuntimeMetadata>().await?;
    Ok(data)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JavaManifestMetadata {
    files: HashMap<String, FileType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
enum FileType {
    Directory,
    File {
        downloads: Option<Downloads>,
        executable: Option<bool>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Downloads {
    lzma: Option<FileDownload>,
    raw: Option<FileDownload>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileDownload {
    sha1: String,
    size: u64,
    url: String,
}

async fn get_java_manifest(url: String, options: &JavaMetadataOptions) -> Result<JavaManifestMetadata, Box<dyn Error>> {
    let timeout_duration = options.reqwest_timeout.unwrap_or(Duration::from_secs(10));

    let client = Client::new();
    let data = client
        .get(url)
        .timeout(timeout_duration)
        .send().await?
        .json::<JavaManifestMetadata>().await?;
    Ok(data)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RuntimeManifestMetadata {
    runtime: JavaRuntime,
    manifest: JavaManifestMetadata,
}

async fn get_runtime_manifest(arch_mapping: &str, java_version: &str, options: Option<JavaMetadataOptions>) -> Result<(JavaRuntime, JavaManifestMetadata), Box<dyn Error>> {
    let java_versions_json = get_java_runtime(&options.clone().unwrap()).await?;
    let platform = match arch_mapping {
        "windows-x86" => &java_versions_json.windows_x86,
        "windows-x64" => &java_versions_json.windows_x64,
        "windows-arm64" => &java_versions_json.windows_arm64,
        "linux" => &java_versions_json.linux,
        "linux-i386" => &java_versions_json.linux_i386,
        "mac-os" => &java_versions_json.mac_os,
        "mac-os-arm64" => &java_versions_json.mac_os_arm64,
        _ => panic!("Unsupported OS or architecture"),
    };
    let java_runtime = match java_version {
        "jre-legacy" => &platform.jre_legacy,
        "java-runtime-alpha" => &platform.java_runtime_alpha,
        "java-runtime-beta" => &platform.java_runtime_beta,
        "java-runtime-gamma" => &platform.java_runtime_gamma,
        "java-runtime-gamma-snapshot" => &platform.java_runtime_gamma_snapshot,
        "minecraft-java-exe" => &platform.minecraft_java_exe,
        _ => panic!("Unsupported Java version"),
    };

    return if let Some(runtime) = java_runtime.get(0) {
        let manifest = get_java_manifest(runtime.manifest.url.to_string(), &options.clone().unwrap()).await?;
        Ok((runtime.clone(), manifest))
    } else {
        Err("No Java runtime found".into())
    }
}

async fn process_java_files(java_files: HashMap<String, FileType>, version_name: &String, arch_mapping: &str) -> Vec<FileDownloadMetadata> {
    // let os_specific_file = if cfg!(target_os = "windows") { "bin/javaw.exe" } else { "bin/java" };
    let os_specific_file = if get_os_name() == "windows" { "bin/javaw.exe" } else { "bin/java" };
    let java_path_key = java_files.keys()
        .find(|path| path.ends_with(os_specific_file))
        .expect("Java executable not found").to_string();
    // let to_delete = PathBuf::from(java_path_key)
    //     .parent()
    //     .expect("Failed to find parent directory")
    //     .to_path_buf();
    let to_delete = java_path_key.trim_end_matches(os_specific_file);

    let mut files: Vec<FileDownloadMetadata> = Vec::new();
    for (path, entry) in java_files {
        match entry {
            FileType::Directory => continue,
            FileType::File { downloads, executable } => {
                if let Some(downloads) = downloads {
                    let adjusted_path = path.replace(to_delete, "");
                    // println!("path {:?}, {:?}", path, path.replace(to_delete, ""));
                    files.push(FileDownloadMetadata {
                        path: format!("runtime/jre-{}-{}/{}", version_name, arch_mapping, adjusted_path),
                        // path: format!("runtime/jre-{}-{}/{}", version_name, arch_mapping, path.replace(to_delete.to_str().unwrap_or(""), "")),
                        executable: Option::from(executable.unwrap_or(false)),
                        sha1: Option::from(downloads.raw.clone().unwrap().sha1),
                        size: Option::from(downloads.raw.clone().unwrap().size),
                        url: Option::from(downloads.raw.clone().unwrap().url),
                        type_: "Java".to_string(),
                        content: None,
                    });
                }
            },
        }
    }
    return files;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaFilesMetadata {
    pub(crate) path: String,
    pub(crate) list: Vec<FileDownloadMetadata>,
}

pub async fn get_java_files(package_info: &PackageInfo, mut options: Option<JavaMetadataOptions>) -> Result<JavaFilesMetadata, Box<dyn Error>> {
    options = options.or(Some(JavaMetadataOptions::default()));
    let arch_mapping = get_os_arch_mapping();
    let binding = "jre-legacy".to_string();
    let java_version = package_info.java_version.as_ref()
        .map(|v| &v.component)
        .unwrap_or(&binding);

    let (runtime, manifest) = get_runtime_manifest(arch_mapping, java_version, options).await?;
    let list = process_java_files(manifest.files, &runtime.version.name, arch_mapping).await;

    Ok(JavaFilesMetadata {
        //TODO: replace by resolve path
        path: format!("runtime/jre-{}-{}/bin/java", &runtime.version.name, arch_mapping),
        list,
    })
}