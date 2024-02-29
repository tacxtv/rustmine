use std::path::PathBuf;
use crate::launch::utils::get_loader_info;

pub mod neoforge;

#[derive(Debug, Clone)]
pub struct Loader {
    pub(crate) type_: String,
    pub(crate) version: String,
    pub(crate) build: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) enable: Option<bool>,
}

pub async fn install(path: PathBuf, loader_config: Loader) {
    let loader_info = get_loader_info(loader_config.type_.as_str());
    match loader_config.type_.as_str() {
        "neoforge" => neoforge::install_neoforge(path, loader_config, loader_info).await,
        _ => println!("Loader not found")
    }
}