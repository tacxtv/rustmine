use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde_json::Value;
use tokio::fs;
use crate::launch::minecraft::json::{GameArgument, is_older, PackageInfo};
use crate::launch::utils::{get_os_name, get_path_libraries};

struct XboxAccount {
    xuid: Option<String>,
    display_name: Option<String>,
}

struct Meta {
    type_: String,
}

struct Authenticator {
    access_token: String,
    name: String,
    uuid: String,
    xbox_account: Option<XboxAccount>,
    meta: Option<Meta>,
    user_properties: String,
    client_id: Option<String>,
    client_token: Option<String>,
}

impl Authenticator {
    fn get_xuid(&self) -> String {
        match &self.xbox_account {
            Some(xbox_account) => {
                match &xbox_account.xuid {
                    Some(xuid) => xuid.clone(),
                    None => self.access_token.clone(),
                }
            },
            None => self.access_token.clone(),
        }
    }

    fn get_client_id_or_token(&self) -> &str {
        self.client_id.as_deref()
            .or(self.client_token.as_deref())
            .unwrap_or(&self.access_token)
    }
}

pub struct JvmMemory {
    pub(crate) min: String,
    pub(crate) max: String,
}

pub struct ArgumentsOptions {
    pub(crate) has_natives: bool,
    pub(crate) memory: JvmMemory,
    pub game_arguments: Option<String>,
    pub jvm_arguments: Option<String>,
}

#[derive(Debug)]
pub struct ArgumentsResult {
    pub(crate) game: Vec<String>,
    pub(crate) jvm: Vec<String>,
    pub(crate) class_path: Vec<String>,
    pub(crate) main_class: String,
}

pub async fn get_arguments(path: &PathBuf, package: PackageInfo, options: &ArgumentsOptions) -> ArgumentsResult {
    let game = get_game_arguments(path, &package, options);
    let jvm = get_jvm_arguments(path, &package, options).await;
    let class_path = get_class_path(path, &package, options);

    ArgumentsResult {
        game,
        jvm,
        class_path: class_path.class_path,
        main_class: class_path.main_class,
    }
}

fn get_game_arguments(path: &PathBuf, package: &PackageInfo, options: &ArgumentsOptions) -> Vec<String> {
    let authenticator = Authenticator {
        access_token: "xxxxx".to_string(),
        name: "Tacxounet".to_string(),
        uuid: "xxxxx".to_string(),
        xbox_account: Some(XboxAccount {
            xuid: Option::from("2535436694008266".to_string()),
            display_name: Option::from("xxxxx".to_string()),
        }),
        meta: Some(Meta {
            type_: "msa".to_string(),
        }),
        user_properties: "xxxxx".to_string(),
        client_id: Option::from("xxxxx".to_string()),
        client_token: Option::from("xxxxx".to_string()),
    };

    let mut game: Vec<GameArgument> = match &package.minecraft_arguments {
        Some(args) => args.split_whitespace()
            .map(String::from)
            .map(GameArgument::from)
            .collect(),
        None => package.arguments.as_ref().map_or(vec![], |args| args.game.clone()),
    };

    let mut table = HashMap::new();
    table.insert("${auth_access_token}".to_string(), authenticator.access_token.clone());
    table.insert("${auth_session}".to_string(), authenticator.access_token.clone());
    table.insert("${auth_player_name}".to_string(), authenticator.name.clone());
    table.insert("${auth_uuid}".to_string(), authenticator.uuid.clone());
    table.insert("${auth_xuid}".to_string(), authenticator.get_xuid());
    table.insert("${user_properties}".to_string(), authenticator.user_properties.clone());
    table.insert("${user_type}".to_string(), authenticator.meta.as_ref().map_or("legacy".to_string(), |meta| meta.type_.clone()));
    table.insert("${version_name}".to_string(), package.id.clone());
    table.insert("${assets_index_name}".to_string(), package.asset_index.id.clone());
    table.insert("${game_directory}".to_string(), path.as_os_str().to_str().unwrap().to_string());
    // table.insert("${assets_root}".to_string(), is_older(&package)
    //     .then(|| path.join("resources").to_str().unwrap().to_string())
    //     .unwrap_or_else(|| path.join("assets").to_str().unwrap().to_string())
    // );
    table.insert("${assets_root}".to_string(), is_older(&package)
        .then(|| "resources".to_string())
        .unwrap_or_else(|| "assets".to_string())
    );
    table.insert("${game_assets}".to_string(), table.get("${assets_root}").unwrap().clone());
    table.insert("${version_type}".to_string(), package.type_.clone());
    table.insert("${clientid}".to_string(), authenticator.get_client_id_or_token().to_string());

    let mut to_remove = Vec::new();
    let mut to_replace = Vec::new();

    for (i, item) in game.iter().enumerate() {
        match item {
            GameArgument::ComplexWithRules { .. } | GameArgument::ComplexWithValue(_) | GameArgument::CatchAll(_) => {
                //TODO implement custom rules
                to_remove.push(i);
            },
            _ => {
                if let Some(s) = item.as_str() {
                    if let Some(replacement) = table.get(s) {
                        to_replace.push((i, replacement.clone()));
                    }
                }
            },
        }
    }

    for (i, replacement) in to_replace {
        game[i] = GameArgument::from(replacement);
    }

    for i in to_remove.iter().rev() {
        game.remove(*i);
    }

    game.extend(options.game_arguments.clone().unwrap_or_default().split_whitespace().map(String::from).map(GameArgument::from));
    game.iter()
        .filter_map(|arg| arg.as_str())
        .map(|s| s.to_string())
        .collect()
}

