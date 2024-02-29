pub fn get_loader() {
    let loader = Loader {
        type_: "neoforge".to_owned(),
        version: "1.20.1".to_string(),
        build: "latest".to_owned(),
        path: Option::from(PathBuf::from("./loader")),
        enable: Option::from(false),
    };
    let path = PathBuf::from("instances");
    install(path, loader);
}