use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LatestInfo {
    pub release: String,
    pub snapshot: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub url: String,
    pub time: DateTime<Utc>,
    #[serde(rename = "release_time")]
    pub release_time: Option<DateTime<Utc>>,
    pub sha1: String,
    #[serde(rename = "compliance_level")]
    pub compliance_level: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionManifest {
    pub latest: LatestInfo,
    pub versions: Vec<VersionInfo>,
}

//////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    action: String,
    pub(crate) os: Option<HashMap<String, String>>,
    features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum GameArgument {
    Simple(String),
    ComplexWithRules {
        rules: Vec<Rule>,
        value: Vec<String>,
    },
    ComplexWithValue(String),
    CatchAll(Value),
}

impl From<String> for GameArgument {
    fn from(s: String) -> Self {
        match s.as_str() {
            _ => GameArgument::Simple(s),
        }
    }
}

impl GameArgument {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            GameArgument::Simple(s) => Some(s),
            GameArgument::CatchAll(s) => s.as_str(),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Arguments {
    pub(crate) game: Vec<GameArgument>,
    pub(crate) jvm: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetIndex {
    pub(crate) id: String,
    sha1: String,
    size: u32,
    #[serde(rename = "totalSize")]
    total_size: u64,
    pub(crate) url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientDownload {
    pub(crate) sha1: String,
    pub(crate) size: u64,
    pub(crate) url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Downloads {
    pub(crate) client: ClientDownload,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaVersion {
    pub(crate) component: String,
    #[serde(rename = "majorVersion")]
    major_version: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactDownload {
    pub(crate) path: String,
    pub(crate) sha1: String,
    pub(crate) size: u64,
    pub(crate) url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LibraryDownloads {
    pub(crate) artifact: Option<ArtifactDownload>,
    pub(crate) classifiers: Option<HashMap<String, ArtifactDownload>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Library {
    pub(crate) downloads: LibraryDownloads,
    pub(crate) name: String,
    pub(crate) rules: Option<Vec<Rule>>,
    pub(crate) natives: Option<HashMap<String, String>>,
    extract: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LogFile {
    id: String,
    sha1: String,
    size: u32,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClientLogging {
    argument: String,
    file: LogFile,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Logging {
    client: ClientLogging,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageInfo {
    #[serde(rename = "minecraftArguments")]
    pub(crate) minecraft_arguments: Option<String>,
    pub(crate) arguments: Option<Arguments>,
    #[serde(rename = "assetIndex")]
    pub(crate) asset_index: AssetIndex,
    pub(crate) assets: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: Option<i32>,
    pub(crate) downloads: Downloads,
    pub(crate) id: String,
    #[serde(rename = "javaVersion")]
    pub(crate) java_version: Option<JavaVersion>,
    pub(crate) libraries: Vec<Library>,
    logging: Option<Logging>,
    #[serde(rename = "mainClass")]
    pub(crate) main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    minimum_launcher_version: u32,
    #[serde(rename = "releaseTime")]
    release_time: Option<DateTime<Utc>>,
    time: String,
    #[serde(rename = "type")]
    pub(crate) type_: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfoMetadata {
    pub(crate) version: String,
    pub(crate) info_version: VersionInfo,
    pub(crate) package: PackageInfo,
}

pub struct VersionMetadataOptions {
    reqwest_timeout: Option<Duration>,
}

impl Default for VersionMetadataOptions {
    fn default() -> Self {
        Self {
            reqwest_timeout: Some(Duration::from_secs(10)),
        }
    }
}

async fn get_version_manifest(options: VersionMetadataOptions) -> Result<VersionManifest, Box<dyn Error>> {
    let current_time = Utc::now();
    let iso_string = current_time.to_rfc3339();
    let url = format!("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json?_t={}", iso_string);
    let timeout_duration = options.reqwest_timeout.unwrap_or(Duration::from_secs(10));

    let client = Client::new();
    let data = client
        .get(url)
        .timeout(timeout_duration)
        .send().await?
        .json::<VersionManifest>().await?;
    Ok(data)
}

pub async fn get_version_metadata(version: &str, mut options: Option<VersionMetadataOptions>) -> Result<InfoMetadata, Box<dyn Error>> {
    options = options.or(Some(VersionMetadataOptions::default()));
    let manifest = get_version_manifest(options.unwrap()).await?;
    let version_id = match version {
        "latest_release" | "r" | "lr" => manifest.latest.release.clone(),
        "latest_snapshot" | "s" | "ls" => manifest.latest.snapshot.clone(),
        _ => version.to_string(),
    };
    let version_info = match manifest.versions.iter().find(|v| v.id == version_id) {
        Some(info) => info.clone(),
        None => return Err("Version not found".into()),
    };

    let package = reqwest::get(&version_info.url)
        .await?
        .json::<PackageInfo>()
        .await?;
    println!("{:?}", package);
    let info_metadata = InfoMetadata {
        version: version_id,
        info_version: version_info,
        package,
    };
    Ok(info_metadata)
}

pub fn is_older(package: &PackageInfo) -> bool {
    package.assets == "legacy" || package.assets == "pre-1.6"
}