async fn get_jvm_arguments(path: &PathBuf, package: &PackageInfo, options: &ArgumentsOptions) -> Vec<String> {
    let os = get_os_name();
    let mut opts = HashMap::new();
    opts.insert("windows", "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump");
    opts.insert("macos", "-XstartOnFirstThread");
    opts.insert("linux", "-Xss1M");

    let mut jvm = vec![
        format!("-Xms{}", options.memory.min),
        format!("-Xmx{}", options.memory.max),
        "-XX:+UnlockExperimentalVMOptions".to_string(),
        "-XX:G1NewSizePercent=20".to_string(),
        "-XX:G1ReservePercent=20".to_string(),
        "-XX:MaxGCPauseMillis=50".to_string(),
        "-XX:G1HeapRegionSize=32M".to_string(),
        "-Dfml.ignoreInvalidMinecraftCertificates=true".to_string(),
        format!("-Djna.tmpdir=versions/{}/natives", package.id),
        // format!("-Djna.tmpdir={}/versions/{}/natives", path.to_str().unwrap().to_string(), package.id),
        format!("-Dorg.lwjgl.system.SharedLibraryExtractPath=versions/{}/natives", package.id),
        // format!("-Dorg.lwjgl.system.SharedLibraryExtractPath={}/versions/{}/natives", path.to_str().unwrap().to_string(), package.id),
        format!("-Dio.netty.native.workdir=versions/{}/natives", package.id),
        // format!("-Dio.netty.native.workdir={}/versions/{}/natives", path.to_str().unwrap().to_string(), package.id),
    ];

    if package.minecraft_arguments.is_none() {
        if let Some(opt) = opts.get(os) {
            jvm.push(opt.to_string());
        }
    }

    if options.has_natives {
        jvm.push(format!("-Djava.library.path={}/versions/{}/natives", path.to_str().unwrap().to_string(), package.id));
    }

    if os == "macos" {
        let path_assets = format!("{}/assets/indexes/{}.json", path.to_str().unwrap().to_string(), package.assets);
        let assets_content = fs::read_to_string(path_assets).await.unwrap();
        let assets: Value = serde_json::from_str(&assets_content).expect("Failed to parse assets index");

        if let Some(icon_hash) = assets["objects"]["icons/minecraft.icns"]["hash"].as_str() {
            let icon = format!("{}/assets/objects/{}/{}", path.to_str().unwrap().to_string(), &icon_hash[0..2], icon_hash);
            jvm.push("-Xdock:name=Minecraft".to_string());
            jvm.push(format!("-Xdock:icon={}", icon));
        }
    }

    jvm.extend(options.jvm_arguments.clone().unwrap_or_default().split_whitespace().map(String::from));
    jvm
}


fn filter_class_path(class_path: Vec<String>) -> Vec<String> {
    let mut last_segments = HashSet::new();
    class_path.into_iter().filter(|url| {
        url.split('/').last().map_or(false, |last_segment| last_segments.insert(last_segment.to_string()))
    }).collect()
}

pub struct ClassPath {
    pub(crate) main_class: String,
    pub(crate) class_path: Vec<String>,
}

fn get_class_path(path: &PathBuf, package: &PackageInfo, _options: &ArgumentsOptions) -> ClassPath {
    let mut class_path: Vec<String> = Vec::new();
    let mut libraries = package.libraries.clone();
    // if let Some(loader) = loader_json {
    //     if let Some(loader_libraries) = loader.get("libraries") {
    //         libraries.extend(loader_libraries.as_array().unwrap().clone());
    //     }
    // }
    let mut seen = HashSet::new();
    libraries = libraries.into_iter().filter(|lib| seen.insert(lib.name.clone())).collect();

    let platform = get_os_name();
    for lib in libraries {
        if let Some(natives) = &lib.natives {
            let native = natives.get(platform);

            if native.is_none() {
                continue;
            }
        } else if let Some(rules) = &lib.rules {
            if let Some(rule) = rules.get(0) {
                if let Some(os) = &rule.os {
                    if os.get("name").unwrap().as_str() != platform {
                        continue;
                    }
                }
            }
        }

        // let path_libraries = get_path_libraries(&lib.name, None, None);
        // let lib_path = if let Some(loader) = &lib.loader {
        //     format!("{}/libraries/{}", loader, path.display())
        // } else {
        //     format!("{}/libraries/{}", path.to_str().unwrap().to_string(), path_libraries)
        // };

        let lib_parse = get_path_libraries(&lib.name, None, None);
        // let lib_path = format!("{}/libraries/{}/{}", path.to_str().unwrap().to_string(), lib_parse.path, lib_parse.name);
        let lib_path = format!("libraries/{}/{}", lib_parse.path, lib_parse.name);

        class_path.push(lib_path);
    }

    // TODO: loader + mcp
    // class_path.push(format!("{}/versions/{}/{}.jar", path.to_str().unwrap().to_string(), package.id, package.id));
    class_path.push(format!("versions/{}/{}.jar", package.id, package.id));

    let separator = if get_os_name() == "windows" { ";" } else { ":" };
    let filter_class_path = filter_class_path(class_path.clone());

    ClassPath {
        main_class: package.main_class.clone(),
        class_path: vec!["-cp", filter_class_path.join(separator).as_str()].iter().map(|s| s.to_string()).collect(),
    }
}