use crate::launch::downloader::FileDownloadMetadata;

pub fn check_bundle(_bundle: Vec<FileDownloadMetadata>) -> Vec<FileDownloadMetadata> {
    println!("Checking bundle...");
    return _bundle;
    // println!("{:?}", bundle);
    // let version_metadata = minecraft::json::get_version_metadata("1.6.4", None).await.unwrap().clone();
    // let libraries = get_libraries(&version_metadata.package);
    // let assets = get_assets("https://gist.githubusercontent.com/tacxou/fb1135d15a4772e28d5cf4223553f5fe/raw/cc1b41c2b1e954f32a0ac7b715c80b10e30cb590/assets_manifest.json".to_owned(), None).await.unwrap();
    // let mut path = std::env::current_dir().unwrap();
    // path.push("instances");
    // path.push("miratest");
    // let natives = minecraft::libraries::get_natives(path, &version_metadata.package, libraries);
    // println!("Natives: {:?}", natives);
    // println!("Bundle checked!")
}