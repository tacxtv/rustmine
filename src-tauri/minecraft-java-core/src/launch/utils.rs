use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use uuid::Uuid;

pub fn get_os_name() -> &'static str {
    let platform = env::consts::OS;

    match platform {
        "windows" => "windows",
        "macos" => "osx",
        "linux" => "linux",

        //TODO: Add more OS
        _ => panic!("Unsupported OS"),
    }
}

pub fn get_arch_name() -> &'static str {
    let arch = env::consts::ARCH;

    match arch {
        "x86" | "arm" => "32",
        "x86_64" | "arm64" => "64",

        //TODO: Add more arch
        _ => panic!("Unsupported architecture"),
    }
}

pub fn get_os_arch_mapping() -> &'static str {
    let platform = env::consts::OS;
    let arch = env::consts::ARCH;

    match (platform, arch) {
        ("windows", "x86") => "windows-x86",
        ("windows", "x86_64") => "windows-x64",
        ("windows", "arm") => "windows-arm64",

        ("macos", "x86_64") => "mac-os",
        ("macos", "arm") => "mac-os-arm64",

        ("linux", "x86") => "linux-i386",
        ("linux", "x86_64") => "linux",

        //TODO: Add more OS
        _ => panic!("Unsupported OS or architecture"),
    }
}

pub struct LibraryPath {
    pub(crate) path: String,
    pub(crate) name: String,
}

pub fn get_path_libraries(main: &str, native_string: Option<&str>, force_ext: Option<&str>) -> LibraryPath {
    let lib_split: Vec<&str> = main.split(':').collect();
    let file_name = if lib_split.len() > 3 {
        format!("{}-{}", lib_split[2], lib_split[3])
    } else {
        lib_split[2].to_string()
    };

    let final_file_name = if file_name.contains('@') {
        file_name.replace('@', ".")
    } else {
        format!(
            "{}{}{}",
            file_name,
            native_string.unwrap_or(""),
            force_ext.unwrap_or(".jar")
        )
    };

    let path_lib = format!(
        "{}/{}/{}",
        lib_split[0].replace(".", "/"),
        lib_split[1],
        lib_split[2].split('@').next().unwrap_or_default()
    );

    LibraryPath {
        path: path_lib,
        name: format!("{}-{}", lib_split[1], final_file_name),
    }
}

pub async fn create_temp_file_with_content(content: &[u8]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let temp_dir = env::temp_dir();
    let temp_file_name = Uuid::new_v4().to_string();
    let temp_file_path = temp_dir.join(temp_file_name).with_extension("tmp");

    let mut temp_file = File::create(&temp_file_path)?;
    temp_file.write_all(content)?;

    Ok(temp_file_path)
}

pub async fn read_temp_file_content(temp_file_path: PathBuf) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    println!("Reading temp file: {:?}", temp_file_path);
    let mut file = File::open(temp_file_path.clone())?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    println!("Temp file deleted: {:?}", temp_file_path);
    Ok(content)
}

pub struct LoaderInfo {
    pub(crate) metadata: String,
    pub(crate) json: Option<String>,
    pub(crate) install: Option<String>,
    pub(crate) legacy_metadata: Option<String>,
    pub(crate) legacy_install: Option<String>,
    pub(crate) promotions: Option<String>,
    pub(crate) universal: Option<String>,
    pub(crate) client: Option<String>,
    pub(crate) meta: Option<String>,
}

pub fn get_loader_info(loader_type: &str) -> LoaderInfo {
    match loader_type {
        "forge" => LoaderInfo {
            metadata: "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json".to_string(),
            install: Some("https://maven.minecraftforge.net/net/minecraftforge/forge/${version}/forge-${version}-installer".to_string()),
            promotions: Some("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json".to_string()),
            meta: Some("https://files.minecraftforge.net/net/minecraftforge/forge/${build}/meta.json".to_string()),
            universal: Some("https://maven.minecraftforge.net/net/minecraftforge/forge/${version}/forge-${version}-universal".to_string()),
            client: Some("https://maven.minecraftforge.net/net/minecraftforge/forge/${version}/forge-${version}-client".to_string()),
            legacy_metadata: None,
            legacy_install: None,
            json: None,
        },
        "neoforge" => LoaderInfo {
            metadata: "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge".to_string(),
            install: Some("https://maven.neoforged.net/net/neoforged/neoforge/${version}/neoforge-${version}-installer.jar".to_string()),
            legacy_metadata: Some("https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge".to_string()),
            legacy_install: Some("https://maven.neoforged.net/net/neoforged/forge/${version}/forge-${version}-installer.jar".to_string()),
            promotions: None,
            universal: None,
            client: None,
            json: None,
            meta: None,
        },
        "fabric" => LoaderInfo {
            metadata: "https://meta.fabricmc.net/v2/versions".to_string(),
            json: Some("https://meta.fabricmc.net/v2/versions/loader/{version}/{build}/profile/json".to_string()),
            legacy_metadata: None,
            legacy_install: None,
            promotions: None,
            universal: None,
            install: None,
            client: None,
            meta: None,
        },
        "legacyfabric" => LoaderInfo {
            metadata: "https://meta.legacyfabric.net/v2/versions".to_string(),
            json: Some("https://meta.legacyfabric.net/v2/versions/loader/${version}/${build}/profile/json".to_string()),
            legacy_metadata: None,
            legacy_install: None,
            promotions: None,
            universal: None,
            install: None,
            client: None,
            meta: None,
        },
        "quilt" => LoaderInfo {
            metadata: "https://meta.quiltmc.org/v3/versions".to_string(),
            json: Some("https://meta.quiltmc.org/v3/versions/loader/${version}/${build}/profile/json".to_string()),
            legacy_metadata: None,
            legacy_install: None,
            promotions: None,
            universal: None,
            install: None,
            client: None,
            meta: None,
        },
        _ => panic!("Loader type inconnu!"),
    }
}
