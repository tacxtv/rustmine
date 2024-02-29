mod minecraft;
mod downloader;
mod utils;
mod loaders;

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use minecraft::java::get_java_files;
use minecraft::libraries::{get_assets, get_libraries, get_natives};
use crate::launch::downloader::download_multiple_files;
use crate::launch::loaders::Loader;
use crate::launch::minecraft::arguments::{ArgumentsOptions, get_arguments, JvmMemory};
use crate::launch::minecraft::assets::get_game_assets;
use crate::launch::minecraft::bundle::check_bundle;
use crate::launch::minecraft::java::JavaFilesMetadata;
use crate::launch::minecraft::json::PackageInfo;

#[derive(Debug, Clone)]
pub struct Java {
    pub(crate) path: Option<PathBuf>,
    pub(crate) version: Option<String>,
    pub(crate) type_: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
    pub(crate) fullscreen: Option<bool>,
    // pub(crate) resizable: Option<bool>,
    // pub(crate) title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub(crate) min: Option<String>,
    pub(crate) max: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LaunchMetadata {
    pub(crate) path: PathBuf,
    pub(crate) version: String,
    pub(crate) instance_name: Option<String>,
    pub(crate) loader: Option<Loader>,
    pub(crate) java: Option<Java>,
    pub(crate) screen: Option<Screen>,
    pub(crate) memory: Memory,
}

impl Default for LaunchMetadata {
    fn default() -> Self {
        Self {
            path: "instances".to_owned().parse().unwrap(),
            version: "latest_release".to_owned(),
            instance_name: None,
            loader: {
                Some(Loader {
                    type_: "neoforge".to_owned(),
                    version: "1.20.1".to_string(),
                    build: "latest".to_owned(),
                    path: Option::from(PathBuf::from("./loader")),
                    enable: Option::from(false),
                })
            },
            java: {
                Some(Java {
                    path: None,
                    version: None,
                    type_: Option::from("jre".to_owned()),
                })
            },
            screen: {
                Some(Screen {
                    width: Option::from(None),
                    height: Option::from(None),
                    fullscreen: Option::from(false),
                })
            },
            memory: Memory {
                min: Option::from("2G".to_owned()),
                max: Option::from("4G".to_owned()),
            },
        }
    }
}

pub async fn launch_minecraft(mut options: Option<LaunchMetadata>) {
    options = options.or(Some(LaunchMetadata::default()));

    let mut path = std::env::current_dir().unwrap();
    path.push("instances");
    path.push("miratest2");

    let data = download_minecraft(&path, options.clone().unwrap()).await;
    play_minecraft(&path, data, options.clone().unwrap()).await;
}

struct DownloadedData {
    version: String,
    package: PackageInfo,
    // loader: None,
    java: JavaFilesMetadata,
    has_natives: bool,
}

async fn download_minecraft(path: &PathBuf, options: LaunchMetadata) -> DownloadedData {
    let version_metadata = minecraft::json::get_version_metadata(options.version.as_str(), None).await.unwrap().clone();
    let libraries = get_libraries(&version_metadata.package).await.unwrap();
    let assets = get_assets("https://gist.githubusercontent.com/tacxou/fb1135d15a4772e28d5cf4223553f5fe/raw/cc1b41c2b1e954f32a0ac7b715c80b10e30cb590/assets_manifest.json".to_owned(), None).await.unwrap();
    let game_assets = get_game_assets(&version_metadata.package, None).await.unwrap();
    let java_files = get_java_files(&version_metadata.package, None).await.unwrap().clone();

    // println!("game_assets: {:?}", game_assets.len());

    let mut bundle = Vec::new();
    bundle.extend(libraries.clone());
    bundle.extend(assets.data.clone());
    bundle.extend(game_assets.clone());
    // bundle.extend(java_files.list.clone());
    bundle = check_bundle(bundle);

    println!("bundle: {:?}", bundle);

    download_multiple_files(path.clone(), &bundle, None).await;

    let natives = get_natives(path.clone(), &version_metadata.package, libraries);
    let has_natives = natives.len() > 0;
    println!("{:?}", natives);

    return DownloadedData {
        version: version_metadata.version,
        package: version_metadata.package,
        // loader: None,
        java: java_files,
        has_natives,
    };
}

async fn play_minecraft(path: &PathBuf, data: DownloadedData, options: LaunchMetadata) {
    println!("Playing Minecraft...");

    let minecraft_arguments = get_arguments(path, data.package, &ArgumentsOptions {
        has_natives: data.has_natives,
        memory: JvmMemory {
            min: options.memory.min.unwrap_or("2G".to_owned()),
            max: options.memory.max.unwrap_or("4G".to_owned()),
        },
        game_arguments: None,
        jvm_arguments: None,
    }).await;
    println!("{:?}", minecraft_arguments);

    let mut arguments: Vec<String> = Vec::new();
    arguments.extend(minecraft_arguments.jvm.iter().map(|s| s.to_string()));
    arguments.extend(minecraft_arguments.class_path.iter().map(|s| s.to_string()));
    // arguments.extend(loader_arguments.jvm.iter().map(|s| s.to_string()));
    arguments.push(minecraft_arguments.main_class.to_string());
    arguments.extend(minecraft_arguments.game.iter().map(|s| s.to_string()));
    // arguments.extend(loader_arguments.game.iter().map(|s| s.to_string()));

    println!("path.clone(): {:?}", path.clone());
    println!("data.java.path: {:?}", data.java.path);
    let mut exec_process = path.clone();
    let java_path = PathBuf::from(data.java.path.clone());
    exec_process.push(java_path.as_os_str());
    println!("exec_process: {:?}", exec_process.to_str().unwrap().to_string());
    println!("arguments: {:?}", arguments.join(" "));
    let mut child = Command::new(exec_process.to_str().unwrap().to_string() + ".exe")
        .args(&arguments)
        .current_dir(path.clone())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start minecraft process");

    let stdout = child.stdout.take().expect("failed to capture stdout");
    let stderr = child.stderr.take().expect("failed to capture stderr");


    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("Sortie stdout: {}", line),
                Err(e) => eprintln!("Erreur lors de la lecture de stdout: {}", e),
            }
        }
    });

    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("Sortie stderr: {}", line),
                Err(e) => eprintln!("Erreur lors de la lecture de stderr: {}", e),
            }
        }
    });

    stdout_thread.join().expect("Le thread de stdout a paniqué");
    stderr_thread.join().expect("Le thread de stderr a paniqué");

    // let _ = child.wait().expect("Échec de l'attente du processus enfant");
}